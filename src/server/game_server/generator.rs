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
//   Common:      weight 20
//   Rare:        weight  5
//   Really rare: weight  2
//   Super rare:  weight  1
//
// OceanShallow (5) and SwampDark (7) have dedicated tables.

static OBJECTS_GRASS: &[(&str, u32, u8)] = &[
    // Common
    ("Metal Vein",            12, 2), ("Green Blob",            20, 1),
    ("Spawner - Sticks",      20, 1), ("Stone Vein",            12, 2),
    // Rare
    ("Cotton Plant",           2, 1), ("Large Stone Vein",       3, 2),
    ("Large Metal Vein",       3, 2), ("Spawner - Nuts",         5, 1),
    ("Red Blob",               5, 1), ("Pond",                   5, 3),
    ("Mossy Tree",             5, 2), ("Ancient Pillars",        5, 2),
    // Really rare
    ("Large Emerald Vein",     1, 2), ("Emerald Vein",           1, 2),
    ("Blue Blob",              2, 1), ("Beehive",                2, 1),
];

static OBJECTS_SNOW: &[(&str, u32, u8)] = &[
    // Common
    ("Green Blob",            20, 1), ("Stone Vein (Snowy)",    12, 2),
    // Rare
    ("Red Blob",               5, 1), ("Frozen Pond",            5, 3),
    ("Snowman",                5, 1), ("Frozen Tree",            5, 2),
    ("Titanium Vein",          3, 2),
    // Really rare
    ("Blue Blob",              2, 1), ("MoonBerry Bush",         2, 1),
];

static OBJECTS_DESERT: &[(&str, u32, u8)] = &[
    // Common (blobs deliberately less common)
    ("Stone Vein (Desert)",   12, 2), ("Cactus",                20, 1),
    ("Green Blob",             8, 1),
    // Rare
    ("Uranium Vein",           3, 2), ("Gold Vein",              3, 2),
    ("Red Blob",               3, 1),
    // Really rare
    ("Blue Blob",              2, 1), ("Spawner - Sticks",       2, 1),
];

static OBJECTS_EVERGREEN: &[(&str, u32, u8)] = &[
    // Common
    ("Evergreen Tree",        20, 2), ("Tar Pit",                4, 2),
    ("Dug-up Brown Mushroom", 20, 1), ("Salmonberry Bush",      20, 1),
    ("Stone Vein",            12, 2), ("Spawner - Sticks",      20, 1),
    ("Green Blob",            20, 1),
    // Rare
    ("Large Stone Vein",       3, 2), ("Dug-Up Red Mushroom",   5, 1),
    ("Gold Vein",              3, 2),
    // Really rare
    ("Blue Blob",              2, 1), ("Metal Vein",             2, 2),
    ("Giant Red Mushroom",     2, 1),
    // Super rare
    ("Large Ruby Vein",        1, 2), ("Ruby Vein",              1, 2),
];

// Spirit Tree is here; Spirit Branch is placed as a cluster off Spirit Tree.
static OBJECTS_SAKURA: &[(&str, u32, u8)] = &[
    // Common
    ("Green Blob",            20, 1), ("Stone Vein (Sakura)",   12, 2),
    ("Sakura Tree",           20, 2),
    // Rare
    ("Red Blob",               5, 1), ("Flowers",                5, 1),
    ("Sakura Pond",            5, 3), ("Stone Lantern",          5, 1),
    // Really rare
    ("Amethyst Vein",          2, 2), ("Lavender Bush",          2, 1),
    ("Blue Blob",              2, 1), ("Spawner - Sticks",       2, 1),
    ("Titanium Vein (Sakura)", 2, 2),
    // Super rare
    ("Spirit Tree",            1, 3),
];

// Ocean: shells are extremely rare — hard to find even 3 in a whole ocean.
// Stone Vein dominates so shell rolls almost always lose.
static OBJECTS_OCEAN: &[(&str, u32, u8)] = &[
    // Common (relative to the rest of the table)
    ("Stone Vein (Ocean)",    80, 2),
    // Super rare — each shell ~0.18% per roll
    ("Spawner - Blue Shells",  1, 1), ("Spawner - White Shells", 1, 1),
    ("Spawner - Green Shells", 1, 1), ("Spawner - Purple Shells",1, 1),
    ("Spawner - Black Shells", 1, 1), ("Spawner - Red Shells",   1, 1),
    ("Spawner - Gold Shells",  1, 1),
];

