// baskets.rs — basket (container) persistence for managed game servers.
//
// File: baskets.hwb  (HAMP World Baskets)
//
// Stored separately from world.hws so the basket set can grow and be saved
// independently of chunk state.
//
//  ─── Header ────────────────────────────────────────────────────────────────
//  [4]  magic:   b"HAMB"
//  [1]  version: u8 = 1
//
//  ─── Baskets ───────────────────────────────────────────────────────────────
//  [4]  basket_count: u32 le
//  per basket:
//    [8]  basket_id:    i64 le
//    [4]  contents_len: u32 le
//    [N]  contents:     raw `BasketContents` wire bytes
//
// `contents` is stored verbatim in the client's wire format:
//   [2]  n_items: i16 le
//   n_items × {
//     [2]  slot:  i16 le
//     [2]  count: i16 le
//     [?]  InventoryItem (UnpackFromWeb blob: 3 typed sections)
//   }
//
// Storing raw bytes avoids re-parsing on save/load; the blob is opaque to
// the server and only forwarded to clients.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::sync::RwLock;

use super::packets_client::skip_basket_contents;

const MAGIC: &[u8; 4] = b"HAMB";
const VERSION: u8 = 1;
pub const FILE_NAME: &str = "baskets.hwb";

/// Empty `BasketContents` wire blob: `i16(0)` (no items).
const EMPTY_CONTENTS: [u8; 2] = [0, 0];

// ── Store ─────────────────────────────────────────────────────────────────

/// In-memory basket store keyed by basket unique ID.
///
/// Each value is the raw `BasketContents` wire blob (excluding the basket_id
/// prefix) — server stores it verbatim and replays it to any client that
/// opens the basket.
pub struct BasketStore {
    baskets: RwLock<HashMap<i64, Vec<u8>>>,
}

impl BasketStore {
    pub fn new() -> Self {
        Self { baskets: RwLock::new(HashMap::new()) }
    }

    /// Returns this basket's `BasketContents` wire bytes, or an empty
    /// container (i16 zero) if the basket has never been saved.
    pub fn get_contents(&self, id: i64) -> Vec<u8> {
        self.baskets.read().unwrap()
            .get(&id)
            .cloned()
            .unwrap_or_else(|| EMPTY_CONTENTS.to_vec())
    }

    /// Overwrites the stored contents for this basket.
    pub fn put(&self, id: i64, contents: &[u8]) {
        self.baskets.write().unwrap().insert(id, contents.to_vec());
    }

    /// Number of stored baskets (for logging).
    pub fn len(&self) -> usize {
        self.baskets.read().unwrap().len()
    }
}

// ── Save ──────────────────────────────────────────────────────────────────

/// Writes the full basket store to `path` atomically (write-then-rename).
pub fn save(store: &BasketStore, path: &Path) -> io::Result<()> {
    let tmp = path.with_extension("hwb.tmp");
    {
        let file = File::create(&tmp)?;
        let mut w = BufWriter::new(file);
        write_store(store, &mut w)?;
        w.flush()?;
    }
    fs::rename(&tmp, path)
}

fn write_store<W: Write>(store: &BasketStore, w: &mut W) -> io::Result<()> {
    w.write_all(MAGIC)?;
    w.write_all(&[VERSION])?;

    let baskets = store.baskets.read().unwrap();
    w.write_all(&(baskets.len() as u32).to_le_bytes())?;
    for (id, contents) in baskets.iter() {
        w.write_all(&id.to_le_bytes())?;
        w.write_all(&(contents.len() as u32).to_le_bytes())?;
        w.write_all(contents)?;
    }
    Ok(())
}

// ── Load ──────────────────────────────────────────────────────────────────

/// Reads a basket file and returns the populated store.
///
/// A blob that fails the `BasketContents` skip check (truncated / corrupt)
/// is dropped with a warning rather than aborting the whole load.
pub fn load(path: &Path) -> io::Result<BasketStore> {
    let file = File::open(path)?;
    let mut r = BufReader::new(file);

    let mut magic = [0u8; 4];
    r.read_exact(&mut magic)?;
    if &magic != MAGIC {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "not a HAMP basket file"));
    }
    let mut vb = [0u8; 1];
    r.read_exact(&mut vb)?;
    if vb[0] != VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unsupported basket file version {}", vb[0]),
        ));
    }

    let mut count_bytes = [0u8; 4];
    r.read_exact(&mut count_bytes)?;
    let count = u32::from_le_bytes(count_bytes) as usize;

    let mut baskets = HashMap::with_capacity(count);
    for _ in 0..count {
        let mut id_bytes  = [0u8; 8];
        let mut len_bytes = [0u8; 4];
        r.read_exact(&mut id_bytes)?;
        r.read_exact(&mut len_bytes)?;
        let id  = i64::from_le_bytes(id_bytes);
        let len = u32::from_le_bytes(len_bytes) as usize;
        let mut buf = vec![0u8; len];
        r.read_exact(&mut buf)?;

        // Best-effort sanity check: skip through the contents and confirm
        // the blob is at least as long as one valid BasketContents parse.
        let end = skip_basket_contents(&buf, 0);
        if end > buf.len() {
            eprintln!("[BASKETS] basket {} has malformed contents (len {}), skipping", id, len);
            continue;
        }
        baskets.insert(id, buf);
    }

    Ok(BasketStore { baskets: RwLock::new(baskets) })
}
