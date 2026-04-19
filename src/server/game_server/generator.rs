// generator.rs — deterministic world biome/object generator.
//
// Architecture
// ────────────
// The world is divided into 36×36-chunk sectors. Each sector has its own
// independent BiomeMap — a 36×36 grid of biome IDs derived by:
//
//  1. Mapping the chunk's (x, z) within the sector to a blob ID via BLOB_MAP.
//  2. Assigning each of the 36 blob IDs a biome type from a weighted pool
//     (per-sector seeded RNG, deterministic).
//  3. Post-processing: ocean blobs have a 13% chance to become OceanShallow (5);
//     swamp blobs have a 50% chance to become SwampDark (7).
//
// RE source: ChunkControl$GenerateNewBiomeMap (0x9837f8),
//            ChunkControl$GenBlobBiometype   (0x98339c),
//            ChunkControl$GetBiomeMapCoordinates (0x983e20),
//            BanditCampsControl$PopulateOverworldChunk (0x95dc7c).
//
// Object spawning
// ───────────────
// After biome assignment, generate_chunk_elements() places 3–8 natural objects
// per chunk using a per-chunk seeded RNG and per-biome weighted spawn tables.
// Objects are InventoryItem packets with a single `item_id` string property.
// Multi-tile objects (size 2, 3, 5) are placed at their top-left tile; the
// server tracks occupied tiles to prevent overlap.
//
// Biome IDs
// ─────────
//   0 = Grass        4 = Ocean          8 = Woodlands
//   1 = Snow         5 = OceanShallow   9 = Sakura
//   2 = Desert       6 = Swamp
//   3 = Evergreen    7 = SwampDark
//
// Floor properties (per-chunk random, seeded by seed ^ chunk_x ^ chunk_z):
//   floor_rotation ∈ [0, 3]
//   floor_texture  ∈ [0, BIOME_TEXTURE_COUNTS[biome] - 1]
//
// BLOB_MAP
// ────────
// Extracted from assets/biome-map.png (36×36 px).
// Each unique colour in the PNG, encountered in scan order, gets the next
// sequential blob ID (0–35). The resulting 36×36 array maps
// (local_z_in_sector, local_x_in_sector) → blob_id.

// ── Biome IDs ─────────────────────────────────────────────────────────────

pub const BIOME_GRASS:          u8 = 0;
pub const BIOME_SNOW:           u8 = 1;
pub const BIOME_DESERT:         u8 = 2;
pub const BIOME_EVERGREEN:      u8 = 3;
pub const BIOME_OCEAN:          u8 = 4;
pub const BIOME_OCEAN_SHALLOW:  u8 = 5;
pub const BIOME_SWAMP:          u8 = 6;
pub const BIOME_SWAMP_DARK:     u8 = 7;
pub const BIOME_WOODLANDS:      u8 = 8;
pub const BIOME_SAKURA:         u8 = 9;

/// Number of floor textures available per biome.
/// Exact counts are Unity-serialized and not accessible from the binary;
/// these are conservative defaults (4 for most, 2 for rarer biomes).
/// Adjust after in-game testing if textures wrap or show incorrect tiles.
pub const BIOME_TEXTURE_COUNTS: [u8; 10] = [
    4, // Grass
    3, // Snow
    3, // Desert
    3, // Evergreen
    2, // Ocean
    2, // OceanShallow
    2, // Swamp
    2, // SwampDark
    3, // Woodlands
    3, // Sakura
];

/// Mob pair strings for each biome (mobA, mobB).
/// These are used by client-side `ChunkControl` to spawn ambient mobs.
/// Empty strings = no mobs.
pub const BIOME_MOBS: [(&str, &str); 10] = [
    ("",  ""),  // Grass
    ("",  ""),  // Snow
    ("",  ""),  // Desert
    ("",  ""),  // Evergreen
    ("",  ""),  // Ocean
    ("",  ""),  // OceanShallow
    ("",  ""),  // Swamp
    ("",  ""),  // SwampDark
    ("",  ""),  // Woodlands
    ("",  ""),  // Sakura
];

// ── Placed object (chunk element) ────────────────────────────────────────

