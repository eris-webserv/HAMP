// Deterministic overworld generation using the client's biome and object rules.
// Seed derivation, configured start-area overrides and chunk persistence remain
// server-owned. The data and placement rules mirror ChunkControl,
// ChunkGeneratorOverworld, ConstructionControl and MobControl.
use crate::defs::packet::pack_string;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[path = "overworld_data.rs"]
mod data;
use data::{BIOMES, BLOB_MAP, PAINTS};

#[derive(Clone, Copy)]
struct ObjectDefinition {
    name: &'static str,
    is_item: bool,
    rarity: u8,
    clump: (usize, usize),
    clump_overwrite: &'static str,
    dont_rotate: bool,
    min_depth: f32,
}

struct BiomeDefinition {
    min_depth: f32,
    texture_count: usize,
    scenic_budget: (usize, usize),
    item_budget: (usize, usize),
    dont_spawn_greens: bool,
    copy_mobs_from: i8,
    mobs: &'static [&'static str],
    objects: &'static [ObjectDefinition],
}

pub struct PlacedObject {
    pub cell_x: u8,
    pub cell_z: u8,
    pub rotation: u8,
    pub item_data: Vec<u8>,
}

/// InventoryItem encoding: short properties, string properties, long properties.
pub fn pack_item(name: &str) -> Vec<u8> {
    pack_item_data(name, &[], &[], &[])
}

fn pack_item_data(
    name: &str,
    shorts: &[(&str, i16)],
    strings: &[(&str, &str)],
    longs: &[(&str, i32)],
) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&(shorts.len() as u16).to_le_bytes());
    for (key, value) in shorts {
        out.extend(pack_string(key));
        out.extend_from_slice(&value.to_le_bytes());
    }
    out.extend_from_slice(&((strings.len() + 1) as u16).to_le_bytes());
    out.extend(pack_string("item_id"));
    out.extend(pack_string(name));
    for (key, value) in strings {
        out.extend(pack_string(key));
        out.extend(pack_string(value));
    }
    out.extend_from_slice(&(longs.len() as u16).to_le_bytes());
    for (key, value) in longs {
        out.extend(pack_string(key));
        out.extend_from_slice(&value.to_le_bytes());
    }
    out
}

pub const BIOME_GRASS: u8 = 0;
pub const BIOME_SNOW: u8 = 1;
pub const BIOME_DESERT: u8 = 2;
pub const BIOME_EVERGREEN: u8 = 3;
pub const BIOME_OCEAN: u8 = 4;
pub const BIOME_OCEAN_SHALLOW: u8 = 5;
pub const BIOME_SWAMP: u8 = 6;
pub const BIOME_SWAMP_DARK: u8 = 7;
pub const BIOME_WOODLANDS: u8 = 8;
pub const BIOME_SAKURA: u8 = 9;

#[derive(Clone, Debug)]
pub struct BiomeWeights {
    pub grass: f32,
    pub snow: f32,
    pub desert: f32,
    pub evergreen: f32,
    pub ocean: f32,
    pub swamp: f32,
    pub woodlands: f32,
    pub sakura: f32,
}

impl Default for BiomeWeights {
    fn default() -> Self {
        Self {
            grass: 1.00,
            snow: 0.50,
            desert: 0.50,
            evergreen: 0.50,
            ocean: 1.00,
            swamp: 0.50,
            woodlands: 0.50,
            sakura: 0.50,
        }
    }
}

/// Per-zone biome configuration.
#[derive(Clone, Debug)]
pub struct ZoneConfig {
    pub name: String,
    pub weights: BiomeWeights,
}

impl ZoneConfig {
    pub fn new(name: impl Into<String>, weights: BiomeWeights) -> Self {
        Self {
            name: name.into(),
            weights,
        }
    }

    pub fn default_main() -> Self {
        Self::new("overworld", BiomeWeights::default())
    }
}

// ── WorldTemplate ─────────────────────────────────────────────────────────

/// Top-level world generation configuration.
///
/// `start_biome` forces every chunk within `start_biome_radius` of (0,0)
/// to spawn as that biome — mirrors the `"Biome at start area"` option in
/// the original public server. A negative biome or zero radius disables
/// the override.
#[derive(Clone, Debug)]
pub struct WorldTemplate {
    pub seed: u64,
    pub zones: Vec<ZoneConfig>,
    pub start_biome: i16,
    pub start_biome_radius: i16,
}

