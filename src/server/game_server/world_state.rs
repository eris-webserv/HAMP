// world_state.rs — in-memory world state for managed game sessions.
//
// Stores chunks, player positions, and zone metadata. Chunks are stored
// in a custom format and serialized to the game's wire format on demand.
//
// Adding objects to a chunk
// ────────────────────────
// 1. Create a `ChunkElement` with rotation + `InventoryItem` data.
// 2. Push it onto `chunk.elements` at the desired `(cell_x, cell_z)`.
// 3. The element will be included in the next `Chunk::to_wire()` call.
//
// InventoryItem is a key-value dictionary with 3 typed sections:
//   [n_shorts:i16] [n × (key:str, val:i16)]
//   [n_strings:i16] [n × (key:str, val:str)]
//   [n_ints:i16] [n × (key:str, val:i32)]

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::defs::packet::pack_string;
use crate::server::game_server::baskets::BasketStore;
use crate::server::game_server::generator::{PlacedObject, WorldGenerator, WorldTemplate};
use crate::server::game_server::special_generators::{
    self, ZoneKind,
};

// ── Position / rotation types ─────────────────────────────────────────────

/// Position on the wire: 4 × i16 (8 bytes).
///
/// World coordinates: x = chunk_x * 10 + local_x / 10, z = chunk_z * 10 + local_z / 10.
/// Y is not transmitted (always 0 on the base path).
#[derive(Clone, Copy, Default, Debug)]
#[allow(dead_code)]
pub struct WorldPosition {
    pub chunk_x: i16,
    pub chunk_z: i16,
    pub local_x: i16,
    pub local_z: i16,
}

/// Quaternion rotation scaled ×100 as 4 × i16.
/// Identity = (0, 0, 0, 100).
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub struct WorldRotation {
    pub qx: i16,
    pub qy: i16,
    pub qz: i16,
    pub qw: i16,
}

impl Default for WorldRotation {
    fn default() -> Self {
        Self { qx: 0, qy: 0, qz: 0, qw: 100 } // identity quaternion
    }
}

// ── Land claim ────────────────────────────────────────────────────────────

/// A land claim on a chunk.  The claim_key has format "zone,chunkX,chunkZ,innerX,innerZ".
/// The same claim is stored on all 3×3 neighbouring chunks for proximity checks.
pub struct LandClaim {
    pub claim_key: String,
    /// Owner username.
    pub user0: String,
    /// Trusted user slot 1.
    pub user1: String,
    /// Trusted user slot 2.
    pub user2: String,
    /// Unix seconds at expiry.
    pub expires_at_secs: u64,
}

impl LandClaim {
    pub fn new(claim_key: String, owner: String, days: u64) -> Self {
        let expires_at_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
            + days * 86400;
        Self { claim_key, user0: owner, user1: String::new(), user2: String::new(), expires_at_secs }
    }

    fn is_expired(&self) -> bool {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        self.expires_at_secs <= now
    }

    fn remaining_dhms(&self) -> (i16, i16, i16, i16) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        let total = self.expires_at_secs.saturating_sub(now);
        let d = (total / 86400) as i16;
        let h = ((total % 86400) / 3600) as i16;
        let m = ((total % 3600) / 60) as i16;
        let s = (total % 60) as i16;
        (d, h, m, s)
    }
}

// ── Chunk element (placed object) ─────────────────────────────────────────

/// A single placed object within a chunk cell.
pub struct ChunkElement {
    /// Cell position within the 10×10 chunk grid (0–9 each).
    pub cell_x: u8,
    pub cell_z: u8,
    /// Placement rotation/direction byte.
    pub rotation: u8,
    /// Raw InventoryItem wire data (UnpackFromWeb format).
    pub item_data: Vec<u8>,
}

// ── Chunk ─────────────────────────────────────────────────────────────────

/// A single chunk in the world grid.
pub struct Chunk {
    pub x: i16,
    pub z: i16,
    pub zone: String,
    pub biome: i16,
    pub floor_rot: i16,
    pub floor_tex: i16,
    pub floor_model: i16,
    pub mob_a: String,
    pub mob_b: String,
    pub elements: Vec<ChunkElement>,
    pub land_claims: HashMap<String, LandClaim>,
}

impl Chunk {
    /// Creates a grass-floor chunk at the given grid position.
    pub fn blank(x: i16, z: i16, zone: &str) -> Self {
        Self {
            x,
            z,
            zone: zone.to_string(),
            biome: 0,       // grass
            floor_rot: 0,
            floor_tex: 1,   // grass texture
            floor_model: 0,
            mob_a: String::new(),
            mob_b: String::new(),
            elements: Vec::new(),
            land_claims: HashMap::new(),
        }
    }