// OceanShallow = beachside: Palm Trees only.
static OBJECTS_OCEAN_SHALLOW: &[(&str, u32, u8)] = &[
    ("Palm Tree",              1, 2),
];

// Swamp: more spaced out than other biomes; cluster logic uses small
// cluster counts (1–2) and rarely places a large-vein variant.
static OBJECTS_SWAMP: &[(&str, u32, u8)] = &[
    // Common
    ("Willow Tree",           20, 2), ("Green Blob",            20, 1),
    ("Dug-up Brown Mushroom", 20, 1), ("Stone Vein",            20, 2),
    ("Rotting Stump",         20, 1),
    // Rare
    ("Giant Purple Mushroom",  5, 1), ("Red Blob",               5, 1),
    ("Metal Vein",             5, 2), ("Large Metal Vein",       5, 2),
    // Really rare
    ("Blue Blob",              2, 1),
];

// SwampDark = swamp lake: only a very rare Stone Vein (70% chance of nothing).
static OBJECTS_SWAMP_DARK: &[(&str, u32, u8)] = &[
    ("Stone Vein",             1, 2),
];

// Woodlands: wheat/mushroom clusters are the signature; pumpkins and giant
// pumpkins are lone finds. Veins cluster, with the large variant being the
// rarer seed of a cluster.
static OBJECTS_WOODLANDS: &[(&str, u32, u8)] = &[
    // Common
    ("Birch Tree (Variant 1)",20, 2), ("Birch Tree (Variant 2)",20, 2),
    ("Dug-Up Wheat",          20, 1), ("Green Blob",            20, 1),
    // Rare
    ("Blue Blob",              5, 1), ("Red Blob",               5, 1),
    ("Metal Vein",             5, 2), ("Large Metal Vein",       5, 2),
    ("Stone Vein (White)",     5, 2),
    // Super rare
    ("Silver Vein",            2, 2), ("Dug-up Brown Mushroom",  2, 1),
    ("Dug-up Pumpkin",         2, 1),
    // Really really rare
    ("Spawner - Nuts",         1, 1), ("Dug-Up Giant Pumpkin",   1, 2),
];

fn biome_object_table(biome: i16) -> &'static [(&'static str, u32, u8)] {
    match biome as u8 {
        BIOME_GRASS         => OBJECTS_GRASS,
        BIOME_SNOW          => OBJECTS_SNOW,
        BIOME_DESERT        => OBJECTS_DESERT,
        BIOME_EVERGREEN     => OBJECTS_EVERGREEN,
        BIOME_OCEAN         => OBJECTS_OCEAN,
        BIOME_OCEAN_SHALLOW => OBJECTS_OCEAN_SHALLOW,
        BIOME_SWAMP         => OBJECTS_SWAMP,
        BIOME_SWAMP_DARK    => OBJECTS_SWAMP_DARK,
        BIOME_WOODLANDS     => OBJECTS_WOODLANDS,
        BIOME_SAKURA        => OBJECTS_SAKURA,
        _                   => OBJECTS_GRASS,
    }
}