impl Default for WorldTemplate {
    fn default() -> Self {
        Self {
            seed: 0,
            zones: vec![ZoneConfig::default_main()],
            start_biome: BIOME_GRASS as i16,
            start_biome_radius: 3,
        }
    }
}

impl WorldTemplate {
    pub fn new(seed: u64, zones: Vec<ZoneConfig>) -> Self {
        Self {
            seed,
            zones,
            start_biome: BIOME_GRASS as i16,
            start_biome_radius: 3,
        }
    }

    fn zone_weights(&self, zone_name: &str) -> &BiomeWeights {
        self.zones
            .iter()
            .find(|z| z.name == zone_name)
            .map(|z| &z.weights)
            .unwrap_or_else(|| &self.zones[0].weights)
    }
}

// ── ChunkBiomeParams ──────────────────────────────────────────────────────

/// Output of the generator for a single chunk.
pub struct ChunkBiomeParams {
    pub biome: i16,
    pub floor_rot: i16,
    pub floor_tex: i16,
    pub mob_a: String,
    pub mob_b: String,
    pub elements: Vec<PlacedObject>,
}

// ── Deterministic RNG ─────────────────────────────────────────────────────
//
// splitmix64 — good avalanche, fast, no state. One call per value needed.
// Mix seed with sector and/or chunk coords to get independent streams.

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

fn rng_u32(seed: u64, salt: u64) -> u32 {
    splitmix64(seed ^ splitmix64(salt)) as u32
}

// ── WorldGenerator ────────────────────────────────────────────────────────

/// SplitMix remains the server's source of deterministic random values.
struct Rng {
    seed: u64,
    counter: u64,
}
impl Rng {
    fn new(seed: u64) -> Self {
        Self { seed, counter: 0 }
    }
    fn raw(&mut self) -> u32 {
        self.counter += 1;
        rng_u32(self.seed, self.counter)
    }
    fn range(&mut self, min: usize, max: usize) -> usize {
        let draw = self.raw();
        if max <= min {
            min
        } else {
            min + draw as usize % (max - min)
        }
    }
    fn value(&mut self) -> f32 {
        (self.raw() >> 8) as f32 / 16_777_215.0
    }
    fn float_range(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.value()
    }
}

struct SectorMap {
    biomes: [[u8; 36]; 36],
    mobs: [(&'static str, &'static str); 36],
}

pub struct WorldGenerator {
    template: WorldTemplate,
    sector_cache: RwLock<HashMap<(String, i32, i32), Arc<SectorMap>>>,
}

impl BiomeWeights {
    fn eligible(&self, depth: f32) -> Vec<(u8, f32)> {
        [
            (BIOME_GRASS, self.grass),
            (BIOME_SNOW, self.snow),
            (BIOME_DESERT, self.desert),
            (BIOME_EVERGREEN, self.evergreen),
            (BIOME_OCEAN, self.ocean),
            (BIOME_SWAMP, self.swamp),
            (BIOME_WOODLANDS, self.woodlands),
            (BIOME_SAKURA, self.sakura),
        ]
        .into_iter()
        .filter(|(id, weight)| {
            weight.is_finite() && *weight > 0.0 && BIOMES[*id as usize].min_depth <= depth
        })
        .collect()
    }

    fn choose(&self, depth: f32, rng: &mut Rng) -> u8 {
        let entries = self.eligible(depth);
        // The default configuration expands to the client's pool [0,0,1,2,3,4,4,6,8,9].
        // Preserve arbitrary fractional weights exposed by server configuration.
        if entries.is_empty() {
            return BIOME_GRASS;
        }
        if entries
            .iter()
            .all(|(_, w)| *w * 2.0 == (*w * 2.0).floor() && *w <= 10000.0)
        {
            let total: usize = entries.iter().map(|(_, w)| (*w * 2.0) as usize).sum();
            let mut index = rng.range(0, total);
            for (id, weight) in &entries {
                let count = (*weight * 2.0) as usize;
                if index < count {
                    return *id;
                }
                index -= count;
            }
        } else {
            let total: f32 = entries.iter().map(|(_, w)| w).sum();
            let mut draw = rng.value() * total;
            for (id, weight) in &entries {
                if draw < *weight {
                    return *id;
                }
                draw -= weight;
            }
        }
        entries.last().unwrap().0
    }
}