/// A single object to be placed in a chunk cell.
/// `item_data` is the full InventoryItem wire encoding (see `pack_item`).
pub struct PlacedObject {
    pub cell_x:    u8,
    pub cell_z:    u8,
    pub rotation:  u8,
    pub item_data: Vec<u8>,
}

// ── Object spawn tables ───────────────────────────────────────────────────
//
// (item_name, weight, tile_size)
//   weight:    relative spawn probability (higher = more common)
//   tile_size: footprint — 1=1×1, 2=2×2, 3=3×3, 5=5×5
//
// OceanShallow (5) reuses Ocean (4). SwampDark (7) reuses Swamp (6).
// RE'd from the Python reference implementation (segual/game_server.py).

static OBJECTS_GRASS: &[(&str, u32, u8)] = &[
    ("Birch Tree (Variant 1)", 8, 2), ("Birch Tree (Variant 2)", 6, 2),
    ("Mossy Tree",             4, 2), ("Willow Tree",            2, 2),
    ("Stone Vein",             3, 2), ("Metal Vein",             1, 1),
    ("Spawner - Sticks",       5, 1), ("Spawner - Bones",        2, 1),
    ("Spawner - Nuts",         3, 1), ("Berry Bush",             4, 1),
    ("Flowers",                6, 1), ("Cotton Plant",           2, 1),
    ("Creature Nest",          1, 1), ("Beehive",                1, 1),
    ("Lavender Bush",          2, 1), ("Tea Tree",               2, 1),
];

static OBJECTS_SNOW: &[(&str, u32, u8)] = &[
    ("Frozen Tree",            8, 2), ("Evergreen Tree",         6, 2),
    ("Stone Vein (Snowy)",     4, 2), ("Silver Vein",            2, 1),
    ("Ice Shard Vein",         2, 1), ("Spawner - Snowballs",    5, 1),
    ("Spawner - Sticks",       3, 1), ("Spawner - Bones",        2, 1),
    ("Creature Nest",          1, 1),
];

static OBJECTS_DESERT: &[(&str, u32, u8)] = &[
    ("Palm Tree",              8, 2), ("Cactus",                 6, 1),
    ("Stone Vein (Desert)",    4, 2), ("Gold Vein",              2, 1),
    ("Spawner - Sticks",       3, 1), ("Spawner - Bones",        3, 1),
    ("Spawner - Coconuts",     4, 1), ("Spawner - Fossils",      2, 1),
    ("Hot Pepper Plant",       2, 1), ("Creature Nest",          1, 1),
];

static OBJECTS_EVERGREEN: &[(&str, u32, u8)] = &[
    ("Evergreen Tree",        10, 2), ("Large Mossy Tree",       3, 2),
    ("Mossy Tree",             4, 2), ("Stone Vein",             3, 2),
    ("Metal Vein",             2, 1), ("Titanium Vein",          1, 1),
    ("Spawner - Sticks",       5, 1), ("Spawner - Nuts",         3, 1),
    ("Spawner - Bones",        2, 1), ("Berry Bush",             3, 1),
    ("Giant Brown Mushroom",   2, 1), ("Giant Red Mushroom",     1, 1),
    ("Beehive",                1, 1), ("Creature Nest",          1, 1),
    ("Spiderhive",             1, 1),
];

static OBJECTS_OCEAN: &[(&str, u32, u8)] = &[
    ("Stone Vein (Ocean)",     4, 2),
    ("Spawner - Red Shells",   3, 1), ("Spawner - Blue Shells",  3, 1),
    ("Spawner - Green Shells", 3, 1), ("Spawner - Purple Shells",2, 1),
    ("Spawner - White Shells", 2, 1), ("Spawner - Black Shells", 1, 1),
    ("Spawner - Gold Shells",  1, 1), ("Spawner - Bones",        2, 1),
];

static OBJECTS_SWAMP: &[(&str, u32, u8)] = &[
    ("Willow Tree",            6, 2), ("Mossy Tree",             5, 2),
    ("Rotting Stump",          4, 1), ("Stone Vein",             3, 2),
    ("Spawner - Sticks",       4, 1), ("Spawner - Bones",        3, 1),
    ("Spawner - Spirit Branch",2, 1), ("Giant Purple Mushroom",  3, 1),
    ("Giant Brown Mushroom",   2, 1), ("Lavender Bush",          2, 1),
    ("Spiderhive",             2, 1), ("Creature Nest",          1, 1),
    ("Lava",                   1, 1),
];

