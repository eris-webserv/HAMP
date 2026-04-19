// persist.rs — binary world-state save/load.
//
// File: world.hws  (HAMP World State)
//
//  ─── Header ────────────────────────────────────────────────────────────────
//  [4]  magic:   b"HAMP"
//  [1]  version: u8 = 1
//
//  ─── Template ──────────────────────────────────────────────────────────────
//  [8]  seed: u64 le
//  [2]  start_biome: i16 le               (v2+)
//  [2]  start_biome_radius: i16 le        (v2+)
//  [2]  zone_count: u16 le
//  per zone:
//    [str] name: u16_len + utf-8 bytes
//    v1:   [8]   biome weights: u8 × 8
//    v2+:  [32]  biome weights: f32 le × 8
//             (grass, snow, desert, evergreen, ocean, swamp, woodlands, sakura)
//
//  ─── Chunks ────────────────────────────────────────────────────────────────
//  [4]  chunk_count: u32 le
//  per chunk:
//    [2]  x:          i16 le
//    [2]  z:          i16 le
//    [str] zone:      u16_len + utf-8 bytes
//    [2]  biome:      i16 le
//    [2]  floor_rot:  i16 le
//    [2]  floor_tex:  i16 le
//    [2]  floor_model:i16 le
//    [str] mob_a:     u16_len + utf-8 bytes
//    [str] mob_b:     u16_len + utf-8 bytes
//    [2]  element_count: u16 le
//    per element:
//      [1]  cell_x:       u8
//      [1]  cell_z:       u8
//      [1]  rotation:     u8
//      [2]  item_data_len:u16 le
//      [N]  item_data:    bytes
//
//  ─── Containers (reserved) ─────────────────────────────────────────────────
//  [4]  container_count: u32 le = 0

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::sync::RwLock;

use super::baskets::BasketStore;
use super::generator::{BiomeWeights, WorldGenerator, WorldTemplate, ZoneConfig};
use super::world_state::{Chunk, ChunkElement, WorldState, ZoneEntry};

const MAGIC: &[u8; 4] = b"HAMP";
const VERSION: u8 = 2;
pub const FILE_NAME: &str = "world.hws";

// ── Low-level write helpers ───────────────────────────────────────────────

fn wu8 <W: Write>(w: &mut W, v: u8)  -> io::Result<()> { w.write_all(&[v]) }
fn wi16<W: Write>(w: &mut W, v: i16) -> io::Result<()> { w.write_all(&v.to_le_bytes()) }
fn wu16<W: Write>(w: &mut W, v: u16) -> io::Result<()> { w.write_all(&v.to_le_bytes()) }
fn wu32<W: Write>(w: &mut W, v: u32) -> io::Result<()> { w.write_all(&v.to_le_bytes()) }
fn wu64<W: Write>(w: &mut W, v: u64) -> io::Result<()> { w.write_all(&v.to_le_bytes()) }

fn wstr<W: Write>(w: &mut W, s: &str) -> io::Result<()> {
    let b = s.as_bytes();
    wu16(w, b.len() as u16)?;
    w.write_all(b)
}

fn wbytes<W: Write>(w: &mut W, data: &[u8]) -> io::Result<()> {
    wu16(w, data.len() as u16)?;
    w.write_all(data)
}

// ── Low-level read helpers ────────────────────────────────────────────────

fn ru8 <R: Read>(r: &mut R) -> io::Result<u8>  { let mut b=[0u8;1]; r.read_exact(&mut b)?; Ok(b[0]) }
fn ri16<R: Read>(r: &mut R) -> io::Result<i16> { let mut b=[0u8;2]; r.read_exact(&mut b)?; Ok(i16::from_le_bytes(b)) }
fn ru16<R: Read>(r: &mut R) -> io::Result<u16> { let mut b=[0u8;2]; r.read_exact(&mut b)?; Ok(u16::from_le_bytes(b)) }
fn ru32<R: Read>(r: &mut R) -> io::Result<u32> { let mut b=[0u8;4]; r.read_exact(&mut b)?; Ok(u32::from_le_bytes(b)) }
fn ru64<R: Read>(r: &mut R) -> io::Result<u64> { let mut b=[0u8;8]; r.read_exact(&mut b)?; Ok(u64::from_le_bytes(b)) }

fn rstr<R: Read>(r: &mut R) -> io::Result<String> {
    let len = ru16(r)? as usize;
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf)?;
    String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn rbytes<R: Read>(r: &mut R) -> io::Result<Vec<u8>> {
    let len = ru16(r)? as usize;
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf)?;
    Ok(buf)
}

// ── Save ──────────────────────────────────────────────────────────────────

/// Writes the full world state to `path` atomically (write-then-rename).
pub fn save(state: &WorldState, path: &Path) -> io::Result<()> {
    let tmp = path.with_extension("hws.tmp");
    {
        let file = File::create(&tmp)?;
        let mut w = BufWriter::new(file);
        write_state(state, &mut w)?;
        w.flush()?;
    }
    fs::rename(&tmp, path)
}