fn mob_pair(biome: u8, rng: &mut Rng) -> (&'static str, &'static str) {
    let mut definition = &BIOMES[biome as usize];
    if definition.copy_mobs_from >= 0 {
        definition = &BIOMES[definition.copy_mobs_from as usize];
    }
    if definition.mobs.is_empty() {
        return ("crab", "crab");
    }
    (
        definition.mobs[rng.range(0, definition.mobs.len())],
        definition.mobs[rng.range(0, definition.mobs.len())],
    )
}

fn distance(x: f32, y: f32, z: f32) -> f32 {
    (x * x + y * y + z * z).sqrt()
}

fn chunk_depth(x: i32, z: i32) -> f32 {
    // GameController.DepthAt measures from campos_result after the breeding
    // elevator reaches its gameplay position, including its vertical offset.
    distance(
        x as f32 * 10.0 + 5.0 - (-7.6617),
        -0.762,
        z as f32 * 10.0 + 5.0 - 6.4955,
    )
}

impl WorldGenerator {
    pub fn new(template: WorldTemplate) -> Self {
        Self {
            template,
            sector_cache: RwLock::new(HashMap::new()),
        }
    }
    pub fn template_zones(&self) -> impl Iterator<Item = &str> {
        self.template.zones.iter().map(|z| z.name.as_str())
    }
    pub fn template(&self) -> &WorldTemplate {
        &self.template
    }
    fn sector_of(x: i32, z: i32) -> (i32, i32) {
        (x.div_euclid(36), z.div_euclid(36))
    }
    fn local_in_sector(x: i32, z: i32) -> (usize, usize) {
        (x.rem_euclid(36) as usize, z.rem_euclid(36) as usize)
    }

    fn sector_biomes(&self, zone: &str, x: i32, z: i32) -> Arc<SectorMap> {
        let key = (zone.to_string(), x, z);
        if let Some(map) = self.sector_cache.read().unwrap().get(&key) {
            return Arc::clone(map);
        }
        let map = Arc::new(self.generate_sector(zone, x, z));
        Arc::clone(self.sector_cache.write().unwrap().entry(key).or_insert(map))
    }

    fn generate_sector(&self, zone: &str, sx: i32, sz: i32) -> SectorMap {
        let sector_salt = (sx as u64)
            .wrapping_mul(0x517cc1b727220a95)
            .wrapping_add((sz as u64).wrapping_mul(0x6c62272e07bb0142));
        let mut rng = Rng::new(splitmix64(self.template.seed ^ sector_salt));
        let mut depths = [f32::INFINITY; 36];
        for x in 0..36 {
            for z in 0..36 {
                let depth = distance(
                    ((sx * 36 + x as i32) * 10 + 5) as f32,
                    0.0,
                    ((sz * 36 + z as i32) * 10 + 5) as f32,
                );
                let blob = BLOB_MAP[x][z] as usize;
                depths[blob] = depths[blob].min(depth);
            }
        }
        let mut bases = [None; 36];
        let mut map = SectorMap {
            biomes: [[0; 36]; 36],
            mobs: [("crab", "crab"); 36],
        };
        for x in 0..36 {
            for z in 0..36 {
                let blob = BLOB_MAP[x][z] as usize;
                let base = *bases[blob].get_or_insert_with(|| {
                    let biome = self
                        .template
                        .zone_weights(zone)
                        .choose(depths[blob], &mut rng);
                    map.mobs[blob] = mob_pair(biome, &mut rng);
                    biome
                });
                map.biomes[x][z] = match base {
                    BIOME_SWAMP if rng.value() < 0.5 => BIOME_SWAMP_DARK,
                    BIOME_OCEAN if rng.value() < 0.13 => BIOME_OCEAN_SHALLOW,
                    _ => base,
                };
            }
        }
        map
    }