    /// Serializes this chunk to the full S→C 0x0D wire format.
    ///
    /// RE from GSR case 0x0D (CLIENT handler):
    ///   Outer envelope: [0x0D][zone:str][x:i16][z:i16][flag:u8][checkpoint:str]
    ///   If flag==0: inner body is ChunkData::UnpackFromWeb
    ///
    /// ChunkData::UnpackFromWeb reads:
    ///   [x:i16][z:i16][zone:str][biome:i16][floor_rot:i16][floor_tex:i16][floor_model:i16][str sub_zone][str unk]
    ///   [u8 tile_count][tiles...][i16 land_claim_count][claims...]
    /// After UnpackFromWeb, case 0x0D reads: [i16 bandit_camp_count][camps...]
    pub fn to_wire(&self) -> Vec<u8> {
        let mut p = vec![0x0Du8];

        // ── Outer envelope ──
        p.extend(pack_string(&self.zone));           // zone_name (for chunk key lookup)
        p.extend_from_slice(&self.x.to_le_bytes());  // header x
        p.extend_from_slice(&self.z.to_le_bytes());  // header z
        p.push(0x00);                                // flag = 0 (new chunk data)
        p.extend(pack_string(""));                   // checkpoint = "" (new chunk)

        // ── Inner ChunkData::UnpackFromWeb body ──
        p.extend_from_slice(&self.x.to_le_bytes());  // chunk x (again, inside ChunkData)
        p.extend_from_slice(&self.z.to_le_bytes());  // chunk z
        p.extend(pack_string(&self.zone));            // zone_name (inner)
        p.extend_from_slice(&self.biome.to_le_bytes());
        p.extend_from_slice(&self.floor_rot.to_le_bytes());
        p.extend_from_slice(&self.floor_tex.to_le_bytes());
        p.extend_from_slice(&self.floor_model.to_le_bytes());
        p.extend(pack_string(&self.mob_a));           // sub_zone string
        p.extend(pack_string(&self.mob_b));           // unknown string

        // Group elements by (cell_x, cell_z)
        let mut cells: HashMap<(u8, u8), Vec<&ChunkElement>> = HashMap::new();
        for el in &self.elements {
            cells.entry((el.cell_x, el.cell_z)).or_default().push(el);
        }

        p.push(cells.len() as u8); // occupied_tile_count
        for ((cx, cz), items) in &cells {
            p.push(*cx);
            p.push(*cz);
            p.extend_from_slice(&(items.len() as i16).to_le_bytes());
            for item in items {
                p.push(item.rotation);
                p.extend_from_slice(&item.item_data);
            }
        }

        // land claims
        let active: Vec<&LandClaim> = self.land_claims.values()
            .filter(|c| !c.is_expired())
            .collect();
        p.extend_from_slice(&(active.len() as i16).to_le_bytes());
        for claim in &active {
            let (d, h, m, s) = claim.remaining_dhms();
            p.extend(pack_string(&claim.claim_key));
            p.extend(pack_string(&claim.user0));
            p.extend(pack_string(&claim.user1));
            p.extend(pack_string(&claim.user2));
            p.extend_from_slice(&d.to_le_bytes());
            p.extend_from_slice(&h.to_le_bytes());
            p.extend_from_slice(&m.to_le_bytes());
            p.extend_from_slice(&s.to_le_bytes());
        }
        // bandit_camp_count = 0 (read by case 0x0D outer handler via GetShort)
        p.extend_from_slice(&0i16.to_le_bytes());
        p
    }
}

// ── Tracked player state ──────────────────────────────────────────────────

/// Server-side state for a player inside a managed world.
#[allow(dead_code)]
pub struct TrackedPlayer {
    pub position: WorldPosition,
    pub target: WorldPosition,
    pub rotation: WorldRotation,
    pub zone: String,
}

impl TrackedPlayer {
    pub fn new(zone: &str) -> Self {
        Self {
            position: WorldPosition::default(),
            target: WorldPosition::default(),
            rotation: WorldRotation::default(),
            zone: zone.to_string(),
        }
    }
}

fn placed_to_element(p: PlacedObject) -> ChunkElement {
    ChunkElement { cell_x: p.cell_x, cell_z: p.cell_z, rotation: p.rotation, item_data: p.item_data }
}