/// Encodes an item name into the InventoryItem wire format.
/// Format: u16(0 shorts) | u16(1 string) | str("item_id") | str(name) | u16(0 ints)
pub fn pack_item(name: &str) -> Vec<u8> {
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
// Relative biome commonness. Values are ratios (e.g. 1.00, 0.50) and are
// normalized against their sum to fill the 36-blob pool that covers a
// sector. Zero-weighted biomes never appear.
//
// RE: `GameController$ParseBiomeCommonness` in HybridsPublicServer
// uses the same notion of per-biome ratios read from `config_vals`.
// The original scaled percentages 0-100 to the client; our generator
// consumes them directly as ratios.
//
// Defaults mirror the original's 8-biome call pattern
//   (2, 1, 1, 1, 2, 1, 1, 1) expressed as fractions of the grass weight.

#[derive(Clone, Debug)]
pub struct BiomeWeights {
    pub grass:     f32,
    pub snow:      f32,
    pub desert:    f32,
    pub evergreen: f32,
    pub ocean:     f32,
    pub swamp:     f32,
    pub woodlands: f32,
    pub sakura:    f32,
}

impl Default for BiomeWeights {
    fn default() -> Self {
        Self {
            grass:     1.00,
            snow:      0.50,
            desert:    0.50,
            evergreen: 0.50,
            ocean:     1.00,
            swamp:     0.50,
            woodlands: 0.50,
            sakura:    0.50,
        }
    }
}

impl BiomeWeights {
    /// Expands the weights into an ordered 36-entry biome pool.
    ///
    /// Uses largest-remainder rounding so the counts sum to exactly 36
    /// regardless of the input weights. If every weight is <= 0 the pool
    /// falls back to all-grass.
    fn to_pool(&self) -> Vec<u8> {
        const TOTAL: usize = 36;
        let entries: [(u8, f32); 8] = [
            (BIOME_GRASS,     self.grass.max(0.0)),
            (BIOME_SNOW,      self.snow.max(0.0)),
            (BIOME_DESERT,    self.desert.max(0.0)),
            (BIOME_EVERGREEN, self.evergreen.max(0.0)),
            (BIOME_OCEAN,     self.ocean.max(0.0)),
            (BIOME_SWAMP,     self.swamp.max(0.0)),
            (BIOME_WOODLANDS, self.woodlands.max(0.0)),
            (BIOME_SAKURA,    self.sakura.max(0.0)),
        ];
        let sum: f32 = entries.iter().map(|e| e.1).sum();
        if sum <= 0.0 {
            return vec![BIOME_GRASS; TOTAL];
        }

        // Floor counts + remainders for largest-remainder apportionment.
        let mut counts = [0usize; 8];
        let mut rems   = [(0usize, 0.0_f32); 8];
        let mut assigned = 0usize;
        for (i, (_, w)) in entries.iter().enumerate() {
            let raw = *w / sum * TOTAL as f32;
            let floor = raw.floor() as usize;
            counts[i] = floor;
            rems[i]   = (i, raw - raw.floor());
            assigned += floor;
        }
        // Distribute remaining slots to the biggest remainders.
        rems.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let mut k = 0;
        while assigned < TOTAL {
            counts[rems[k % 8].0] += 1;
            assigned += 1;
            k += 1;
        }

        let mut pool = Vec::with_capacity(TOTAL);
        for (i, (biome, _)) in entries.iter().enumerate() {
            for _ in 0..counts[i] { pool.push(*biome); }
        }
        pool.truncate(TOTAL);
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
///
/// `start_biome` forces every chunk within `start_biome_radius` of (0,0)
/// to spawn as that biome — mirrors the `"Biome at start area"` option in
/// the original public server. A negative biome or zero radius disables
/// the override.
#[derive(Clone, Debug)]
pub struct WorldTemplate {
    pub seed:  u64,
    pub zones: Vec<ZoneConfig>,
    pub start_biome:        i16,
    pub start_biome_radius: i16,
}

impl Default for WorldTemplate {
    fn default() -> Self {
        Self {
            seed:  0,
            zones: vec![ZoneConfig::default_main()],
            start_biome:        BIOME_GRASS as i16,
            start_biome_radius: 3,
        }
    }
}

impl WorldTemplate {
    pub fn new(seed: u64, zones: Vec<ZoneConfig>) -> Self {
        Self { seed, zones, start_biome: BIOME_GRASS as i16, start_biome_radius: 3 }
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
        let mut biome   = blob_biomes[blob_id] as i16;

        // Start-area override: force spawn region to a fixed biome, matching
        // the `"Biome at start area"` knob on the public server. Applies when
        // the chunk is within `start_biome_radius` of (0,0) in both axes.
        let radius = self.template.start_biome_radius;
        if radius > 0
            && chunk_x.abs() <= radius as i32
            && chunk_z.abs() <= radius as i32
            && self.template.start_biome >= 0
            && (self.template.start_biome as usize) < BIOME_TEXTURE_COUNTS.len()
        {
            biome = self.template.start_biome;
        }

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
    /// Object counts and clustering behaviour vary per biome.
    /// Multi-tile objects occupy all covered cells in the occupancy bitset but
    /// are stored as a single element at their top-left tile.
    fn generate_chunk_elements(&self, chunk_x: i32, chunk_z: i32, biome: i16) -> Vec<PlacedObject> {
        let table = biome_object_table(biome);
        if table.is_empty() {
            return Vec::new();
        }

        // Separate RNG stream from floor properties (uses 0x01/0x02 as salts).
        let chunk_salt = (chunk_x as u64)
            .wrapping_mul(0x9e3779b97f4a7c15)
            .wrapping_add((chunk_z as u64).wrapping_mul(0x6c62272e07bb0142));
        let obj_seed = splitmix64(self.template.seed ^ chunk_salt ^ 0x0101_0101_0101_0101);
        let mut ctr: u64 = 0;
        let mut rng = |modulus: u32| -> u32 {
            ctr += 1;
            rng_u32(obj_seed, ctr) % modulus
        };

        let total_weight: u32 = table.iter().map(|e| e.1).sum();

        // SwampDark: 70% chance the chunk is completely empty.
        if biome as u8 == BIOME_SWAMP_DARK && rng(10) >= 3 {
            return Vec::new();
        }

        // Per-biome base object count range [min, max].
        let (min_obj, max_obj): (usize, usize) = match biome as u8 {
            BIOME_SWAMP_DARK    => (0, 1),
            BIOME_OCEAN         => (1, 2),
            BIOME_OCEAN_SHALLOW => (1, 3),
            BIOME_SNOW          => (1, 4),
            BIOME_DESERT        => (1, 4),
            BIOME_SWAMP         => (1, 3),
            _                   => (2, 5),
        };
        let num_objects = min_obj + rng((max_obj - min_obj + 1) as u32) as usize;

        let mut occupied: u128 = 0;
        let mut elements: Vec<PlacedObject> = Vec::with_capacity(num_objects + 8);
        let mut spirit_tree_pos: Option<(u8, u8)> = None;

        'outer: for _ in 0..num_objects {
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
            let rotation = rng(4) as u8;

            for _ in 0..10 {
                let max_pos = 10u32.saturating_sub(size as u32);
                let tx = rng(max_pos + 1);
                let tz = rng(max_pos + 1);
                let mut mask: u128 = 0;
                for dz in 0..(size as u32) {
                    for dx in 0..(size as u32) {
                        mask |= 1u128 << ((tz + dz) * 10 + (tx + dx));
                    }
                }
                if occupied & mask == 0 {
                    occupied |= mask;
                    if name == "Spirit Tree" {
                        spirit_tree_pos = Some((tx as u8, tz as u8));
                    }
                    elements.push(PlacedObject {
                        cell_x:    tx as u8,
                        cell_z:    tz as u8,
                        rotation,
                        item_data: pack_item(name),
                    });

                    // Biome-specific cluster spawning after placement.
                    match biome as u8 {
                        BIOME_GRASS => match name {
                            "Metal Vein" => {
                                if rng(4) == 0 {
                                    let n = 1 + rng(2);
                                    place_cluster("Metal Vein", 2, n, tx as u8, tz as u8, 2, &mut occupied, &mut elements, &mut rng);
                                }
                            }
                            "Large Metal Vein" => {
                                if rng(2) == 0 {
                                    let n = 1 + rng(2);
                                    place_cluster("Metal Vein", 2, n, tx as u8, tz as u8, 2, &mut occupied, &mut elements, &mut rng);
                                }
                            }
                            "Stone Vein" => {
                                if rng(8) == 0 {
                                    place_cluster("Stone Vein", 2, 1, tx as u8, tz as u8, 2, &mut occupied, &mut elements, &mut rng);
                                }
                            }
                            "Large Stone Vein" => {
                                if rng(4) == 0 {
                                    place_cluster("Stone Vein", 2, 1, tx as u8, tz as u8, 2, &mut occupied, &mut elements, &mut rng);
                                }
                            }
                            "Cotton Plant" => {
                                if rng(4) == 0 {
                                    let n = 2 + rng(4);
                                    place_cluster("Cotton Plant", 1, n, tx as u8, tz as u8, 2, &mut occupied, &mut elements, &mut rng);
                                }
                            }
                            _ => {}
                        },
                        BIOME_SNOW => match name {
                            "MoonBerry Bush" => {
                                let n = 1 + rng(4);
                                place_cluster("MoonBerry Bush", 1, n, tx as u8, tz as u8, 2, &mut occupied, &mut elements, &mut rng);
                            }
                            "Titanium Vein" => {
                                if rng(4) == 0 {
                                    place_cluster("Titanium Vein", 2, 1, tx as u8, tz as u8, 2, &mut occupied, &mut elements, &mut rng);
                                }
                            }
                            "Stone Vein (Snowy)" => {
                                if rng(8) == 0 {
                                    place_cluster("Stone Vein (Snowy)", 2, 1, tx as u8, tz as u8, 2, &mut occupied, &mut elements, &mut rng);
                                }
                            }
                            _ => {}
                        },
                        BIOME_DESERT => match name {
                            "Gold Vein" => {
                                if rng(4) == 0 {
                                    let n = 1 + rng(2);
                                    place_cluster("Gold Vein", 2, n, tx as u8, tz as u8, 2, &mut occupied, &mut elements, &mut rng);
                                }
                            }
                            "Uranium Vein" => {
                                if rng(4) == 0 {
                                    let n = 1 + rng(2);
                                    place_cluster("Uranium Vein", 2, n, tx as u8, tz as u8, 2, &mut occupied, &mut elements, &mut rng);
                                }
                            }
                            "Stone Vein (Desert)" => {
                                if rng(8) == 0 {
                                    place_cluster("Stone Vein (Desert)", 2, 1, tx as u8, tz as u8, 2, &mut occupied, &mut elements, &mut rng);
                                }
                            }
                            _ => {}
                        },
                        BIOME_EVERGREEN => match name {
                            "Metal Vein" => {
                                if rng(4) == 0 {
                                    let n = 1 + rng(2);
                                    place_cluster("Metal Vein", 2, n, tx as u8, tz as u8, 2, &mut occupied, &mut elements, &mut rng);
                                }
                            }
                            "Giant Red Mushroom" => {
                                let r = 1 + rng(2);
                                place_cluster("Dug-Up Red Mushroom", 1, r, tx as u8, tz as u8, 2, &mut occupied, &mut elements, &mut rng);
                                let b = 1 + rng(2);
                                place_cluster("Dug-up Brown Mushroom", 1, b, tx as u8, tz as u8, 2, &mut occupied, &mut elements, &mut rng);
                            }
                            "Dug-up Brown Mushroom" => {
                                if rng(10) < 2 {
                                    let n = 1 + rng(2);
                                    place_cluster("Dug-up Brown Mushroom", 1, n, tx as u8, tz as u8, 2, &mut occupied, &mut elements, &mut rng);
                                }
                            }
                            "Dug-Up Red Mushroom" => {
                                if rng(10) < 2 {
                                    let n = 1 + rng(2);
                                    place_cluster("Dug-Up Red Mushroom", 1, n, tx as u8, tz as u8, 2, &mut occupied, &mut elements, &mut rng);
                                }
                            }
                            "Stone Vein" => {
                                if rng(8) == 0 {
                                    place_cluster("Stone Vein", 2, 1, tx as u8, tz as u8, 2, &mut occupied, &mut elements, &mut rng);
                                }
                            }
                            _ => {}
                        },
                        BIOME_WOODLANDS => match name {
                            // Wheat always comes in small clusters of 2–5 (one placed, add 1–4).
                            "Dug-Up Wheat" => {
                                let n = 1 + rng(4);
                                place_cluster("Dug-Up Wheat", 1, n, tx as u8, tz as u8, 2, &mut occupied, &mut elements, &mut rng);
                            }
                            // Brown mushrooms like clusters of 2–4 (add 1–3).
                            "Dug-up Brown Mushroom" => {
                                let n = 1 + rng(3);
                                place_cluster("Dug-up Brown Mushroom", 1, n, tx as u8, tz as u8, 2, &mut occupied, &mut elements, &mut rng);
                            }
                            // Regular pumpkins rarely cluster (1–3 when they do).
                            "Dug-up Pumpkin" => {
                                if rng(4) == 0 {
                                    let n = 1 + rng(2);
                                    place_cluster("Dug-up Pumpkin", 1, n, tx as u8, tz as u8, 2, &mut occupied, &mut elements, &mut rng);
                                }
                            }
                            // Veins cluster; the large variant is a rare cluster seed.
                            "Metal Vein" => {
                                if rng(4) == 0 {
                                    let n = 1 + rng(2);
                                    place_cluster("Metal Vein", 2, n, tx as u8, tz as u8, 2, &mut occupied, &mut elements, &mut rng);
                                }
                            }
                            "Large Metal Vein" => {
                                if rng(3) == 0 {
                                    let n = 1 + rng(2);
                                    place_cluster("Metal Vein", 2, n, tx as u8, tz as u8, 2, &mut occupied, &mut elements, &mut rng);
                                }
                            }
                            "Stone Vein (White)" => {
                                if rng(4) == 0 {
                                    let n = 1 + rng(2);
                                    place_cluster("Stone Vein (White)", 2, n, tx as u8, tz as u8, 2, &mut occupied, &mut elements, &mut rng);
                                }
                            }
                            "Silver Vein" => {
                                if rng(3) == 0 {
                                    place_cluster("Silver Vein", 2, 1, tx as u8, tz as u8, 2, &mut occupied, &mut elements, &mut rng);
                                }
                            }
                            _ => {}
                        },
                        BIOME_SWAMP => match name {
                            // Swamp clusters are small (1–2, rarely 3) and spaced out (radius 3).
                            "Stone Vein" => {
                                if rng(3) == 0 {
                                    let n = if rng(10) == 0 { 2 } else { 1 };
                                    place_cluster("Stone Vein", 2, n, tx as u8, tz as u8, 3, &mut occupied, &mut elements, &mut rng);
                                }
                            }
                            "Metal Vein" => {
                                if rng(3) == 0 {
                                    let n = if rng(10) == 0 { 2 } else { 1 };
                                    place_cluster("Metal Vein", 2, n, tx as u8, tz as u8, 3, &mut occupied, &mut elements, &mut rng);
                                }
                            }
                            // Large veins rarely seed clusters here, and fill with the small variant.
                            "Large Metal Vein" => {
                                if rng(5) == 0 {
                                    place_cluster("Metal Vein", 2, 1, tx as u8, tz as u8, 3, &mut occupied, &mut elements, &mut rng);
                                }
                            }
                            "Dug-up Brown Mushroom" => {
                                if rng(3) == 0 {
                                    place_cluster("Dug-up Brown Mushroom", 1, 1, tx as u8, tz as u8, 3, &mut occupied, &mut elements, &mut rng);
                                }
                            }
                            _ => {}
                        },
                        _ => {}
                    }

                    continue 'outer;
                }
            }
        }

        // Sakura: very rarely place a Spirit Branch near any Spirit Tree that spawned.
        if biome as u8 == BIOME_SAKURA {
            if let Some((stx, stz)) = spirit_tree_pos {
                if rng(5) == 0 {
                    place_cluster("Spawner - Spirit Branch", 1, 1, stx, stz, 3, &mut occupied, &mut elements, &mut rng);
                }
            }
        }

        elements
    }
}

/// Places `count` copies of `name` (tile footprint `size`×`size`) within
/// `radius` cells of `(anchor_x, anchor_z)`, respecting the occupancy bitset.
fn place_cluster(
    name:     &'static str,
    size:     u8,
    count:    u32,
    anchor_x: u8,
    anchor_z: u8,
    radius:   i32,
    occupied: &mut u128,
    elements: &mut Vec<PlacedObject>,
    rng:      &mut dyn FnMut(u32) -> u32,
) {
    let s   = size as i32;
    let span = (radius * 2 + 1) as u32;
    for _ in 0..count {
        for _ in 0..10 {
            let dx = rng(span) as i32 - radius;
            let dz = rng(span) as i32 - radius;
            let tx = (anchor_x as i32 + dx).clamp(0, 10 - s) as u32;
            let tz = (anchor_z as i32 + dz).clamp(0, 10 - s) as u32;
            let mut mask: u128 = 0;
            for ddz in 0..(s as u32) {
                for ddx in 0..(s as u32) {
                    mask |= 1u128 << ((tz + ddz) * 10 + (tx + ddx));
                }
            }
            if *occupied & mask == 0 {
                *occupied |= mask;
                let rotation = rng(4) as u8;
                elements.push(PlacedObject {
                    cell_x:    tx as u8,
                    cell_z:    tz as u8,
                    rotation,
                    item_data: pack_item(name),
                });
                break;
            }
        }
    }
}