    pub fn chunk_params(&self, zone: &str, x: i32, z: i32) -> ChunkBiomeParams {
        let (sx, sz) = Self::sector_of(x, z);
        let (lx, lz) = Self::local_in_sector(x, z);
        let map = self.sector_biomes(zone, sx, sz);
        let mut biome = map.biomes[lx][lz];
        let radius = self.template.start_biome_radius;
        if radius > 0
            && x.abs() <= radius as i32
            && z.abs() <= radius as i32
            && (0..10).contains(&self.template.start_biome)
        {
            biome = self.template.start_biome as u8;
        }
        let chunk_salt = (x as u64)
            .wrapping_mul(0x9e3779b97f4a7c15)
            .wrapping_add((z as u64).wrapping_mul(0x6c62272e07bb0142));
        let chunk_seed = splitmix64(self.template.seed ^ chunk_salt);
        let floor_tex =
            (rng_u32(chunk_seed, 0x01) as usize % BIOMES[biome as usize].texture_count) as i16;
        let floor_rot = (rng_u32(chunk_seed, 0x02) % 4) as i16;
        let (mob_a, mob_b) = map.mobs[BLOB_MAP[lx][lz] as usize];
        // Start-area biome overrides should also use that biome's creature list.
        let (mob_a, mob_b) = if biome == map.biomes[lx][lz] {
            (mob_a, mob_b)
        } else {
            mob_pair(biome, &mut Rng::new(chunk_seed ^ 0x4d4f4253))
        };
        ChunkBiomeParams {
            biome: biome as i16,
            floor_rot,
            floor_tex,
            mob_a: mob_a.to_string(),
            mob_b: mob_b.to_string(),
            elements: self.generate_chunk_elements(x, z, biome as i16),
        }
    }

    fn generate_chunk_elements(&self, x: i32, z: i32, biome: i16) -> Vec<PlacedObject> {
        // Preserve the server's existing spawn-pad override.
        if matches!((x, z), (-1, 0) | (-1, 1) | (-2, 0) | (-2, 1)) {
            return Vec::new();
        }
        let chunk_salt = (x as u64)
            .wrapping_mul(0x9e3779b97f4a7c15)
            .wrapping_add((z as u64).wrapping_mul(0x6c62272e07bb0142));
        let mut rng = Rng::new(splitmix64(
            self.template.seed ^ chunk_salt ^ 0x0101_0101_0101_0101,
        ));
        generate_objects(x, z, biome as u8, &mut rng)
    }
}

/// ConstructionControl uses a rotated positive-quadrant 2x2 footprint and
/// centered 3x3/5x5 footprints, never a top-left square for every object.
fn geometry(name: &str, rotation: u8) -> Vec<(i8, i8)> {
    match data::geometry_size(name) {
        2 => match rotation {
            0 => vec![(0, 0), (1, 0), (0, 1), (1, 1)],
            1 => vec![(0, 0), (1, 0), (0, -1), (1, -1)],
            2 => vec![(0, 0), (-1, 0), (0, -1), (-1, -1)],
            _ => vec![(0, 0), (-1, 0), (0, 1), (-1, 1)],
        },
        size @ (3 | 5) => {
            let radius = size / 2;
            (-radius..=radius)
                .flat_map(|x| (-radius..=radius).map(move |z| (x, z)))
                .collect()
        }
        _ => vec![(0, 0)],
    }
}

fn generated_item(name: &str, paint: &str, cx: i32, cz: i32, x: u8, z: u8) -> Vec<u8> {
    if name == "Gold Chest" || name == "Titanium Chest" {
        let id = super::NEXT_UNIQUE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed) as i32;
        pack_item_data(name, &[], &[], &[("basket_id", id)])
    } else if matches!(
        name,
        "cave"
            | "Personal Mine"
            | "Grass Cave Entrance"
            | "Snow Cave Entrance"
            | "Desert Cave Entrance"
            | "Evergreen Cave Entrance"
            | "Ocean Cave Entrance"
            | "Swamp Cave Entrance"
    ) {
        let id = super::NEXT_UNIQUE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed) as i32;
        pack_item_data(
            name,
            &[
                ("outer_item_chunkX", cx as i16),
                ("outer_item_chunkZ", cz as i16),
                ("outer_item_innerX", x as i16),
                ("outer_item_innerZ", z as i16),
            ],
            &[],
            &[("shack_id", id)],
        )
    } else if name == "Flowers" {
        pack_item_data(name, &[], &[("paint", paint)], &[])
    } else {
        pack_item(name)
    }
}

