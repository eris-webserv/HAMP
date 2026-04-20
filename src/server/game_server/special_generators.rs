// special_generators.rs — server-side worldgen for cave, cloud (Magic Bean), and hell (Spooky Well) zones.
//
// RE sources:
//   ChunkGeneratorClouds__GenerateUnexplored  0x995b68
//   ChunkGeneratorHell__GenerateUnexplored    0x996c1c
//   ChunkGeneratorCaves__GenerateUnexplored   0x98eac8
//   InventoryUtils__IsCaveObject              0x88d50c
//   InventoryUtils__IsHeavenDimension         0x88d680
//   InventoryUtils__IsHellDimension           0x88d750

use super::generator::{pack_item, ChunkBiomeParams, PlacedObject};

// ── Zone classification ───────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub enum ZoneKind {
    House,
    Cave { item_id: String },
    Cloud,
    Hell,
}

pub fn zone_kind_from_item_id(item_id: &str) -> ZoneKind {
    match item_id {
        "Magic Bean" => ZoneKind::Cloud,
        "Spooky Well" => ZoneKind::Hell,
        s if is_cave_item(s) => ZoneKind::Cave { item_id: s.to_string() },
        _ => ZoneKind::House,
    }
}

fn is_cave_item(item_id: &str) -> bool {
    matches!(
        item_id,
        "cave"
            | "Personal Mine"
            | "Grass Cave Entrance"
            | "Snow Cave Entrance"
            | "Desert Cave Entrance"
            | "Evergreen Cave Entrance"
            | "Ocean Cave Entrance"
            | "Swamp Cave Entrance"
    )
}

// ── RNG (mirrors splitmix64 in generator.rs) ──────────────────────────────

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

fn rng_u32(seed: u64, salt: u64) -> u32 {
    splitmix64(seed ^ splitmix64(salt)) as u32
}

fn zone_chunk_seed(world_seed: u64, shack_id: i32, cx: i16, cz: i16) -> u64 {
    let zone_salt = (shack_id as u64).wrapping_mul(0x517cc1b727220a95);
    let chunk_salt = (cx as u64)
        .wrapping_mul(0x9e3779b97f4a7c15)
        .wrapping_add((cz as u64).wrapping_mul(0x6c62272e07bb0142));
    splitmix64(world_seed ^ zone_salt ^ chunk_salt)
}

// ── Lava variant bitmaps ──────────────────────────────────────────────────
//
// Precomputed from assets/Variant1-5.png (10×10 pixels).
// pixel mapping: cell(x,z) is lava if PNG pixel at (col=9-z, row=9-x) is opaque black.

const LAVA_V1: [(u8, u8); 65] = [
    (9,7),(9,6),(9,5),(9,4),(9,3),(9,2),(9,1),(8,8),(8,7),(8,6),(8,5),(8,4),(8,3),(8,2),(8,1),
    (7,8),(7,7),(7,6),(7,3),(7,2),(7,1),(7,0),(6,9),(6,8),(6,7),(6,2),(6,1),(6,0),(5,9),(5,8),
    (5,1),(5,0),(4,9),(4,8),(4,1),(4,0),(3,9),(3,8),(3,2),(3,1),(3,0),(2,9),(2,8),(2,7),(2,3),
    (2,2),(2,1),(2,0),(1,9),(1,8),(1,7),(1,6),(1,5),(1,4),(1,3),(1,2),(1,1),(1,0),(0,7),(0,6),
    (0,5),(0,4),(0,3),(0,2),(0,1),
];
const LAVA_V2: [(u8, u8); 18] = [
    (7,6),(7,5),(6,7),(6,6),(6,5),(5,8),(5,7),(5,6),(5,5),(4,9),(4,8),(4,7),(4,6),(3,9),(3,8),
    (3,7),(2,8),(2,7),
];
const LAVA_V3: [(u8, u8); 15] = [
    (9,7),(9,6),(8,8),(8,7),(8,6),(7,8),(7,7),(3,2),(3,1),(2,3),(2,2),(2,1),(1,3),(1,2),(1,1),
];
const LAVA_V4: [(u8, u8); 43] = [
    (9,4),(9,3),(9,2),(8,5),(8,4),(8,3),(8,2),(8,1),(7,5),(7,4),(7,3),(7,2),(7,1),(7,0),(6,5),
    (6,4),(6,3),(6,2),(6,1),(6,0),(5,4),(5,3),(5,2),(5,1),(5,0),(4,4),(4,3),(4,2),(4,1),(4,0),
    (3,4),(3,3),(3,2),(3,1),(3,0),(2,4),(2,3),(2,2),(2,1),(2,0),(1,3),(1,2),(1,1),
];
const LAVA_V5: [(u8, u8); 96] = [
    (9,8),(9,7),(9,6),(9,5),(9,4),(9,3),(9,2),(9,1),(8,9),(8,8),(8,7),(8,6),(8,5),(8,4),(8,3),
    (8,2),(8,1),(8,0),(7,9),(7,8),(7,7),(7,6),(7,5),(7,4),(7,3),(7,2),(7,1),(7,0),(6,9),(6,8),
    (6,7),(6,6),(6,5),(6,4),(6,3),(6,2),(6,1),(6,0),(5,9),(5,8),(5,7),(5,6),(5,5),(5,4),(5,3),
    (5,2),(5,1),(5,0),(4,9),(4,8),(4,7),(4,6),(4,5),(4,4),(4,3),(4,2),(4,1),(4,0),(3,9),(3,8),
    (3,7),(3,6),(3,5),(3,4),(3,3),(3,2),(3,1),(3,0),(2,9),(2,8),(2,7),(2,6),(2,5),(2,4),(2,3),
    (2,2),(2,1),(2,0),(1,9),(1,8),(1,7),(1,6),(1,5),(1,4),(1,3),(1,2),(1,1),(1,0),(0,8),(0,7),
    (0,6),(0,5),(0,4),(0,3),(0,2),(0,1),
];
const LAVA_VARIANTS: [&[(u8, u8)]; 5] = [&LAVA_V1, &LAVA_V2, &LAVA_V3, &LAVA_V4, &LAVA_V5];