static OBJECTS_WOODLANDS: &[(&str, u32, u8)] = &[
    ("Birch Tree (Variant 1)", 7, 2), ("Birch Tree (Variant 2)", 5, 2),
    ("Mossy Tree",             5, 2), ("Large Mossy Tree",       2, 2),
    ("Evergreen Tree",         3, 2), ("Stone Vein",             3, 2),
    ("Metal Vein",             2, 1), ("Emerald Vein",           1, 1),
    ("Spawner - Sticks",       5, 1), ("Spawner - Nuts",         4, 1),
    ("Berry Bush",             3, 1), ("Goldberry Bush",         1, 1),
    ("Flowers",                3, 1), ("Cotton Plant",           2, 1),
    ("Beehive",                2, 1), ("Giant Brown Mushroom",   2, 1),
    ("Creature Nest",          1, 1),
];

static OBJECTS_SAKURA: &[(&str, u32, u8)] = &[
    ("Sakura Tree",           10, 2), ("Birch Tree (Variant 1)", 3, 2),
    ("Stone Vein (Sakura)",    3, 2), ("Titanium Vein (Sakura)", 1, 1),
    ("Spawner - Sticks",       4, 1), ("Spawner - Nuts",         3, 1),
    ("Berry Bush",             3, 1), ("MoonBerry Bush",         2, 1),
    ("Salmonberry Bush",       2, 1), ("Flowers",                5, 1),
    ("Lavender Bush",          3, 1), ("Tea Tree",               3, 1),
    ("Beehive",                1, 1), ("Creature Nest",          1, 1),
];

fn biome_object_table(biome: i16) -> &'static [(&'static str, u32, u8)] {
    match biome as u8 {
        BIOME_GRASS                           => OBJECTS_GRASS,
        BIOME_SNOW                            => OBJECTS_SNOW,
        BIOME_DESERT                          => OBJECTS_DESERT,
        BIOME_EVERGREEN                       => OBJECTS_EVERGREEN,
        BIOME_OCEAN | BIOME_OCEAN_SHALLOW     => OBJECTS_OCEAN,
        BIOME_SWAMP | BIOME_SWAMP_DARK        => OBJECTS_SWAMP,
        BIOME_WOODLANDS                       => OBJECTS_WOODLANDS,
        BIOME_SAKURA                          => OBJECTS_SAKURA,
        _                                     => OBJECTS_GRASS,
    }
}

/// Encodes an item name into the InventoryItem wire format.
/// Format: u16(0 shorts) | u16(1 string) | str("item_id") | str(name) | u16(0 ints)
fn pack_item(name: &str) -> Vec<u8> {
    let mut p = Vec::new();
    p.extend_from_slice(&0u16.to_le_bytes()); // 0 short props
    p.extend_from_slice(&1u16.to_le_bytes()); // 1 string prop
    p.extend(pack_string("item_id"));
    p.extend(pack_string(name));
    p.extend_from_slice(&0u16.to_le_bytes()); // 0 int props
    p
}

// ── Blob map ──────────────────────────────────────────────────────────────
//
// BLOB_MAP[z][x] = blob_id for local coords within a 36×36 sector.
// Extracted from assets/biome-map.png (scan-order colour assignment).