fn mob_item(depth: f32, rng: &mut Rng) -> &'static str {
    let (big, giant) = if depth >= 20.0 {
        (
            (depth * 0.001 + 0.03).min(1.0),
            (depth * 0.00023 + 0.1).min(0.4),
        )
    } else {
        (0.0, 0.0)
    };
    if rng.value() < giant {
        "Mob - Giant"
    } else if rng.value() < big {
        "Mob - Big"
    } else if rng.value() < (depth * -0.0275 + 1.0).max(0.0) {
        "Mob - Tiny"
    } else {
        "Mob - Normal"
    }
}

fn generate_objects(cx: i32, cz: i32, biome: u8, rng: &mut Rng) -> Vec<PlacedObject> {
    let definition = &BIOMES[biome as usize];
    let mut empties = Vec::with_capacity(100);
    for x in 0..10 {
        for z in 0..10 {
            let index = rng.range(0, empties.len());
            empties.insert(index, (x, z));
        }
    }
    let mut filled = [[false; 10]; 10];
    let mut elements = Vec::new();
    let depth = chunk_depth(cx, cz);
    let budget = rng.range(definition.scenic_budget.0, definition.scenic_budget.1 + 1);
    generate_layer(
        &mut elements,
        budget,
        true,
        biome,
        cx,
        cz,
        depth,
        &mut empties,
        &mut filled,
        rng,
    );
    let budget = rng.range(definition.item_budget.0, definition.item_budget.1 + 1);
    generate_layer(
        &mut elements,
        budget,
        false,
        biome,
        cx,
        cz,
        depth,
        &mut empties,
        &mut filled,
        rng,
    );
    if rng.value() < 0.251 {
        let count = if rng.value() < 0.55 { 1 } else { 2 };
        for _ in 0..count {
            if empties.is_empty() {
                break;
            }
            let (x, z) = empties.remove(0);
            elements.push(PlacedObject {
                cell_x: x,
                cell_z: z,
                rotation: 0,
                item_data: pack_item(mob_item(depth, rng)),
            });
        }
    }
    if !definition.dont_spawn_greens && rng.value() < 0.411 && !empties.is_empty() {
        let mut count = if rng.value() < 0.6 { 2 } else { 3 };
        let center = empties[0];
        let mut i = 0;
        while i < empties.len() {
            let (x, z) = empties[i];
            if distance(x as f32 - center.0 as f32, 0.0, z as f32 - center.1 as f32) < 2.5 {
                empties.remove(i);
                elements.push(PlacedObject {
                    cell_x: x,
                    cell_z: z,
                    rotation: 0,
                    item_data: pack_item("Green Blob"),
                });
                count -= 1;
                if count == 0 {
                    break;
                }
            }
            // Match the client's forward scan after removing a list entry.
            i += 1;
        }
    }
    elements
}

