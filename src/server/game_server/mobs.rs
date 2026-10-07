use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::defs::packet::{pack_string, unpack_string};

/// Clients simulate wildlife; the server arbitrates the single AI owner.
#[derive(Default)]
pub(super) struct Wildlife {
    mobs: HashMap<String, Mob>,
}

struct Mob {
    owner: Option<String>,
    zone: String,
    respawn_at: Option<Instant>,
}

pub(super) fn read_id(data: &[u8], offset: usize) -> Option<(String, usize)> {
    let (id, end) = unpack_string(data, offset);
    (end > offset && !id.is_empty()).then_some((id, end))
}

impl Wildlife {
    pub fn claim(&mut self, user: &str, zone: &str, data: &[u8]) -> Option<Vec<u8>> {
        let count = *data.first()?;
        let mut offset = 1;
        let mut ids = Vec::new();
        for _ in 0..count {
            let (id, end) = read_id(data, offset)?;
            offset = end;
            ids.push(id);
        }
        if offset != data.len() { return None; }
        let mut reply = vec![0x3f, count];
        for id in ids {
            let mob = self.mobs.entry(id.clone()).or_insert_with(|| Mob {
                owner: None, zone: zone.to_owned(), respawn_at: None,
            });
            let available = mob.zone == zone && mob.owner.is_none()
                && mob.respawn_at.is_none_or(|time| Instant::now() >= time);
            if available {
                mob.owner = Some(user.to_owned());
                mob.respawn_at = None;
            }
            reply.extend(pack_string(&id));
            reply.push(u8::from(available));
        }
        Some(reply)
    }

    pub fn release(&mut self, user: &str, id: &str) -> Option<String> {
        let mob = self.mobs.get_mut(id)?;
        if mob.owner.as_deref() != Some(user) { return None; }
        mob.owner = None;
        Some(mob.zone.clone())
    }

    pub fn inherit(&mut self, user: &str, zone: &str, id: &str) -> bool {
        let Some(mob) = self.mobs.get_mut(id) else { return false; };
        if mob.zone != zone || mob.owner.is_some() || mob.respawn_at.is_some() { return false; }
        mob.owner = Some(user.to_owned());
        true
    }

    pub fn release_player(&mut self, user: &str) -> Vec<(String, String)> {
        self.mobs.iter_mut().filter_map(|(id, mob)| {
            if mob.owner.as_deref() != Some(user) { return None; }
            mob.owner = None;
            Some((id.clone(), mob.zone.clone()))
        }).collect()
    }

    pub fn died(&mut self, id: &str, seconds: u64) {
        if let Some(mob) = self.mobs.get_mut(id) {
            mob.owner = None;
            mob.respawn_at = Some(Instant::now() + Duration::from_secs(seconds));
        }
    }
}
