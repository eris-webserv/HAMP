//! Client-owned creatures, including personal companions. AI and companion
//! inventory remain on the owning client; this table arbitrates simulation.
use std::collections::HashMap;

use crate::defs::packet::{pack_string, unpack_string};

#[derive(Default)]
pub(super) struct Mobs {
    owners: HashMap<String, Owner>,
    positions: HashMap<String, Vec<u8>>,
}

struct Owner {
    player: String,
    local: bool,
}

fn string(data: &[u8], off: &mut usize) -> Option<String> {
    let (value, next) = unpack_string(data, *off);
    if next == *off || (next - *off - 2) % 2 != 0 { return None; }
    *off = next;
    Some(value)
}

pub(super) fn single_id(data: &[u8]) -> Option<String> {
    let mut off = 0;
    let id = string(data, &mut off)?;
    (off == data.len() && !id.is_empty()).then_some(id)
}

impl Mobs {
    /// 0x3F uses a byte count, not a short. Parse the entire request before
    /// assigning anything so a truncated request cannot leave phantom owners.
    pub fn claim(&mut self, player: &str, body: &[u8]) -> Option<Vec<u8>> {
        let count = *body.first()?;
        let mut off = 1;
        let mut ids = Vec::new();
        for _ in 0..count { ids.push(string(body, &mut off)?); }
        if off != body.len() { return None; }
        let mut response = vec![0x3F, count];
        for id in ids {
            // Even the current owner must not spawn a second copy.
            let allowed = !id.is_empty() && !self.owners.contains_key(&id);
            response.extend(pack_string(&id));
            response.push(u8::from(allowed));
            if allowed {
                self.owners.insert(id, Owner { player: player.into(), local: false });
            }
        }
        Some(response)
    }

    /// 0x59 declares an already-spawned local companion/minion. Repeated
    /// announcements are harmless; another player cannot take its ownership.
    pub fn register(&mut self, player: &str, id: String) {
        self.owners.entry(id).and_modify(|owner| {
            if owner.player == player { owner.local = true; }
        }).or_insert(Owner { player: player.into(), local: true });
    }

    pub fn release(&mut self, player: &str, id: &str) {
        if self.owners.get(id).is_some_and(|owner| owner.player == player) {
            self.owners.remove(id);
            // A cached batch may include the released creature. Discard it;
            // the next movement update supplies a fresh list.
            self.positions.remove(player);
        }
    }

    /// Keep valid movement snapshots for newcomers, preserving the original
    /// player-prefixed 0x41 format. Positions and rotation are each 8 bytes.
    pub fn remember_positions(&mut self, player: &str, body: &[u8]) {
        let Some(&count) = body.first() else { return; };
        let mut off = 1;
        for _ in 0..count {
            let Some(id) = string(body, &mut off) else { return; };
            if off + 24 > body.len() || !self.owners.get(&id).is_some_and(|o| o.player == player) {
                return;
            }
            off += 24;
        }
        if off != body.len() { return; }
        let mut packet = vec![0x41];
        packet.extend(pack_string(player));
        packet.extend_from_slice(body);
        self.positions.insert(player.into(), packet);
    }

    pub fn positions(&self, player: &str) -> Option<Vec<u8>> {
        self.positions.get(player).cloned()
    }

    pub fn ids(&self, player: &str) -> Vec<String> {
        let mut ids: Vec<_> = self.owners.iter().filter(|(_, o)| o.player == player)
            .map(|(id, _)| id.clone()).collect();
        ids.sort();
        ids
    }

    /// Companions follow their owner across zones. Claimed world mobs do not.
    pub fn leave_zone(&mut self, player: &str) {
        self.owners.retain(|_, o| o.player != player || o.local);
        self.positions.remove(player);
    }

    pub fn disconnect(&mut self, player: &str) {
        self.owners.retain(|_, o| o.player != player);
        self.positions.remove(player);
    }
}