#[allow(clippy::too_many_arguments)]
fn generate_layer(
    elements: &mut Vec<PlacedObject>,
    budget: usize,
    scenic: bool,
    biome: u8,
    cx: i32,
    cz: i32,
    depth: f32,
    empties: &mut Vec<(u8, u8)>,
    filled: &mut [[bool; 10]; 10],
    rng: &mut Rng,
) {
    let objects = BIOMES[biome as usize].objects;
    let mut pools: [Vec<usize>; 4] = std::array::from_fn(|_| Vec::new());
    for (i, obj) in objects.iter().enumerate() {
        if scenic == obj.is_item {
            continue;
        }
        match obj.rarity {
            0..=3 => pools[obj.rarity as usize].push(i),
            4 => pools[0].extend([i, i]),
            5 => pools[0].extend([i, i, i]),
            _ => {}
        }
    }
    for _ in 0..budget {
        let special = if biome != BIOME_SWAMP_DARK && biome != BIOME_OCEAN && !scenic {
            if rng.float_range(0.001, 1.0) < 0.042 {
                Some(("Titanium Chest", true))
            } else if rng.float_range(0.001, 1.0) < 0.075 {
                Some(("Gold Chest", true))
            } else if depth > 15.0 && rng.float_range(0.001, 1.0) < 0.096 {
                Some(("Creature Nest", false))
            } else {
                None
            }
        } else {
            None
        };
        let obj = if let Some((name, dont_rotate)) = special {
            ObjectDefinition {
                name,
                is_item: true,
                rarity: 0,
                clump: (0, 0),
                clump_overwrite: "",
                dont_rotate,
                min_depth: 0.0,
            }
        } else {
            let mut rarity = 0;
            if rng.value() >= 0.62 {
                rarity = 1;
                if rng.value() >= 0.7 {
                    rarity = 3;
                    if rng.value() < 0.85 {
                        rarity = 2;
                    }
                }
            }
            let pool = if pools[rarity].is_empty() {
                &pools[0]
            } else {
                &pools[rarity]
            };
            if pool.is_empty() {
                continue;
            }
            objects[pool[rng.range(0, pool.len())]]
        };
        if obj.min_depth > depth {
            continue;
        }
        let mut remaining = rng.range(obj.clump.0, obj.clump.1 + 1);
        let mut name = obj.name;
        let paint = if name == "Flowers" {
            PAINTS[rng.range(0, PAINTS.len())]
        } else {
            ""
        };
        let mut anchor = (0, 0);
        let mut placed = false;
        let mut i = 0;
        while i < empties.len() {
            let (x, z) = empties[i];
            let rotation = rng.range(0, 4) as u8;
            let rotation = if obj.dont_rotate { 0 } else { rotation };
            let footprint = geometry(name, rotation);
            let cells: Vec<(i8, i8)> = footprint
                .iter()
                .map(|(dx, dz)| (x as i8 + dx, z as i8 + dz))
                .collect();
            if cells.iter().any(|(x, z)| {
                !(0..10).contains(x) || !(0..10).contains(z) || filled[*x as usize][*z as usize]
            }) {
                i += 1;
                continue;
            }
            if !placed
                || distance(x as f32 - anchor.0 as f32, 0.0, z as f32 - anchor.1 as f32) <= 3.5
            {
                elements.push(PlacedObject {
                    cell_x: x,
                    cell_z: z,
                    rotation,
                    item_data: generated_item(name, paint, cx, cz, x, z),
                });
                for (x, z) in &cells {
                    filled[*x as usize][*z as usize] = true;
                }
                empties.retain(|(x, z)| !cells.contains(&(*x as i8, *z as i8)));
                if !placed {
                    anchor = (x, z);
                    if !obj.clump_overwrite.trim().is_empty() {
                        name = obj.clump_overwrite;
                    }
                }
                if remaining == 0 {
                    break;
                }
                remaining -= 1;
            }
            placed = true;
            i += 1;
        }
    }
}

/// Keep generated container/entrance IDs above IDs already present in a saved
/// world. This changes no saved chunks or generation seeds.
pub(super) fn reserve_item_ids(bytes: &[u8]) {
    use crate::defs::packet::unpack_string;
    if bytes.len() < 2 {
        return;
    }
    let mut offset = 2;
    let shorts = u16::from_le_bytes([bytes[0], bytes[1]]);
    for _ in 0..shorts {
        let (_, next) = unpack_string(bytes, offset);
        offset = next.saturating_add(2);
    }
    if offset + 2 > bytes.len() {
        return;
    }
    let strings = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]);
    offset += 2;
    for _ in 0..strings {
        let (_, next) = unpack_string(bytes, offset);
        let (_, next) = unpack_string(bytes, next);
        offset = next;
    }
    if offset + 2 > bytes.len() {
        return;
    }
    let longs = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]);
    offset += 2;
    for _ in 0..longs {
        let (key, next) = unpack_string(bytes, offset);
        offset = next;
        if offset + 4 > bytes.len() {
            return;
        }
        let id = i32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        offset += 4;
        if key == "basket_id" || key == "shack_id" {
            super::NEXT_UNIQUE_ID.fetch_max(id as i64 + 1, std::sync::atomic::Ordering::Relaxed);
        }
    }
}
