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
use std::sync::{Mutex, RwLock};
use super::land_claims::{LandClaim, unix_now};

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

// ── Teleporter ────────────────────────────────────────────────────────────

/// A player-built teleporter in a managed world, identified by the location
/// of the teleporter object itself: (zone, chunk_x, chunk_z, inner_x, inner_z).
///
/// Created/updated by C→S 0x33 (finished editing) and 0x30 (screenshot
/// upload); served to clients as S→C 0x2F pages and 0x32 screenshots.
/// Entries with an empty `title` exist only to hold an early screenshot
/// upload and are not listed until a 0x33 arrives.
pub struct Teleporter {
    pub title: String,
    pub description: String,
    pub zone: String,
    pub cx: i16,
    pub cz: i16,
    pub tx: i16,
    pub tz: i16,
    /// Username of the player who set the teleporter up.
    pub built_by: String,
    /// Screenshot image bytes uploaded by the builder (empty until received).
    pub screenshot: Vec<u8>,
}

impl Teleporter {
    /// Identity string the client uses to key its screenshot cache:
    /// "zone,cx,cz,tx,tz" (same concat PackPageOfTeleporters builds).
    pub fn tele_str(&self) -> String {
        format!("{},{},{},{},{}", self.zone, self.cx, self.cz, self.tx, self.tz)
    }

    pub fn is_at(&self, zone: &str, cx: i16, cz: i16, tx: i16, tz: i16) -> bool {
        self.cx == cx && self.cz == cz && self.tx == tx && self.tz == tz && self.zone == zone
    }

    /// Whether this entry should appear in teleporter list pages.
    pub fn is_listed(&self) -> bool {
        !self.title.is_empty()
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
        self.to_wire_with_claims(&[])
    }

    pub fn to_wire_with_claims(&self, claims: &[LandClaim]) -> Vec<u8> {
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
        let mut ordered_cells: Vec<_> = cells.iter().collect();
        ordered_cells.sort_by_key(|(cell, _)| **cell);
        for ((cx, cz), items) in ordered_cells {
            p.push(*cx);
            p.push(*cz);
            p.extend_from_slice(&(items.len() as i16).to_le_bytes());
            for item in items {
                p.push(item.rotation);
                p.extend_from_slice(&item.item_data);
            }
        }

        // Claims are supplied from the authoritative world-level registry.
        let now = unix_now();
        p.extend_from_slice(&(claims.len() as i16).to_le_bytes());
        for claim in claims { claim.pack_chunk(&mut p, now); }
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
    pub land_claims: RwLock<HashMap<String, LandClaim>>,
    /// Serialize permission checks, mutations, expiry, and persistence snapshots.
    pub(crate) mutation_lock: Mutex<()>,
    pub players: RwLock<HashMap<String, TrackedPlayer>>,
    pub baskets: BasketStore,
    /// All known zones keyed by name. Pre-populated from the template; extended at runtime
    /// by BUILD (interior items) and plugins (custom zones).
    pub zones: RwLock<HashMap<String, ZoneEntry>>,
    /// All player-built teleporters, in creation order (page N = entries 3N..3N+3).
    pub teleporters: RwLock<Vec<Teleporter>>,
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
            land_claims: RwLock::new(HashMap::new()),
            mutation_lock: Mutex::new(()),
            players: RwLock::new(HashMap::new()),
            baskets: BasketStore::new(),
            zones: RwLock::new(zone_map),
            teleporters: RwLock::new(Vec::new()),
            generator,
        }
    }

    /// Returns the wire-encoded chunk at (zone, x, z), generating one if missing.
    /// Interior zones are never lazily generated; only the default zone supports lazy gen.
    pub fn get_chunk_wire(&self, zone: &str, x: i16, z: i16) -> Vec<u8> {
        let _mutation = self.mutation_lock.lock().unwrap();
        let claims = self.claims_at(zone, x, z, unix_now());
        {
            let chunks = self.chunks.read().unwrap();
            if let Some(m) = chunks.get(zone) {
                if let Some(chunk) = m.get(&(x, z)) {
                    return chunk.to_wire_with_claims(&claims);
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
                ZoneKind::House => return Chunk::blank(x, z, zone).to_wire_with_claims(&claims),
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
                };
            let wire = chunk.to_wire_with_claims(&claims);
            self.chunks.write().unwrap()
                .entry(zone.to_string())
                .or_default()
                .insert((x, z), chunk);
            return wire;
        }

        if !worldgen {
            return Chunk::blank(x, z, zone).to_wire_with_claims(&claims);
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
        };
        let wire = chunk.to_wire_with_claims(&claims);
        self.chunks.write().unwrap()
            .entry(zone.to_string())
            .or_default()
            .insert((x, z), chunk);
        wire
    }

    pub fn upsert_teleporter(&self, zone: &str, cx: i16, cz: i16, tx: i16, tz: i16, title: &str, desc: &str, editor: &str) {
        let mut teles = self.teleporters.write().unwrap();
        if let Some(t) = teles.iter_mut().find(|t| t.is_at(zone, cx, cz, tx, tz)) {
            t.title = title.to_string();
            t.description = desc.to_string();
        } else {
            teles.push(Teleporter {
                title: title.to_string(),
                description: desc.to_string(),
                zone: zone.to_string(),
                cx, cz, tx, tz,
                built_by: editor.to_string(),
                screenshot: Vec::new(),
            });
        }
    }

    /// Stores a screenshot for the teleporter at the given location (C→S 0x30).
    /// Creates an unlisted placeholder if the upload arrives before the 0x33 edit.
    pub fn set_teleporter_screenshot(&self, zone: &str, cx: i16, cz: i16, tx: i16, tz: i16, shot: Vec<u8>, uploader: &str) {
        let mut teles = self.teleporters.write().unwrap();
        if let Some(t) = teles.iter_mut().find(|t| t.is_at(zone, cx, cz, tx, tz)) {
            t.screenshot = shot;
        } else {
            teles.push(Teleporter {
                title: String::new(),
                description: String::new(),
                zone: zone.to_string(),
                cx, cz, tx, tz,
                built_by: uploader.to_string(),
                screenshot: shot,
            });
        }
    }

    /// Removes the teleporter at the given location (its object was removed).
    pub fn remove_teleporter(&self, zone: &str, cx: i16, cz: i16, tx: i16, tz: i16) {
        self.teleporters.write().unwrap().retain(|t| !t.is_at(zone, cx, cz, tx, tz));
    }

}