pub const BLOB_MAP: [[u8; 36]; 36] = [
    [ 0,  0,  0,  0,  0,  1,  1,  1,  1,  1,  1,  1,  1,  2,  2,  2,  2,  3,  3,  3,  3,  3,  3,  4,  4,  4,  4,  4,  4,  4,  4,  5,  5,  5,  5,  5],
    [ 0,  0,  0,  0,  0,  1,  1,  1,  1,  1,  1,  1,  1,  2,  2,  2,  2,  3,  3,  3,  3,  3,  3,  4,  4,  4,  4,  4,  4,  4,  4,  5,  5,  5,  5,  5],
    [ 0,  0,  0,  0,  0,  0,  1,  1,  1,  1,  1,  1,  2,  2,  2,  2,  2,  2,  3,  3,  3,  3,  3,  3,  4,  4,  4,  4,  4,  4,  4,  5,  5,  5,  5,  5],
    [ 0,  0,  0,  0,  0,  0,  1,  1,  1,  1,  1,  1,  2,  2,  2,  2,  2,  2,  3,  3,  3,  3,  3,  3,  4,  4,  4,  4,  4,  4,  5,  5,  5,  5,  5,  5],
    [ 0,  0,  0,  0,  0,  0,  0,  1,  1,  1,  1,  1,  2,  2,  2,  2,  2,  2,  3,  3,  3,  3,  3,  3,  4,  4,  4,  4,  4,  4,  5,  5,  5,  5,  5,  5],
    [ 0,  0,  0,  0,  0,  0,  0,  1,  1,  1,  1,  1,  2,  2,  2,  2,  6,  6,  3,  3,  3,  3,  3,  3,  4,  4,  4,  4,  7,  7,  5,  5,  5,  5,  5,  5],
    [ 8,  8,  0,  0,  0,  0,  0,  1,  1,  1,  9,  9,  2,  2,  2,  6,  6,  6,  3,  3, 10, 10, 10, 10,  4,  4,  7,  7,  7,  7, 11, 11,  5,  5,  5,  5],
    [ 8,  8,  8,  8,  8,  9,  9,  9,  9,  9,  9,  9,  6,  6,  6,  6,  6,  6, 10, 10, 10, 10, 10, 10,  7,  7,  7,  7,  7,  7, 11, 11, 11, 11, 11, 11],
    [ 8,  8,  8,  8,  8,  9,  9,  9,  9,  9,  9,  9,  6,  6,  6,  6,  6,  6, 10, 10, 10, 10, 10, 10,  7,  7,  7,  7,  7,  7, 11, 11, 11, 11, 11, 11],
    [ 8,  8,  8,  8,  8,  9,  9,  9,  9,  9,  9,  9,  6,  6,  6,  6,  6,  6, 10, 10, 10, 10, 10, 10,  7,  7,  7,  7,  7,  7, 11, 11, 11, 11, 11, 11],
    [ 8,  8,  8,  8,  9,  9,  9,  9,  9,  9,  9,  9,  6,  6,  6,  6,  6,  6, 10, 10, 10, 10, 10, 10,  7,  7,  7,  7,  7,  7, 11, 11, 11, 11, 11, 11],
    [ 8,  8,  8,  8,  9,  9,  9,  9,  9,  9,  9, 12,  6,  6,  6,  6,  6, 13, 10, 10, 10, 10, 10, 14,  7,  7,  7,  7,  7, 15, 11, 11, 11, 11, 11, 16],
    [17, 17, 17, 18, 18, 18, 18, 18, 18, 18, 12, 12, 13, 13, 13, 13, 13, 13, 14, 14, 14, 14, 14, 14, 15, 15, 15, 15, 15, 15, 16, 16, 16, 16, 16, 16],
    [17, 17, 17, 18, 18, 18, 18, 18, 18, 18, 12, 12, 13, 13, 13, 13, 13, 13, 14, 14, 14, 14, 14, 14, 15, 15, 15, 15, 15, 15, 16, 16, 16, 16, 16, 16],
    [17, 17, 17, 18, 18, 18, 18, 18, 18, 18, 12, 12, 13, 13, 13, 13, 13, 13, 14, 14, 14, 14, 14, 14, 15, 15, 15, 15, 15, 15, 16, 16, 16, 16, 16, 16],
    [17, 17, 17, 18, 18, 18, 18, 18, 18, 18, 12, 12, 13, 13, 13, 13, 13, 13, 14, 14, 14, 14, 14, 14, 15, 15, 15, 15, 15, 15, 16, 16, 16, 16, 16, 16],
    [17, 17, 17, 18, 18, 18, 18, 18, 18, 12, 12, 12, 13, 13, 13, 13, 13, 19, 14, 14, 14, 14, 14, 20, 15, 15, 15, 15, 15, 21, 16, 16, 16, 16, 16, 22],
    [17, 17, 23, 23, 23, 23, 18, 18, 12, 12, 12, 12, 19, 19, 19, 13, 13, 19, 20, 20, 20, 14, 14, 20, 21, 21, 21, 15, 15, 21, 22, 22, 22, 16, 16, 22],
    [23, 23, 23, 23, 23, 23, 23, 12, 12, 12, 12, 12, 19, 19, 19, 19, 19, 19, 20, 20, 20, 20, 20, 20, 21, 21, 21, 21, 21, 21, 22, 22, 22, 22, 22, 22],
    [23, 23, 23, 23, 23, 23, 23, 12, 12, 12, 12, 12, 19, 19, 19, 19, 19, 19, 20, 20, 20, 20, 20, 20, 21, 21, 21, 21, 21, 21, 22, 22, 22, 22, 22, 22],
    [23, 23, 23, 23, 23, 23, 23, 12, 12, 12, 12, 12, 19, 19, 19, 19, 19, 19, 20, 20, 20, 20, 20, 20, 21, 21, 21, 21, 21, 21, 22, 22, 22, 22, 22, 22],
    [23, 23, 23, 23, 23, 23, 23, 24, 24, 24, 24, 24, 19, 19, 19, 19, 19, 25, 20, 20, 20, 20, 20, 26, 21, 21, 21, 21, 21, 27, 22, 22, 22, 22, 22, 28],
    [23, 23, 23, 23, 23, 24, 24, 24, 24, 24, 24, 24, 25, 25, 25, 25, 25, 25, 26, 26, 26, 26, 26, 26, 27, 27, 27, 27, 27, 27, 28, 28, 28, 28, 28, 28],
    [23, 23, 23, 23, 23, 24, 24, 24, 24, 24, 24, 24, 25, 25, 25, 25, 25, 25, 26, 26, 26, 26, 26, 26, 27, 27, 27, 27, 27, 27, 28, 28, 28, 28, 28, 28],
    [23, 23, 23, 23, 23, 24, 24, 24, 24, 24, 24, 24, 25, 25, 25, 25, 25, 25, 26, 26, 26, 26, 26, 26, 27, 27, 27, 27, 27, 27, 28, 28, 28, 28, 28, 28],
    [23, 23, 23, 23, 24, 24, 24, 24, 24, 24, 24, 24, 25, 25, 25, 25, 25, 25, 26, 26, 26, 26, 26, 26, 27, 27, 27, 27, 27, 27, 28, 28, 28, 28, 28, 28],
    [29, 29, 29, 29, 24, 24, 24, 24, 24, 24, 24, 29, 25, 25, 25, 25, 25, 29, 26, 26, 26, 26, 26, 29, 27, 27, 27, 27, 27, 29, 28, 28, 28, 28, 28, 29],
    [29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29],
    [29, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30],
    [30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30],
    [30, 30, 30, 30, 30, 31, 31, 31, 31, 31, 31, 31, 32, 32, 32, 32, 33, 33, 33, 33, 33, 33, 33, 33, 33, 30, 30, 30, 30, 30, 35, 35, 35, 35, 35, 35],
    [31, 31, 31, 31, 31, 31, 31, 31, 32, 32, 32, 32, 32, 32, 33, 33, 33, 33, 33, 33, 34, 34, 34, 34, 34, 34, 34, 34, 34, 35, 35, 35, 35, 35, 35, 35],
    [31, 31, 31, 31, 31, 31, 31, 31, 32, 32, 32, 32, 32, 32, 33, 33, 33, 33, 33, 33, 34, 34, 34, 34, 34, 34, 34, 34, 34, 35, 35, 35, 35, 35, 35, 35],
    [31, 31, 31, 31, 31, 31, 31, 32, 32, 32, 32, 32, 32, 33, 33, 33, 33, 33, 33, 34, 34, 34, 34, 34, 34, 34, 34, 34, 34, 35, 35, 35, 35, 35, 35, 35],
    [31, 31, 31, 31, 31, 31, 31, 32, 32, 32, 32, 32, 32, 33, 33, 33, 33, 33, 33, 34, 34, 34, 34, 34, 34, 34, 34, 34, 34, 35, 35, 35, 35, 35, 35, 35],
    [31, 31, 31, 31, 31, 32, 32, 32, 32, 32, 32, 32, 33, 33, 33, 33, 34, 34, 34, 34, 34, 34, 34, 34, 34, 30, 30, 30, 30, 30, 35, 35, 35, 35, 35, 35],
];

