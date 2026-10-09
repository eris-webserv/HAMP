// persist.rs — binary world-state save/load.
//
// File: world.hws  (HAMP World State)
//
//  ─── Header ────────────────────────────────────────────────────────────────
//  [4]  magic:   b"HAMP"
//  [1]  version: u8 = 7 (reader accepts 1–7)
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
//    v3+: [2] per-chunk claim_count: u16 (0 in v6; legacy records on load)
//
//  ─── Containers (reserved) ─────────────────────────────────────────────────
//  [4]  container_count: u32 le = 0
//
//  ─── Teleporters (v5+) ─────────────────────────────────────────────────────
//  [4]  teleporter_count: u32 le
//  per teleporter:
//    [str] zone
//    [2]  cx: i16 le   [2] cz: i16 le   [2] tx: i16 le   [2] tz: i16 le
//    [str] title
//    [str] description
//    [str] built_by
//    [4]  screenshot_len: u32 le
//    [N]  screenshot bytes
//
//  (v4 stored per-chunk teleporter titles after land claims; those are
//  migrated into world-level entries with an empty built_by on load.)
//
//  ─── Canonical land claims (v6+) ──────────────────────────────────────────
//  [4] claim_count: u32 le
//  per claim: [str location_key][str owner][str whitelist1][str whitelist2]
//             [8] expires_at: u64 Unix seconds; v7 permits MAX for permanent claims

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::sync::{Mutex, RwLock};

use super::baskets::BasketStore;
use super::generator::{BiomeWeights, WorldGenerator, WorldTemplate, ZoneConfig};
use super::land_claims::{ClaimLocation, LandClaim, MAX_CLIENT_EXPIRY, PERMANENT_EXPIRY, is_admin_claim};
use super::world_mutations::Item;
use super::parse_shack_info;
use super::special_generators;
use super::world_state::{Chunk, ChunkElement, InteriorData, Teleporter, WorldState, ZoneEntry};

const MAGIC: &[u8; 4] = b"HAMP";

const VERSION: u8 = 7;
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
    let _mutation = state.mutation_lock.lock().unwrap();
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
    let total: u32 = chunks.values().map(|m| m.len() as u32).sum();
    wu32(w, total)?;
    for chunk in chunks.values().flat_map(|m| m.values()) {
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
        // v6 claims are canonical world records; keep the legacy chunk slot empty.
        wu16(w, 0)?;
    }

    // Containers (reserved)
    wu32(w, 0)?;

    // Teleporters
    let teles = state.teleporters.read().unwrap();
    wu32(w, teles.len() as u32)?;
    for t in teles.iter() {
        wstr(w, &t.zone)?;
        wi16(w, t.cx)?;
        wi16(w, t.cz)?;
        wi16(w, t.tx)?;
        wi16(w, t.tz)?;
        wstr(w, &t.title)?;
        wstr(w, &t.description)?;
        wstr(w, &t.built_by)?;
        wu32(w, t.screenshot.len() as u32)?;
        w.write_all(&t.screenshot)?;
    }

    // Canonical land claims (v6), saved once independently of generated chunks.
    let claims = state.land_claims.read().unwrap();
    wu32(w, claims.len() as u32)?;
    for claim in claims.values() {
        wstr(w, &claim.location.key())?;
        wstr(w, &claim.owner)?;
        wstr(w, &claim.whitelist[0])?;
        wstr(w, &claim.whitelist[1])?;
        wu64(w, claim.expires_at)?;
    }
    Ok(())
}

// ── Load ──────────────────────────────────────────────────────────────────

/// Reads a world state file and reconstructs a `WorldState`.
/// The generator is re-created from the saved template so that
/// chunks outside the saved radius can still be lazily generated.
pub fn load(path: &Path) -> io::Result<WorldState> {
    let file = File::open(path)?;
    read_state(BufReader::new(file))
}