const HELL_MOBS: [&str; 10] = [
    "bat", "devil", "dragon", "dwarf", "gecko",
    "politician", "raptor", "scorpion", "snake", "trex",
];

// ── Deterministic RNG state ───────────────────────────────────────────────

struct Rng {
    seed: u64,
    ctr:  u64,
}

impl Rng {
    fn new(seed: u64) -> Self { Self { seed, ctr: 0 } }

    fn next(&mut self, modulus: u32) -> u32 {
        self.ctr += 1;
        rng_u32(self.seed, self.ctr) % modulus
    }

    fn try_place(&mut self, elements: &mut Vec<PlacedObject>, occupied: &mut u128, item_name: &str) {
        self.try_place_rot(elements, occupied, item_name, 0);
    }

    fn try_place_rot(&mut self, elements: &mut Vec<PlacedObject>, occupied: &mut u128, item_name: &str, rotation: u8) {
        for _ in 0..10 {
            self.ctr += 1;
            let r = rng_u32(self.seed, self.ctr);
            let px = (r % 10) as u8;
            let pz = ((r >> 16) % 10) as u8;
            let bit = pz as u128 * 10 + px as u128;
            if (*occupied >> bit) & 1 == 0 {
                *occupied |= 1 << bit;
                elements.push(PlacedObject { cell_x: px, cell_z: pz, rotation, item_data: pack_item(item_name) });
                return;
            }
        }
    }
}

// ── Hell (Spooky Well) ────────────────────────────────────────────────────
//
// RE: ChunkGeneratorHell__GenerateUnexplored (0x996c1c)
//
//  1. mob_a, mob_b = random picks from HELL_MOBS (10 entries)
//  2. lava variant 1-5 → place "Lava" on all black pixels
//  3. 2 stalagmites ("Large Black Stalagmite" or "Small Black Stalagmite"), each 50/50
//  4. 7% chance: "Old Torch"
//  5. 20% chance: "Magmite Vein" cluster (1 + 1-2 neighbours)
//  6. 12% chance: mob spawners by tier (0-20%: 2×Normal, 20-40%: 3×Normal,
//     40-60%: 1×Giant, 60-80%: 2×VengefulSpirit, 80-100%: 1×Bandit+2×BanditElite)