// ── Zone registry ─────────────────────────────────────────────────────────

/// Item-backed data for zones that live inside a placed object (houses, caves, dimensions, …).
/// Absent for plain zones (overworld, plugin-defined zones with no physical item).
pub struct InteriorData {
    pub item_bytes: Vec<u8>,
    pub rotation: u8,
    pub cx: i16,
    pub cz: i16,
    pub tx: i16,
    pub tz: i16,
    pub outer_zone: String,
    pub kind: ZoneKind,
}

/// One entry in the zone registry.
/// Plain zones (overworld, plugin zones) have `interior: None`.
/// Item-backed zones (houses, caves, dimensions) carry `interior: Some(…)`.
pub struct ZoneEntry {
    pub interior: Option<InteriorData>,
    pub worldgen: bool,
}

impl ZoneEntry {
    pub fn plain() -> Self { Self { interior: None, worldgen: true } }
    pub fn interior(data: InteriorData) -> Self { Self { interior: Some(data), worldgen: false } }
}

// ── World state ───────────────────────────────────────────────────────────

/// Complete state for a single managed world.
#[allow(dead_code)]
pub struct WorldState {
    pub name: String,
    pub default_zone: String,
    pub chunks: RwLock<HashMap<String, HashMap<(i16, i16), Chunk>>>,
    pub players: RwLock<HashMap<String, TrackedPlayer>>,
    pub baskets: BasketStore,
    /// All known zones keyed by name. Pre-populated from the template; extended at runtime
    /// by BUILD (interior items) and plugins (custom zones).
    pub zones: RwLock<HashMap<String, ZoneEntry>>,
    pub(crate) generator: WorldGenerator,
}

impl WorldState {
    /// Creates a new world using the given generation template.
    ///
    /// `grid_radius` pre-generates a square of chunks around the origin;
    /// chunks outside this radius are generated lazily on first access.
    pub fn new(name: &str, grid_radius: i16, template: WorldTemplate) -> Self {
        let default_zone = template.zones.first()
            .map(|z| z.name.clone())
            .unwrap_or_else(|| "overworld".to_string());

        let generator = WorldGenerator::new(template);
        let mut zone_grid: HashMap<(i16, i16), Chunk> = HashMap::new();

        for x in -grid_radius..=grid_radius {
            for z in -grid_radius..=grid_radius {
                let params = generator.chunk_params(&default_zone, x as i32, z as i32);
                zone_grid.insert((x, z), Chunk {
                    x,
                    z,
                    zone: default_zone.clone(),
                    biome:       params.biome,
                    floor_rot:   params.floor_rot,
                    floor_tex:   params.floor_tex,
                    floor_model: 0,
                    mob_a:       params.mob_a,
                    mob_b:       params.mob_b,
                    elements:    params.elements.into_iter().map(placed_to_element).collect(),
                    land_claims: HashMap::new(),
                });
            }
        }

        let mut chunks = HashMap::new();
        chunks.insert(default_zone.clone(), zone_grid);

        // Pre-populate zone registry from template zone names.
        let zone_map: HashMap<String, ZoneEntry> = generator.template_zones()
            .map(|name| (name.to_string(), ZoneEntry::plain()))
            .collect();

        Self {
            name: name.to_string(),
            default_zone,
            chunks: RwLock::new(chunks),
            players: RwLock::new(HashMap::new()),
            baskets: BasketStore::new(),
            zones: RwLock::new(zone_map),
            generator,
        }
    }