/// PlayerGone already has a creature list in the client's protocol. Include
/// owned companions here so they are removed when their owner goes away.
pub(super) fn gone_packets(player: &str, ids: &[String]) -> Vec<Vec<u8>> {
    let groups: Vec<&[String]> = if ids.is_empty() { vec![&[]] } else { ids.chunks(255).collect() };
    groups.into_iter().map(|group| {
        let mut packet = vec![0x13, 0];
        packet.extend(pack_string(player));
        packet.push(group.len() as u8);
        for id in group { packet.extend(pack_string(id)); }
        packet
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(ids: &[&str]) -> Vec<u8> {
        let mut body = vec![ids.len() as u8];
        for id in ids { body.extend(pack_string(id)); }
        body
    }

    #[test]
    fn companion_cannot_be_claimed_or_released_by_another_player() {
        let mut mobs = Mobs::default();
        mobs.register("alice", "pet".into());
        mobs.register("bob", "pet".into());
        mobs.release("bob", "pet");
        assert_eq!(mobs.ids("alice"), ["pet"]);
        assert_eq!(mobs.claim("bob", &request(&["pet"])).unwrap().last(), Some(&0));
        mobs.release("alice", "pet");
        assert_eq!(mobs.claim("bob", &request(&["pet"])).unwrap().last(), Some(&1));
    }

    #[test]
    fn malformed_claims_are_atomic_and_duplicates_spawn_only_once() {
        let mut mobs = Mobs::default();
        let mut broken = request(&["guard", "wolf"]);
        broken.pop();
        assert!(mobs.claim("alice", &broken).is_none());
        assert!(mobs.ids("alice").is_empty());
        let result = mobs.claim("alice", &request(&["guard", "guard", ""])).unwrap();
        let mut expected = vec![0x3F, 3];
        for (id, allowed) in [("guard", 1), ("guard", 0), ("", 0)] {
            expected.extend(pack_string(id));
            expected.push(allowed);
        }
        assert_eq!(result, expected);
        assert!(single_id(&[1, 0, b'x']).is_none());
    }

    #[test]
    fn companions_follow_across_zones_but_world_mobs_are_released() {
        let mut mobs = Mobs::default();
        mobs.register("alice", "pet".into());
        mobs.claim("alice", &request(&["guard"])).unwrap();
        let mut positions = request(&["pet"]);
        positions.extend([0u8; 24]);
        mobs.remember_positions("alice", &positions);
        assert!(mobs.positions("alice").is_some());
        mobs.leave_zone("alice");
        assert_eq!(mobs.ids("alice"), ["pet"]);
        assert!(mobs.positions("alice").is_none());
        assert_eq!(mobs.claim("bob", &request(&["guard"])).unwrap().last(), Some(&1));
        mobs.disconnect("alice");
        assert!(mobs.ids("alice").is_empty());
    }

    #[test]
    fn movement_cache_rejects_truncation_and_foreign_mobs() {
        let mut mobs = Mobs::default();
        mobs.register("alice", "pet".into());
        let mut body = request(&["pet"]);
        body.extend([7u8; 24]);
        mobs.remember_positions("alice", &body);
        let cached = mobs.positions("alice").unwrap();
        assert_eq!(cached[0], 0x41);
        let (owner, off) = unpack_string(&cached, 1);
        assert_eq!(owner, "alice");
        assert_eq!(&cached[off..], &body);
        mobs.remember_positions("bob", &body);
        assert!(mobs.positions("bob").is_none());
        body.pop();
        mobs.remember_positions("alice", &body);
        assert_eq!(mobs.positions("alice").unwrap(), cached);
        mobs.release("alice", "pet");
        assert!(mobs.positions("alice").is_none());
    }

    #[test]
    fn gone_lists_include_every_owned_mob_even_above_byte_count_limit() {
        let ids: Vec<_> = (0..256).map(|i| format!("mob{i}")).collect();
        let packets = gone_packets("alice", &ids);
        assert_eq!(packets.len(), 2);
        let mut found = Vec::new();
        for packet in packets {
            assert_eq!(&packet[..2], &[0x13, 0]);
            let (owner, mut off) = unpack_string(&packet, 2);
            assert_eq!(owner, "alice");
            let count = packet[off];
            off += 1;
            for _ in 0..count { found.push(string(&packet, &mut off).unwrap()); }
            assert_eq!(off, packet.len());
        }
        assert_eq!(found, ids);
    }
}
