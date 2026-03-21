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

use crate::defs::packet::pack_string;

// ── Position / rotation types ─────────────────────────────────────────────

/// Position on the wire: 4 × i16 (8 bytes).
///
/// World coordinates: x = chunk_x * 10 + local_x / 10, z = chunk_z * 10 + local_z / 10.
/// Y is not transmitted (always 0 on the base path).
#[derive(Clone, Copy, Default, Debug)]
pub struct WorldPosition {
    pub chunk_x: i16,
    pub chunk_z: i16,
    pub local_x: i16,
    pub local_z: i16,
}

/// Quaternion rotation scaled ×100 as 4 × i16.
/// Identity = (0, 0, 0, 100).
#[derive(Clone, Copy, Debug)]
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
    // timers omitted for now — add when land claims are needed
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

        // land_claim_count = 0
        p.extend_from_slice(&0i16.to_le_bytes());
        // bandit_camp_count = 0 (read by case 0x0D outer handler via GetShort)
        p.extend_from_slice(&0i16.to_le_bytes());
        p
    }
}

// ── Tracked player state ──────────────────────────────────────────────────

/// Server-side state for a player inside a managed world.
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

// ── World state ───────────────────────────────────────────────────────────

/// Complete state for a single managed world.
pub struct WorldState {
    pub name: String,
    pub default_zone: String,
    pub chunks: RwLock<HashMap<(i16, i16), Chunk>>,
    pub players: RwLock<HashMap<String, TrackedPlayer>>,
}

impl WorldState {
    /// Creates a new world with a grid of blank chunks around the origin.
    pub fn new(name: &str, grid_radius: i16) -> Self {
        let zone = "Main".to_string();
        let mut chunks = HashMap::new();
        for x in -grid_radius..=grid_radius {
            for z in -grid_radius..=grid_radius {
                chunks.insert((x, z), Chunk::blank(x, z, &zone));
            }
        }

        Self {
            name: name.to_string(),
            default_zone: zone,
            chunks: RwLock::new(chunks),
            players: RwLock::new(HashMap::new()),
        }
    }

    /// Returns the wire-encoded chunk at (x, z), generating a blank one if missing.
    pub fn get_chunk_wire(&self, x: i16, z: i16) -> Vec<u8> {
        let chunks = self.chunks.read().unwrap();
        if let Some(chunk) = chunks.get(&(x, z)) {
            chunk.to_wire()
        } else {
            drop(chunks);
            // Auto-generate blank chunk on first access
            let chunk = Chunk::blank(x, z, &self.default_zone);
            let wire = chunk.to_wire();
            self.chunks.write().unwrap().insert((x, z), chunk);
            wire
        }
    }
}