fn read_state<R: Read>(mut r: R) -> io::Result<WorldState> {

    // Header
    let mut magic = [0u8; 4];
    r.read_exact(&mut magic)?;
    if &magic != MAGIC {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "not a HAMP world file"));
    }
    let version = ru8(&mut r)?;
    if !(1..=VERSION).contains(&version) {
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
    let mut chunks: HashMap<String, HashMap<(i16, i16), Chunk>> = HashMap::new();
    let mut teleporters: Vec<Teleporter> = Vec::new();
    let mut claims: HashMap<String, LandClaim> = HashMap::new();
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
        if version >= 3 {
            let claim_count = ru16(&mut r)? as usize;
            for _ in 0..claim_count {
                let claim_key       = rstr(&mut r)?;
                let user0           = rstr(&mut r)?;
                let user1           = rstr(&mut r)?;
                let user2           = rstr(&mut r)?;
                let expires_at_secs = ru64(&mut r)?;
                if let Some(location) = ClaimLocation::parse(&claim_key) {
                    if chrono::DateTime::<chrono::Utc>::from_timestamp(expires_at_secs as i64, 0).is_some()
                        && expires_at_secs <= MAX_CLIENT_EXPIRY {
                        let mut claim = LandClaim { location, owner: user0, whitelist: [user1, user2], expires_at: expires_at_secs };
                        claim.normalize();
                        // Collapse old duplicated chunk records without restarting expiry.
                        let entry = claims.entry(claim.location.key()).or_insert_with(|| claim.clone());
                        if claim.expires_at > entry.expires_at { *entry = claim; }
                    }
                }
            }
        }
        // v4 stored per-chunk teleporter titles; migrate to world-level entries.
        if version == 4 {
            let tele_count = ru16(&mut r)? as usize;
            for _ in 0..tele_count {
                let tx    = ru8(&mut r)?;
                let tz    = ru8(&mut r)?;
                let title = rstr(&mut r)?;
                let desc  = rstr(&mut r)?;
                teleporters.push(Teleporter {
                    title,
                    description: desc,
                    zone: zone.clone(),
                    cx: x, cz: z,
                    tx: tx as i16, tz: tz as i16,
                    built_by: String::new(),
                    screenshot: Vec::new(),
                });
            }
        }
        chunks.entry(zone.clone()).or_default()
            .insert((x, z), Chunk { x, z, zone, biome, floor_rot, floor_tex, floor_model, mob_a, mob_b, elements });
    }

    // Containers (reserved — skip count, nothing to read)
    let _container_count = ru32(&mut r)?;

    // Teleporters (v5+)
    if version >= 5 {
        let tele_count = ru32(&mut r)? as usize;
        for _ in 0..tele_count {
            let zone  = rstr(&mut r)?;
            let cx    = ri16(&mut r)?;
            let cz    = ri16(&mut r)?;
            let tx    = ri16(&mut r)?;
            let tz    = ri16(&mut r)?;
            let title = rstr(&mut r)?;
            let desc  = rstr(&mut r)?;
            let built_by = rstr(&mut r)?;
            let shot_len = ru32(&mut r)? as usize;
            let mut screenshot = vec![0u8; shot_len];
            r.read_exact(&mut screenshot)?;
            teleporters.push(Teleporter {
                title, description: desc, zone, cx, cz, tx, tz, built_by, screenshot,
            });
        }
    }

    if version >= 6 {
        let count = ru32(&mut r)?;
        for _ in 0..count {
            let key = rstr(&mut r)?;
            let owner = rstr(&mut r)?;
            let whitelist = [rstr(&mut r)?, rstr(&mut r)?];
            let expires_at = ru64(&mut r)?;
            let location = ClaimLocation::parse(&key)
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid claim location"))?;
            if expires_at > MAX_CLIENT_EXPIRY && !(version >= 7 && expires_at == PERMANENT_EXPIRY) {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid claim expiry"));
            }
            let mut claim = LandClaim { location, owner, whitelist, expires_at };
            claim.normalize();
            claims.insert(claim.location.key(), claim);
        }
    }

    // Upgrade existing Admin Land Claims too: their old 32,000-day deadline
    // must not remain an authoritative expiry after installing this version.
    for chunk in chunks.values_mut().flat_map(|zone| zone.values_mut()) {
        for element in &mut chunk.elements {
            let Some(mut item) = Item::decode(&element.item_data) else { continue; };
            if !is_admin_claim(item.id()) { continue; }
            let Some(at) = ClaimLocation::new(&chunk.zone, chunk.x, chunk.z, element.cell_x as i16, element.cell_z as i16) else { continue; };
            if let Some(claim) = claims.get_mut(&at.key()) {
                claim.expires_at = PERMANENT_EXPIRY;
                item.claim_expiry(PERMANENT_EXPIRY);
                element.item_data = item.encode();
            }
        }
    }

    // Rebuild the interior zone registry from saved chunk elements.
    // On first run this is done live as items are placed (0x20 handler).
    // On restart the chunks are loaded but ws.zones only has template zones,
    // so any shack zone entered after restart would return a blank interior.
    let wgen = WorldGenerator::new(template.clone());
    let mut zones: HashMap<String, ZoneEntry> = wgen.template_zones()
        .map(|n| (n.to_string(), ZoneEntry::plain()))
        .collect();

    for (zone_name, zone_map) in &chunks {
        for ((cx, cz), chunk) in zone_map {
            for el in &chunk.elements {
                if let Some((shack_id, item_id)) = parse_shack_info(&el.item_data) {
                    let shack_zone = format!("shack{}", shack_id);
                    let kind = special_generators::zone_kind_from_item_id(&item_id);
                    zones.entry(shack_zone.clone()).or_insert_with(|| {
                        ZoneEntry::interior(InteriorData {
                            item_bytes: el.item_data.clone(),
                            rotation:   el.rotation,
                            cx: *cx,
                            cz: *cz,
                            tx: el.cell_x as i16,
                            tz: el.cell_z as i16,
                            outer_zone: zone_name.clone(),
                            kind,
                        })
                    });
                }
            }
        }
    }

    Ok(WorldState {
        name:         "World".to_string(),
        default_zone,
        chunks:  RwLock::new(chunks),
        land_claims: RwLock::new(claims),
        mutation_lock: Mutex::new(()),
        players: RwLock::new(HashMap::new()),
        baskets: BasketStore::new(),
        zones:   RwLock::new(zones),
        teleporters: RwLock::new(teleporters),
        generator: WorldGenerator::new(template),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::land_claims::{tests::world, unix_now};
    use std::io::Cursor;

    #[test]
    fn canonical_claims_survive_restart_without_resetting_expiry_or_loading_neighbors() {
        let world = world(); let at = ClaimLocation::new("overworld", -4, 7, 2, 8).unwrap();
        let mut claim = LandClaim::new(at.clone(), "User", 8, unix_now());
        claim.whitelist = ["User2".into(), "GUEST".into()];
        world.land_claims.write().unwrap().insert(at.key(), claim.clone());
        let mut bytes = Vec::new(); write_state(&world, &mut bytes).unwrap();
        assert_eq!(bytes[4], VERSION);
        let restored = read_state(Cursor::new(bytes)).unwrap();
        let saved = restored.land_claims.read().unwrap()[&at.key()].clone();
        assert_eq!(saved.owner, "user"); assert_eq!(saved.whitelist, ["user2", "guest"]);
        assert_eq!(saved.expires_at, claim.expires_at);
        assert!(restored.can_build("overworld", -3, 8, "USER2", false, unix_now()));
        assert!(!restored.can_build("overworld", -3, 8, "enemy", false, unix_now()));
        assert!(!restored.chunks.read().unwrap()["overworld"].contains_key(&(-3, 8)));
        let packet = restored.get_chunk_wire("overworld", -3, 8);
        assert!(packet.windows(crate::defs::packet::pack_string(&at.key()).len())
            .any(|slice| slice == crate::defs::packet::pack_string(&at.key())));
    }

    fn legacy(version: u8, copies: u32, expiry: u64) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(MAGIC); wu8(&mut bytes, version).unwrap(); wu64(&mut bytes, 42).unwrap();
        if version >= 2 { wi16(&mut bytes, 0).unwrap(); wi16(&mut bytes, 0).unwrap(); }
        wu16(&mut bytes, 1).unwrap(); wstr(&mut bytes, "overworld").unwrap();
        if version >= 2 { for _ in 0..8 { bytes.extend(1f32.to_le_bytes()); } } else { bytes.extend([1u8; 8]); }
        wu32(&mut bytes, copies).unwrap();
        for cx in 0..copies {
            wi16(&mut bytes, cx as i16).unwrap(); wi16(&mut bytes, 0).unwrap(); wstr(&mut bytes, "overworld").unwrap();
            for value in [0, 0, 1, 0] { wi16(&mut bytes, value).unwrap(); }
            wstr(&mut bytes, "").unwrap(); wstr(&mut bytes, "").unwrap(); wu16(&mut bytes, 0).unwrap();
            if version >= 3 {
                wu16(&mut bytes, 1).unwrap();
                for value in ["overworld,0,0,4,5", "User", "USER2", ""] { wstr(&mut bytes, value).unwrap(); }
                wu64(&mut bytes, expiry).unwrap();
            }
            if version == 4 { wu16(&mut bytes, 0).unwrap(); }
        }
        wu32(&mut bytes, 0).unwrap(); // containers
        if version >= 5 { wu32(&mut bytes, 0).unwrap(); } // teleporters
        bytes
    }

    #[test]
    fn legacy_world_versions_load_and_duplicated_claims_collapse() {
        let expiry = unix_now() + 86_400;
        for version in 1..=5 {
            let state = read_state(Cursor::new(legacy(version, 3, expiry))).unwrap();
            assert_eq!(state.chunks.read().unwrap()["overworld"].len(), 3);
            let claims = state.land_claims.read().unwrap();
            if version < 3 { assert!(claims.is_empty()); continue; }
            assert_eq!(claims.len(), 1);
            let claim = &claims["overworld,0,0,4,5"];
            assert_eq!(claim.owner, "user"); assert_eq!(claim.whitelist, ["user2", ""]);
            assert_eq!(claim.expires_at, expiry);
        }
    }

    #[test]
    fn malformed_claim_expiry_is_rejected_before_client_calendar_conversion() {
        let world = world(); let at = ClaimLocation::new("overworld", 0, 0, 4, 5).unwrap();
        let mut claim = LandClaim::new(at.clone(), "User", 3, unix_now()); claim.expires_at = u64::MAX - 1;
        world.land_claims.write().unwrap().insert(at.key(), claim);
        let mut bytes = Vec::new(); write_state(&world, &mut bytes).unwrap();
        assert!(read_state(Cursor::new(bytes)).is_err());
    }

    #[test]
    fn old_admin_deadlines_become_permanent_and_survive_another_save() {
        let world = world(); let at = ClaimLocation::new("overworld", 0, 0, 4, 5).unwrap();
        let mut claim = LandClaim::new(at.clone(), "User", 3, unix_now());
        claim.expires_at = unix_now() - 1; // even a previously due admin deadline is not retained
        claim.whitelist = ["User2".into(), "".into()];
        world.land_claims.write().unwrap().insert(at.key(), claim);
        world.chunks.write().unwrap().get_mut("overworld").unwrap().get_mut(&(0, 0)).unwrap().elements.push(ChunkElement {
            cell_x: 4, cell_z: 5, rotation: 2, item_data: Item::named("Admin Land Claim").encode(),
        });
        let mut old_bytes = Vec::new(); write_state(&world, &mut old_bytes).unwrap(); old_bytes[4] = 6;
        let migrated = read_state(Cursor::new(old_bytes)).unwrap();
        assert_eq!(migrated.land_claims.read().unwrap()[&at.key()].expires_at, PERMANENT_EXPIRY);
        let chunks = migrated.chunks.read().unwrap();
        let item = Item::decode(&chunks["overworld"][&(0, 0)].elements.last().unwrap().item_data).unwrap();
        assert_eq!(item.id(), "Admin Land Claim"); drop(chunks);
        let mut bytes = Vec::new(); write_state(&migrated, &mut bytes).unwrap();
        assert_eq!(bytes[4], 7);
        let restored = read_state(Cursor::new(bytes)).unwrap();
        assert!(restored.can_build("overworld", 1, 1, "USER2", false, u64::MAX));
        assert!(!restored.can_build("overworld", 1, 1, "stranger", false, u64::MAX));
        assert_eq!(restored.land_claims.read().unwrap()[&at.key()].expires_at, PERMANENT_EXPIRY);
    }
}