// ── BiomeWeights ──────────────────────────────────────────────────────────
//
// Controls how many of the 36 blobs get each biome type.
// Total must equal 36 for a fully-covered sector; excess blobs beyond the
// pool wrap back to grass.
//
// Default matches game call: GenerateNewBiomeMap(..., 2, 1, 1, 1, 2, 1, 1, 1)
//   grass=2, snow=1, desert=1, evergreen=1, ocean=2, swamp=1, woodlands=1, sakura=1

#[derive(Clone, Debug)]
pub struct BiomeWeights {
    pub grass:     u8,
    pub snow:      u8,
    pub desert:    u8,
    pub evergreen: u8,
    pub ocean:     u8,
    pub swamp:     u8,
    pub woodlands: u8,
    pub sakura:    u8,
}

impl Default for BiomeWeights {
    fn default() -> Self {
        Self {
            grass:     2,
            snow:      1,
            desert:    1,
            evergreen: 1,
            ocean:     2,
            swamp:     1,
            woodlands: 1,
            sakura:    1,
        }
    }
}

impl BiomeWeights {
    /// Expands the weights into an ordered biome-assignment pool.
    /// The pool is shuffled per-sector using the seeded RNG.
    fn to_pool(&self) -> Vec<u8> {
        let mut pool = Vec::with_capacity(36);
        for _ in 0..self.grass     { pool.push(BIOME_GRASS);     }
        for _ in 0..self.snow      { pool.push(BIOME_SNOW);      }
        for _ in 0..self.desert    { pool.push(BIOME_DESERT);    }
        for _ in 0..self.evergreen { pool.push(BIOME_EVERGREEN); }
        for _ in 0..self.ocean     { pool.push(BIOME_OCEAN);     }
        for _ in 0..self.swamp     { pool.push(BIOME_SWAMP);     }
        for _ in 0..self.woodlands { pool.push(BIOME_WOODLANDS); }
        for _ in 0..self.sakura    { pool.push(BIOME_SAKURA);    }
        // Pad to 36 with grass if weights sum to less than 36
        while pool.len() < 36 { pool.push(BIOME_GRASS); }
        pool.truncate(36);
        pool
    }
}