pub fn generate_hell_chunk(world_seed: u64, shack_id: i32, cx: i16, cz: i16) -> ChunkBiomeParams {
    let mut rng = Rng::new(zone_chunk_seed(world_seed, shack_id, cx, cz));

    let mob_a = HELL_MOBS[rng.next(10) as usize].to_string();
    let mob_b = HELL_MOBS[rng.next(10) as usize].to_string();

    let variant_idx = rng.next(5) as usize;
    let lava_cells = LAVA_VARIANTS[variant_idx];

    let mut elements: Vec<PlacedObject> = Vec::new();
    let mut occupied: u128 = 0;

    for &(lx, lz) in lava_cells {
        let bit = lz as u128 * 10 + lx as u128;
        occupied |= 1 << bit;
        elements.push(PlacedObject { cell_x: lx, cell_z: lz, rotation: 0, item_data: pack_item("Lava") });
    }

    for _ in 0..2 {
        let name = if rng.next(2) == 0 { "Large Black Stalagmite" } else { "Small Black Stalagmite" };
        rng.try_place(&mut elements, &mut occupied, name);
    }

    if rng.next(100) < 7 {
        rng.try_place(&mut elements, &mut occupied, "Old Torch");
    }

    if rng.next(100) < 20 {
        let vx = rng.next(10) as i32;
        let vz = rng.next(10) as i32;
        let bit = vz as u128 * 10 + vx as u128;
        if (occupied >> bit) & 1 == 0 {
            let rot = rng.next(4) as u8;
            occupied |= 1 << bit;
            elements.push(PlacedObject { cell_x: vx as u8, cell_z: vz as u8, rotation: rot, item_data: pack_item("Magmite Vein") });

            let extra = 1 + rng.next(2) as usize;
            let mut anchor_x = vx;
            let mut anchor_z = vz;
            for _ in 0..extra {
                let dx = rng.next(5) as i32 - 2;
                let dz = rng.next(5) as i32 - 2;
                let nx = anchor_x + dx;
                let nz = anchor_z + dz;
                if nx >= 0 && nx < 10 && nz >= 0 && nz < 10 {
                    let nbit = nz as u128 * 10 + nx as u128;
                    if (occupied >> nbit) & 1 == 0 {
                        let nrot = rng.next(4) as u8;
                        occupied |= 1 << nbit;
                        elements.push(PlacedObject { cell_x: nx as u8, cell_z: nz as u8, rotation: nrot, item_data: pack_item("Magmite Vein") });
                        anchor_x = nx;
                        anchor_z = nz;
                    }
                }
            }
        }
    }

    if rng.next(100) < 12 {
        let tier = rng.next(100);
        if tier < 20 {
            rng.try_place(&mut elements, &mut occupied, "Mob - Normal");
            rng.try_place(&mut elements, &mut occupied, "Mob - Normal");
        } else if tier < 40 {
            for _ in 0..3 { rng.try_place(&mut elements, &mut occupied, "Mob - Normal"); }
        } else if tier < 60 {
            rng.try_place(&mut elements, &mut occupied, "Mob - Giant");
        } else if tier < 80 {
            rng.try_place(&mut elements, &mut occupied, "Mob - Vengeful Spirit");
            rng.try_place(&mut elements, &mut occupied, "Mob - Vengeful Spirit");
        } else {
            rng.try_place(&mut elements, &mut occupied, "Mob - Bandit");
            rng.try_place(&mut elements, &mut occupied, "Mob - Bandit Elite");
            rng.try_place(&mut elements, &mut occupied, "Mob - Bandit Elite");
        }
    }

    let floor_rot = rng.next(4) as i16;

    ChunkBiomeParams { biome: 0, floor_rot, floor_tex: 0, mob_a, mob_b, elements }
}

// ── Cloud (Magic Bean / Heaven) ───────────────────────────────────────────
//
// RE: ChunkGeneratorClouds__GenerateUnexplored (0x995b68)
//
//  1. mob_a = mob_b = "angel"
//  2. 2-4 clusters of 25 "Clouds" placements (offsets from random centre)
//  3. 20% chance: place 1 mob on a random occupied cloud cell

// 25 (dx, dz) offsets per cluster — extracted from the decompile.
const CLOUD_OFFSETS: [(i32, i32); 25] = [
    (0, 0), (1, 0), (0, 1), (-1, 0), (0, -1),
    (1, 1), (1, -1), (-1, 1), (-1, -1),
    (2, 0), (0, 2), (-2, 0), (0, -2),
    (2, 2), (2, -2), (-2, 2), (-2, -2),
    (1, 2), (-1, 2), (1, -2), (-1, -2),
    (2, 1), (2, -1), (-2, 1), (-2, -1),
];

pub fn generate_cloud_chunk(world_seed: u64, shack_id: i32, cx: i16, cz: i16) -> ChunkBiomeParams {
    let mut rng = Rng::new(zone_chunk_seed(world_seed, shack_id, cx, cz));

    let mut elements: Vec<PlacedObject> = Vec::new();
    let mut occupied: u128 = 0;
    let mut cloud_cells: Vec<(u8, u8)> = Vec::new();

    let cluster_count = 2 + rng.next(3) as usize; // Range(2,5) → 2,3,4
    for _ in 0..cluster_count {
        let centre_x = rng.next(10) as i32;
        let centre_z = rng.next(10) as i32;
        for &(dx, dz) in &CLOUD_OFFSETS {
            let px = centre_x + dx;
            let pz = centre_z + dz;
            if px < 0 || px >= 10 || pz < 0 || pz >= 10 { continue; }
            let bit = pz as u128 * 10 + px as u128;
            if (occupied >> bit) & 1 == 0 {
                occupied |= 1 << bit;
                let px = px as u8;
                let pz = pz as u8;
                elements.push(PlacedObject { cell_x: px, cell_z: pz, rotation: 0, item_data: pack_item("Clouds") });
                cloud_cells.push((px, pz));
            }
        }
    }

    if !cloud_cells.is_empty() && rng.next(100) < 20 {
        let idx = rng.next(cloud_cells.len() as u32) as usize;
        let (mx, mz) = cloud_cells[idx];
        elements.push(PlacedObject { cell_x: mx, cell_z: mz, rotation: 0, item_data: pack_item("angel") });
    }

    let floor_rot = rng.next(4) as i16;

    ChunkBiomeParams { biome: 0, floor_rot, floor_tex: 0, mob_a: "angel".to_string(), mob_b: "angel".to_string(), elements }
}