fn write_state<W: Write>(state: &WorldState, w: &mut W) -> io::Result<()> {
    // Header
    w.write_all(MAGIC)?;
    wu8(w, VERSION)?;

    // Template
    let tmpl = state.generator.template();
    wu64(w, tmpl.seed)?;
    wi16(w, tmpl.start_biome)?;
    wi16(w, tmpl.start_biome_radius)?;
    wu16(w, tmpl.zones.len() as u16)?;
    for zone in &tmpl.zones {
        wstr(w, &zone.name)?;
        let wt = &zone.weights;
        for v in [wt.grass, wt.snow, wt.desert, wt.evergreen,
                  wt.ocean, wt.swamp, wt.woodlands, wt.sakura] {
            w.write_all(&v.to_le_bytes())?;
        }
    }

    // Chunks
    let chunks = state.chunks.read().unwrap();
    wu32(w, chunks.len() as u32)?;
    for chunk in chunks.values() {
        wi16(w, chunk.x)?;
        wi16(w, chunk.z)?;
        wstr(w, &chunk.zone)?;
        wi16(w, chunk.biome)?;
        wi16(w, chunk.floor_rot)?;
        wi16(w, chunk.floor_tex)?;
        wi16(w, chunk.floor_model)?;
        wstr(w, &chunk.mob_a)?;
        wstr(w, &chunk.mob_b)?;
        wu16(w, chunk.elements.len() as u16)?;
        for el in &chunk.elements {
            wu8(w, el.cell_x)?;
            wu8(w, el.cell_z)?;
            wu8(w, el.rotation)?;
            wbytes(w, &el.item_data)?;
        }
    }

    // Containers (reserved)
    wu32(w, 0)?;

    Ok(())
}

// ── Load ──────────────────────────────────────────────────────────────────

/// Reads a world state file and reconstructs a `WorldState`.
/// The generator is re-created from the saved template so that
/// chunks outside the saved radius can still be lazily generated.
pub fn load(path: &Path) -> io::Result<WorldState> {
    let file = File::open(path)?;
    let mut r = BufReader::new(file);

    // Header
    let mut magic = [0u8; 4];
    r.read_exact(&mut magic)?;
    if &magic != MAGIC {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "not a HAMP world file"));
    }
    let version = ru8(&mut r)?;
    if version != 1 && version != 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unsupported world file version {version}"),
        ));
    }

    // Template
    let seed = ru64(&mut r)?;
    let (start_biome, start_biome_radius) = if version >= 2 {
        (ri16(&mut r)?, ri16(&mut r)?)
    } else {
        (0, 0) // v1 had no start-area override
    };
    let zone_count = ru16(&mut r)? as usize;
    let mut zones = Vec::with_capacity(zone_count);
    for _ in 0..zone_count {
        let name = rstr(&mut r)?;
        let weights = if version >= 2 {
            let mut buf = [0u8; 32];
            r.read_exact(&mut buf)?;
            let rf = |i: usize| f32::from_le_bytes(buf[i*4..i*4+4].try_into().unwrap());
            BiomeWeights {
                grass:     rf(0), snow:      rf(1), desert:    rf(2), evergreen: rf(3),
                ocean:     rf(4), swamp:     rf(5), woodlands: rf(6), sakura:    rf(7),
            }
        } else {
            let mut wb = [0u8; 8];
            r.read_exact(&mut wb)?;
            BiomeWeights {
                grass:     wb[0] as f32, snow:      wb[1] as f32,
                desert:    wb[2] as f32, evergreen: wb[3] as f32,
                ocean:     wb[4] as f32, swamp:     wb[5] as f32,
                woodlands: wb[6] as f32, sakura:    wb[7] as f32,
            }
        };
        zones.push(ZoneConfig::new(name, weights));
    }
    let mut template = WorldTemplate::new(seed, zones);
    template.start_biome        = start_biome;
    template.start_biome_radius = start_biome_radius;
    let default_zone = template.zones.first()
        .map(|z| z.name.clone())
        .unwrap_or_else(|| "overworld".to_string());

    // Chunks
    let chunk_count = ru32(&mut r)? as usize;
    let mut chunks = HashMap::with_capacity(chunk_count);
    for _ in 0..chunk_count {
        let x          = ri16(&mut r)?;
        let z          = ri16(&mut r)?;
        let zone       = rstr(&mut r)?;
        let biome      = ri16(&mut r)?;
        let floor_rot  = ri16(&mut r)?;
        let floor_tex  = ri16(&mut r)?;
        let floor_model= ri16(&mut r)?;
        let mob_a      = rstr(&mut r)?;
        let mob_b      = rstr(&mut r)?;
        let elem_count = ru16(&mut r)? as usize;
        let mut elements = Vec::with_capacity(elem_count);
        for _ in 0..elem_count {
            elements.push(ChunkElement {
                cell_x:    ru8(&mut r)?,
                cell_z:    ru8(&mut r)?,
                rotation:  ru8(&mut r)?,
                item_data: rbytes(&mut r)?,
            });
        }
        chunks.insert((x, z), Chunk { x, z, zone, biome, floor_rot, floor_tex, floor_model, mob_a, mob_b, elements });
    }

    // Containers (reserved — skip count, nothing to read)
    let _container_count = ru32(&mut r)?;

    Ok(WorldState {
        name:         "World".to_string(),
        default_zone,
        chunks:  RwLock::new(chunks),
        players: RwLock::new(HashMap::new()),
        baskets: BasketStore::new(),
        zones: {
            let wgen = WorldGenerator::new(template.clone());
            let map = wgen.template_zones().map(|n| (n.to_string(), ZoneEntry::plain())).collect();
            RwLock::new(map)
        },
        generator:    WorldGenerator::new(template),
    })
}