// ── ZoneConfig ────────────────────────────────────────────────────────────

/// Per-zone biome configuration.
#[derive(Clone, Debug)]
pub struct ZoneConfig {
    pub name:    String,
    pub weights: BiomeWeights,
}

impl ZoneConfig {
    pub fn new(name: impl Into<String>, weights: BiomeWeights) -> Self {
        Self { name: name.into(), weights }
    }

    pub fn default_main() -> Self {
        Self::new("overworld", BiomeWeights::default())
    }
}

// ── WorldTemplate ─────────────────────────────────────────────────────────

/// Top-level world generation configuration.
#[derive(Clone, Debug)]
pub struct WorldTemplate {
    pub seed:  u64,
    pub zones: Vec<ZoneConfig>,
}

impl Default for WorldTemplate {
    fn default() -> Self {
        Self {
            seed:  0,
            zones: vec![ZoneConfig::default_main()],
        }
    }
}

impl WorldTemplate {
    pub fn new(seed: u64, zones: Vec<ZoneConfig>) -> Self {
        Self { seed, zones }
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
    pub biome:     i16,
    pub floor_rot: i16,
    pub floor_tex: i16,
    pub mob_a:     String,
    pub mob_b:     String,
    pub elements:  Vec<PlacedObject>,
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

use crate::defs::packet::pack_string;
use std::collections::HashMap;
use std::sync::RwLock;

/// Generates and caches sector BiomeMaps for a world.
///
/// Each sector (sector_x, sector_z) independently assigns biome types to the
/// 36 blob IDs using the zone's BiomeWeights and a seeded shuffle.
pub struct WorldGenerator {
    template: WorldTemplate,
    /// Cache: (zone_name, sector_x, sector_z) → [biome_id; 36]
    sector_cache: RwLock<HashMap<(String, i32, i32), [u8; 36]>>,
}

impl WorldGenerator {
    pub fn new(template: WorldTemplate) -> Self {
        Self {
            template,
            sector_cache: RwLock::new(HashMap::new()),
        }
    }

    /// Returns an iterator over the zone names defined in the template.
    pub fn template_zones(&self) -> impl Iterator<Item = &str> {
        self.template.zones.iter().map(|z| z.name.as_str())
    }

    pub fn template(&self) -> &WorldTemplate {
        &self.template
    }

    /// Returns the chunk's sector coordinates.
    /// sector = (floor(chunk_x / 36), floor(chunk_z / 36))
    fn sector_of(chunk_x: i32, chunk_z: i32) -> (i32, i32) {
        let sx = chunk_x.div_euclid(36);
        let sz = chunk_z.div_euclid(36);
        (sx, sz)
    }

    /// Local position within a sector (0–35 each).
    fn local_in_sector(chunk_x: i32, chunk_z: i32) -> (usize, usize) {
        let lx = chunk_x.rem_euclid(36) as usize;
        let lz = chunk_z.rem_euclid(36) as usize;
        (lx, lz)
    }

    /// Returns or generates the blob→biome mapping for a sector.
    fn sector_biomes(&self, zone_name: &str, sector_x: i32, sector_z: i32) -> [u8; 36] {
        let key = (zone_name.to_string(), sector_x, sector_z);
        {
            let cache = self.sector_cache.read().unwrap();
            if let Some(map) = cache.get(&key) {
                return *map;
            }
        }

        let map = self.generate_sector(zone_name, sector_x, sector_z);
        self.sector_cache.write().unwrap().insert(key, map);
        map
    }

    /// Generates a blob→biome assignment for one sector.
    ///
    /// RE: ChunkControl$GenerateNewBiomeMap (0x9837f8)
    ///  1. Build a pool of biome type entries from weights.
    ///  2. Fisher-Yates shuffle using seeded RNG.
    ///  3. Assign pool[blob_id % pool.len()] to each blob.
    ///  4. Post-process ocean→shallow (13%), swamp→dark (50%).
    fn generate_sector(&self, zone_name: &str, sector_x: i32, sector_z: i32) -> [u8; 36] {
        let weights = self.template.zone_weights(zone_name);
        let mut pool = weights.to_pool();

        // Sector seed: mix world seed with sector coords
        let sector_salt = (sector_x as u64)
            .wrapping_mul(0x517cc1b727220a95)
            .wrapping_add((sector_z as u64).wrapping_mul(0x6c62272e07bb0142));
        let base_seed = splitmix64(self.template.seed ^ sector_salt);

        // Fisher-Yates shuffle of the pool
        for i in (1..pool.len()).rev() {
            let j = rng_u32(base_seed, i as u64) as usize % (i + 1);
            pool.swap(i, j);
        }

        // Assign biomes to blobs
        let mut blob_biomes = [BIOME_GRASS; 36];
        for blob_id in 0..36usize {
            blob_biomes[blob_id] = pool[blob_id];
        }

        // Post-processing — RE: ChunkControl$GenerateNewBiomeMap
        //   ocean blobs: 13% chance → OceanShallow
        //   swamp blobs: 50% chance → SwampDark
        for blob_id in 0..36usize {
            let post_salt = (blob_id as u64).wrapping_add(0x8000_0000);
            let roll = rng_u32(base_seed, post_salt) % 100;
            match blob_biomes[blob_id] {
                BIOME_OCEAN  if roll < 13 => { blob_biomes[blob_id] = BIOME_OCEAN_SHALLOW; }
                BIOME_SWAMP  if roll < 50 => { blob_biomes[blob_id] = BIOME_SWAMP_DARK; }
                _ => {}
            }
        }

        blob_biomes
    }

    /// Returns biome parameters for a chunk at (chunk_x, chunk_z) in zone.
    pub fn chunk_params(&self, zone_name: &str, chunk_x: i32, chunk_z: i32) -> ChunkBiomeParams {
        let (sx, sz) = Self::sector_of(chunk_x, chunk_z);
        let (lx, lz) = Self::local_in_sector(chunk_x, chunk_z);

        let blob_biomes = self.sector_biomes(zone_name, sx, sz);
        let blob_id     = BLOB_MAP[lz][lx] as usize;
        let biome       = blob_biomes[blob_id] as i16;

        // Per-chunk floor properties: seeded by world seed ^ chunk coords
        let chunk_salt = (chunk_x as u64)
            .wrapping_mul(0x9e3779b97f4a7c15)
            .wrapping_add((chunk_z as u64).wrapping_mul(0x6c62272e07bb0142));
        let chunk_seed = splitmix64(self.template.seed ^ chunk_salt);

        let tex_count = BIOME_TEXTURE_COUNTS[biome as usize] as u64;
        let floor_tex = (rng_u32(chunk_seed, 0x01) as u64 % tex_count) as i16;
        let floor_rot = (rng_u32(chunk_seed, 0x02) % 4) as i16;

        let (mob_a, mob_b) = BIOME_MOBS[biome as usize];
        let elements = self.generate_chunk_elements(chunk_x, chunk_z, biome);

        ChunkBiomeParams {
            biome,
            floor_rot,
            floor_tex,
            mob_a: mob_a.to_string(),
            mob_b: mob_b.to_string(),
            elements,
        }
    }

    /// Generates natural objects for a chunk.
    ///
    /// Uses a per-chunk seeded RNG (independent from biome/floor RNG).
    /// Picks 3–8 objects from the biome's weighted table with overlap prevention.
    /// Multi-tile objects (size 2, 3, 5) occupy all covered cells in the
    /// occupancy bitset but are stored as a single element at their top-left tile.
    fn generate_chunk_elements(&self, chunk_x: i32, chunk_z: i32, biome: i16) -> Vec<PlacedObject> {
        let table = biome_object_table(biome);
        if table.is_empty() {
            return Vec::new();
        }

        // Separate RNG stream from floor properties (uses 0x01/0x02 as salts).
        // Salt 0x8000_0000+ is safely out of range of the floor salt space.
        let chunk_salt = (chunk_x as u64)
            .wrapping_mul(0x9e3779b97f4a7c15)
            .wrapping_add((chunk_z as u64).wrapping_mul(0x6c62272e07bb0142));
        let obj_seed = splitmix64(self.template.seed ^ chunk_salt ^ 0x0101_0101_0101_0101);
        let mut ctr: u64 = 0;
        let mut rng = |modulus: u32| -> u32 {
            ctr += 1;
            rng_u32(obj_seed, ctr) % modulus
        };

        // Total weight for weighted pick
        let total_weight: u32 = table.iter().map(|e| e.1).sum();

        // num_objects in [3, 8]
        let num_objects = 3 + rng(6) as usize; // 0..5 → +3 → 3..8

        // Occupied-tile bitset: bit (z * 10 + x) for a 10×10 chunk grid
        let mut occupied: u128 = 0;
        let mut elements = Vec::with_capacity(num_objects);

        'outer: for _ in 0..num_objects {
            // Weighted pick
            let mut roll = rng(total_weight);
            let mut chosen = &table[table.len() - 1];
            for entry in table {
                if roll < entry.1 {
                    chosen = entry;
                    break;
                }
                roll -= entry.1;
            }
            let (name, _, size) = *chosen;
            let size = size as u32;
            let rotation = rng(4) as u8;

            // Up to 10 placement attempts
            for _ in 0..10 {
                let max_pos = 10u32.saturating_sub(size);
                let tx = rng(max_pos + 1);
                let tz = rng(max_pos + 1);

                // Check all tiles the object occupies
                let mut mask: u128 = 0;
                for dz in 0..size {
                    for dx in 0..size {
                        let bit = (tz + dz) * 10 + (tx + dx);
                        mask |= 1u128 << bit;
                    }
                }

                if occupied & mask == 0 {
                    occupied |= mask;
                    elements.push(PlacedObject {
                        cell_x:    tx as u8,
                        cell_z:    tz as u8,
                        rotation,
                        item_data: pack_item(name),
                    });
                    continue 'outer;
                }
            }
        }

        elements
    }
}