    /// Returns the wire-encoded chunk at (zone, x, z), generating one if missing.
    /// Interior zones are never lazily generated; only the default zone supports lazy gen.
    pub fn get_chunk_wire(&self, zone: &str, x: i16, z: i16) -> Vec<u8> {
        {
            let chunks = self.chunks.read().unwrap();
            if let Some(m) = chunks.get(zone) {
                if let Some(chunk) = m.get(&(x, z)) {
                    return chunk.to_wire();
                }
            }
        }

        let zone_info = self.zones.read().unwrap().get(zone).map(|e| {
            (e.worldgen, e.interior.as_ref().map(|i| i.kind.clone()))
        });

        let (worldgen, kind_opt) = zone_info.unwrap_or((false, None));

        // Special interior worldgen for cave/cloud/hell zones.
        if let Some(kind) = kind_opt {
            let shack_id: i32 = zone.strip_prefix("shack")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let world_seed = self.generator.template().seed;

            let (params, floor_model) = match &kind {
                ZoneKind::Hell => (special_generators::generate_hell_chunk(world_seed, shack_id, x, z), 0i16),
                ZoneKind::Cloud => (special_generators::generate_cloud_chunk(world_seed, shack_id, x, z), 0i16),
                ZoneKind::Cave { item_id } => {
                    let fm = special_generators::cave_floor_model(item_id);
                    (special_generators::generate_cave_chunk(world_seed, shack_id, x, z, item_id), fm)
                }
                ZoneKind::House => return Chunk::blank(x, z, zone).to_wire(),
            };

            let chunk = Chunk {
                x, z,
                zone:        zone.to_string(),
                biome:       params.biome,
                floor_rot:   params.floor_rot,
                floor_tex:   params.floor_tex,
                floor_model,
                mob_a:       params.mob_a,
                mob_b:       params.mob_b,
                elements:    params.elements.into_iter().map(placed_to_element).collect(),
                land_claims: HashMap::new(),
            };
            let wire = chunk.to_wire();
            self.chunks.write().unwrap()
                .entry(zone.to_string())
                .or_default()
                .insert((x, z), chunk);
            return wire;
        }

        if !worldgen {
            return Chunk::blank(x, z, zone).to_wire();
        }

        let params = self.generator.chunk_params(zone, x as i32, z as i32);
        let chunk = Chunk {
            x,
            z,
            zone:        zone.to_string(),
            biome:       params.biome,
            floor_rot:   params.floor_rot,
            floor_tex:   params.floor_tex,
            floor_model: 0,
            mob_a:       params.mob_a,
            mob_b:       params.mob_b,
            elements:    params.elements.into_iter().map(placed_to_element).collect(),
            land_claims: HashMap::new(),
        };
        let wire = chunk.to_wire();
        self.chunks.write().unwrap()
            .entry(zone.to_string())
            .or_default()
            .insert((x, z), chunk);
        wire
    }

    /// Adds a land claim to all 9 chunks in the 3×3 neighbourhood of (chunk_x, chunk_z).
    /// The claim key is always relative to the centre chunk.
    pub fn add_land_claims(&self, zone: &str, chunk_x: i16, chunk_z: i16, inner_x: i16, inner_z: i16, owner: &str, days: u64) {
        let claim_key = format!("{},{},{},{},{}", zone, chunk_x, chunk_z, inner_x, inner_z);
        let mut chunks = self.chunks.write().unwrap();
        let worldgen = self.zones.read().unwrap().get(zone).map_or(false, |e| e.worldgen);
        for dx in -1i16..=1 {
            for dz in -1i16..=1 {
                let cx = chunk_x + dx;
                let cz = chunk_z + dz;
                let chunk = chunks.entry(zone.to_string()).or_default()
                    .entry((cx, cz)).or_insert_with(|| {
                        if worldgen {
                            let p = self.generator.chunk_params(zone, cx as i32, cz as i32);
                            Chunk {
                                x: cx, z: cz, zone: zone.to_string(),
                                biome: p.biome, floor_rot: p.floor_rot,
                                floor_tex: p.floor_tex, floor_model: 0,
                                mob_a: p.mob_a, mob_b: p.mob_b,
                                elements: p.elements.into_iter().map(placed_to_element).collect(),
                                land_claims: HashMap::new(),
                            }
                        } else {
                            Chunk::blank(cx, cz, zone)
                        }
                    });
                chunk.land_claims.insert(claim_key.clone(), LandClaim::new(claim_key.clone(), owner.to_string(), days));
            }
        }
    }

    /// Updates a single user slot on the claim at (chunk_x, chunk_z, inner_x, inner_z).
    /// user_index: 0 = owner (user0), 1 = user1, 2 = user2.
    pub fn update_land_claim_user(&self, zone: &str, chunk_x: i16, chunk_z: i16, inner_x: i16, inner_z: i16, user_index: u8, username: &str) {
        let claim_key = format!("{},{},{},{},{}", zone, chunk_x, chunk_z, inner_x, inner_z);
        if let Some(chunk) = self.chunks.write().unwrap()
            .get_mut(zone).and_then(|m| m.get_mut(&(chunk_x, chunk_z)))
        {
            if let Some(claim) = chunk.land_claims.get_mut(&claim_key) {
                match user_index {
                    0 => claim.user0 = username.to_string(),
                    1 => claim.user1 = username.to_string(),
                    2 => claim.user2 = username.to_string(),
                    _ => {}
                }
            }
        }
    }
}