// ── Cave ─────────────────────────────────────────────────────────────────
//
// RE: ChunkGeneratorCaves__GenerateUnexplored (0x98eac8)
//
// The original generator is driven by UngeneratedCaveChunk config objects
// (floor_model, small_ore, large_ore, add_spikes, etc.).  HAMP uses a
// simplified deterministic version:
//
//  1. Place "DEBUG 1x1 fillempty" walls on all four border rows/columns.
//  2. Place one "Spawner - Fossils" at a random interior cell.
//  3. Derive small/large ore from the entrance item biome and place a cluster.
//  4. Optional poison spikes (floor_model != 0 only, 15% chance per interior cell).
//
// floor_model: 0 = dirt cave, 1 = stone cave (enables spikes + cave art).

fn cave_biome_ores(item_id: &str) -> (&'static str, &'static str) {
    match item_id {
        "Snow Cave Entrance"      => ("Mineral - Snowflake Crystal", "Snow Crystal Cluster"),
        "Desert Cave Entrance"    => ("Mineral - Cactus Crystal",    "Desert Crystal Cluster"),
        "Evergreen Cave Entrance" => ("Mineral - Pine Crystal",      "Evergreen Crystal Cluster"),
        "Ocean Cave Entrance"     => ("Mineral - Ocean Crystal",     "Ocean Crystal Cluster"),
        "Swamp Cave Entrance"     => ("Mineral - Swamp Crystal",     "Swamp Crystal Cluster"),
        _                         => ("Mineral - Crystal",           "Crystal Cluster"),
    }
}

pub fn cave_floor_model(item_id: &str) -> i16 {
    match item_id {
        "Personal Mine" => 1, // stone floors → spikes possible
        _               => 0,
    }
}

pub fn generate_cave_chunk(world_seed: u64, shack_id: i32, cx: i16, cz: i16, item_id: &str) -> ChunkBiomeParams {
    let mut rng = Rng::new(zone_chunk_seed(world_seed, shack_id, cx, cz));

    let floor_model = cave_floor_model(item_id);
    let mut elements: Vec<PlacedObject> = Vec::new();
    let mut occupied: u128 = 0;

    // Border walls — all 4 edges (skip corners to avoid double-placing)
    for i in 0u8..10 {
        for &(wx, wz) in &[(0u8, i), (9u8, i), (i, 0u8), (i, 9u8)] {
            let bit = wz as u128 * 10 + wx as u128;
            if (occupied >> bit) & 1 == 0 {
                occupied |= 1 << bit;
                elements.push(PlacedObject { cell_x: wx, cell_z: wz, rotation: 0, item_data: pack_item("DEBUG 1x1 fillempty") });
            }
        }
    }

    // Fossil spawner at a random interior cell (1-8 range)
    let fx = 1 + rng.next(8) as u8;
    let fz = 1 + rng.next(8) as u8;
    let bit = fz as u128 * 10 + fx as u128;
    if (occupied >> bit) & 1 == 0 {
        occupied |= 1 << bit;
        elements.push(PlacedObject { cell_x: fx, cell_z: fz, rotation: 0, item_data: pack_item("Spawner - Fossils") });
    }

    // Ore cluster
    let (small_ore, large_ore) = cave_biome_ores(item_id);
    let ore_count = 2 + rng.next(4) as usize;
    for i in 0..ore_count {
        let ore_name = if i % 3 == 0 { large_ore } else { small_ore };
        let rot = rng.next(4) as u8;
        rng.try_place_rot(&mut elements, &mut occupied, ore_name, rot);
    }

    // Poison spikes in Personal Mine (floor_model == 1)
    if floor_model == 1 {
        for sx in 1u8..9 {
            for sz in 1u8..9 {
                if rng.next(100) < 15 {
                    let bit = sz as u128 * 10 + sx as u128;
                    if (occupied >> bit) & 1 == 0 {
                        occupied |= 1 << bit;
                        elements.push(PlacedObject { cell_x: sx, cell_z: sz, rotation: 0, item_data: pack_item("Poison Spikes") });
                    }
                }
            }
        }
    }

    let floor_rot = rng.next(4) as i16;

    ChunkBiomeParams {
        biome: 0,
        floor_rot,
        floor_tex: 0,
        mob_a: String::new(),
        mob_b: String::new(),
        elements,
    }
}
