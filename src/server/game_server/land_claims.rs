//! Authoritative claims, independent of generated chunks and display names.
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Datelike, Timelike, Utc};

use super::world_state::WorldState;
use crate::defs::packet::pack_string;
use crate::utils::text::username_key;

/// Server-side sentinel, never interpreted as a calendar timestamp on the wire.
pub const PERMANENT_EXPIRY: u64 = u64::MAX;
pub const MAX_CLIENT_EXPIRY: u64 = 253_402_300_799; // .NET DateTime.MaxValue, whole seconds
pub const ADMIN_WIRE_DAYS: u64 = 32_000;

pub fn is_admin_claim(item_id: &str) -> bool {
    item_id == "Admin Land Claim"
}

pub fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClaimLocation {
    pub zone: String,
    pub cx: i16,
    pub cz: i16,
    pub ix: i16,
    pub iz: i16,
}

impl ClaimLocation {
    pub fn new(zone: &str, cx: i16, cz: i16, ix: i16, iz: i16) -> Option<Self> {
        if zone.is_empty() || zone.contains(',') || !(0..10).contains(&ix) || !(0..10).contains(&iz)
        {
            return None;
        }
        Some(Self {
            zone: zone.into(),
            cx,
            cz,
            ix,
            iz,
        })
    }

    pub fn key(&self) -> String {
        format!(
            "{},{},{},{},{}",
            self.zone, self.cx, self.cz, self.ix, self.iz
        )
    }

    pub fn parse(key: &str) -> Option<Self> {
        let mut parts = key.split(',');
        let result = Self::new(
            parts.next()?,
            parts.next()?.parse().ok()?,
            parts.next()?.parse().ok()?,
            parts.next()?.parse().ok()?,
            parts.next()?.parse().ok()?,
        )?;
        if parts.next().is_some() {
            return None;
        }
        Some(result)
    }

    pub fn covers(&self, zone: &str, cx: i16, cz: i16) -> bool {
        self.zone == zone
            && (i32::from(self.cx) - i32::from(cx)).abs() <= 1
            && (i32::from(self.cz) - i32::from(cz)).abs() <= 1
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LandClaim {
    pub location: ClaimLocation,
    pub owner: String,
    pub whitelist: [String; 2],
    pub expires_at: u64,
}

impl LandClaim {
    pub fn new(location: ClaimLocation, owner: &str, days: u64, now: u64) -> Self {
        Self {
            location,
            owner: username_key(owner),
            whitelist: Default::default(),
            expires_at: if days == ADMIN_WIRE_DAYS {
                PERMANENT_EXPIRY
            } else {
                now + days * 86_400
            },
        }
    }

    pub fn active(&self, now: u64) -> bool {
        !self.owner.is_empty() && (self.expires_at == PERMANENT_EXPIRY || self.expires_at > now)
    }

    pub fn allows(&self, username: &str) -> bool {
        let key = username_key(username);
        !key.is_empty()
            && (username_key(&self.owner) == key
                || self
                    .whitelist
                    .iter()
                    .any(|user| !user.is_empty() && username_key(user) == key))
    }

    pub fn owned_by(&self, username: &str) -> bool {
        let key = username_key(username);
        !key.is_empty() && username_key(&self.owner) == key
    }

    pub fn normalize(&mut self) {
        self.owner = username_key(&self.owner);
        self.whitelist = self.whitelist.each_ref().map(|user| username_key(user));
    }

    fn remaining(&self, now: u64) -> [i16; 4] {
        // The stock protocol has no permanent flag. Send its admin-claim timer
        // convention without letting that display timer expire server protection.
        if self.expires_at == PERMANENT_EXPIRY {
            return [ADMIN_WIRE_DAYS as i16, 0, 0, 0];
        }
        let seconds = self.expires_at.saturating_sub(now);
        [
            (seconds / 86_400).min(i16::MAX as u64) as i16,
            ((seconds % 86_400) / 3_600) as i16,
            ((seconds % 3_600) / 60) as i16,
            (seconds % 60) as i16,
        ]
    }

    /// ChunkData.UnpackFromWeb: key, three identities, relative D/H/M/S.
    pub fn pack_chunk(&self, out: &mut Vec<u8>, now: u64) {
        out.extend(pack_string(&self.location.key()));
        for name in [&self.owner, &self.whitelist[0], &self.whitelist[1]] {
            out.extend(pack_string(&username_key(name)));
        }
        for value in self.remaining(now) {
            out.extend(value.to_le_bytes());
        }
    }

    /// ZoneData.UnpackFromWeb: key, absolute S/M/H/D/month/year, identities.
    pub fn pack_zone(&self, out: &mut Vec<u8>) {
        out.extend(pack_string(&self.location.key()));
        let at = DateTime::<Utc>::from_timestamp(self.expires_at.min(MAX_CLIENT_EXPIRY) as i64, 0)
            .expect("validated claim expiry");
        for value in [
            at.second() as i16,
            at.minute() as i16,
            at.hour() as i16,
            at.day() as i16,
            at.month() as i16,
            at.year() as i16,
        ] {
            out.extend(value.to_le_bytes());
        }
        for name in [&self.owner, &self.whitelist[0], &self.whitelist[1]] {
            out.extend(pack_string(&username_key(name)));
        }
    }

    pub fn outside_add(&self, now: u64) -> Vec<u8> {
        let mut out = vec![36, 0];
        out.extend(pack_string(&self.location.key()));
        let [days, hours, minutes, seconds] = self.remaining(now);
        for value in [seconds, minutes, hours, days] {
            out.extend(value.to_le_bytes());
        }
        out.extend(pack_string(&username_key(&self.owner)));
        out
    }
}

/// Ordinary lifetimes, plus the client's admin timer convention. LandClaim::new
/// stores that admin value as PERMANENT_EXPIRY, never as a timed deadline.
pub fn claim_days(item_id: &str) -> Option<u64> {
    match item_id {
        "3-day Land Claim" => Some(3),
        "8-day Land Claim" => Some(8),
        "Admin Land Claim" => Some(ADMIN_WIRE_DAYS),
        _ => None,
    }
}

pub fn outside_edit(key: &str, slot: u8, user: &str) -> Vec<u8> {
    let mut out = vec![36, 1];
    out.extend(pack_string(key));
    out.push(slot);
    out.extend(pack_string(&username_key(user)));
    out
}

pub fn outside_remove(key: &str) -> Vec<u8> {
    let mut out = vec![36, 2];
    out.extend(pack_string(key));
    out
}

impl WorldState {
    /// Resolve a nested interior to the outermost world's entrance chunk.
    /// Unknown zones and cycles fail closed rather than bypassing protection.
    pub fn claim_root(&self, zone: &str, cx: i16, cz: i16) -> Option<(String, i16, i16)> {
        let zones = self.zones.read().unwrap();
        let mut zone = zone.to_string();
        let (mut x, mut z) = (cx, cz);
        let mut seen = HashSet::new();
        loop {
            if !seen.insert(zone.clone()) || seen.len() > 16 {
                return None;
            }
            let entry = zones.get(&zone)?;
            match &entry.interior {
                Some(interior) => {
                    zone = interior.outer_zone.clone();
                    x = interior.cx;
                    z = interior.cz;
                }
                None => return Some((zone, x, z)),
            }
        }
    }

    pub fn claims_at(&self, zone: &str, cx: i16, cz: i16, now: u64) -> Vec<LandClaim> {
        let Some((root, x, z)) = self.claim_root(zone, cx, cz) else {
            return Vec::new();
        };
        let mut result: Vec<_> = self
            .land_claims
            .read()
            .unwrap()
            .values()
            .filter(|claim| claim.active(now) && claim.location.covers(&root, x, z))
            .cloned()
            .collect();
        result.sort_by_key(|claim| claim.location.key());
        result
    }

    pub fn can_build(
        &self,
        zone: &str,
        cx: i16,
        cz: i16,
        user: &str,
        admin: bool,
        now: u64,
    ) -> bool {
        if username_key(user).is_empty() || self.claim_root(zone, cx, cz).is_none() {
            return false;
        }
        if admin {
            return true;
        }
        let claims = self.claims_at(zone, cx, cz, now);
        claims.is_empty() || claims.iter().any(|claim| claim.allows(user))
    }

    pub fn can_place_claim(&self, at: &ClaimLocation, user: &str, admin: bool, now: u64) -> bool {
        if at.zone != "overworld" {
            return false;
        }
        if admin {
            return true;
        }
        for dx in -1i32..=1 {
            for dz in -1i32..=1 {
                let (Ok(x), Ok(z)) = (
                    i16::try_from(i32::from(at.cx) + dx),
                    i16::try_from(i32::from(at.cz) + dz),
                ) else {
                    continue;
                };
                if !self.can_build(&at.zone, x, z, user, false, now) {
                    return false;
                }
            }
        }
        true
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::defs::packet::unpack_string;
    use crate::server::game_server::generator::{BiomeWeights, WorldTemplate, ZoneConfig};
    use crate::server::game_server::special_generators::ZoneKind;
    use crate::server::game_server::world_state::{InteriorData, ZoneEntry};

    pub(crate) fn world() -> WorldState {
        WorldState::new(
            "test",
            0,
            WorldTemplate::new(
                42,
                vec![ZoneConfig::new("overworld", BiomeWeights::default())],
            ),
        )
    }

    #[test]
    fn user_and_user2_have_identical_owner_and_whitelist_permissions() {
        let world = world();
        let at = ClaimLocation::new("overworld", 0, 0, 9, 9).unwrap();
        for name in ["User", "user2"] {
            let mut claim = LandClaim::new(at.clone(), name, 3, 100);
            assert!(claim.allows(name));
            assert!(claim.owned_by(&name.to_uppercase()));
            assert!(!claim.allows(""));
            claim.owner = "someone_else".into();
            for slot in 0..2 {
                claim.whitelist = Default::default();
                claim.whitelist[slot] = name.into();
                world
                    .land_claims
                    .write()
                    .unwrap()
                    .insert(at.key(), claim.clone());
                assert!(world.can_build("overworld", 1, -1, name, false, 101));
                assert!(!claim.owned_by(name));
                assert!(!world.can_build("overworld", 1, -1, "stranger", false, 101));
            }
        }
    }

    #[test]
    fn nine_chunks_protected_without_generation_and_with_negative_coordinates() {
        let world = world();
        let at = ClaimLocation::new("overworld", -2, -3, 0, 9).unwrap();
        world
            .land_claims
            .write()
            .unwrap()
            .insert(at.key(), LandClaim::new(at, "User", 8, 100));
        for x in -3..=-1 {
            for z in -4..=-2 {
                assert!(!world.can_build("overworld", x, z, "stranger", false, 101));
                assert!(world.can_build("overworld", x, z, "USER", false, 101));
            }
        }
        assert!(world.can_build("overworld", 0, -3, "stranger", false, 101));
        assert_eq!(world.chunks.read().unwrap()["overworld"].len(), 1);
    }

    #[test]
    fn overlap_matches_client_or_policy_and_two_chunk_spacing() {
        let world = world();
        let enemy = ClaimLocation::new("overworld", 0, 0, 0, 0).unwrap();
        world
            .land_claims
            .write()
            .unwrap()
            .insert(enemy.key(), LandClaim::new(enemy, "enemy", 3, 100));
        assert!(!world.can_place_claim(
            &ClaimLocation::new("overworld", 2, 2, 9, 9).unwrap(),
            "User",
            false,
            101
        ));
        assert!(world.can_place_claim(
            &ClaimLocation::new("overworld", 3, 0, 0, 0).unwrap(),
            "User",
            false,
            101
        ));
        let friendly = ClaimLocation::new("overworld", 0, 0, 1, 1).unwrap();
        world
            .land_claims
            .write()
            .unwrap()
            .insert(friendly.key(), LandClaim::new(friendly, "User", 3, 100));
        assert!(world.can_build("overworld", 1, 1, "user", false, 101));
        assert!(world.can_place_claim(
            &ClaimLocation::new("overworld", 2, 2, 9, 9).unwrap(),
            "User",
            false,
            101
        ));
    }

    #[test]
    fn nested_interiors_inherit_outermost_entrance_and_cycles_deny() {
        let world = world();
        let mut zones = world.zones.write().unwrap();
        for (name, parent, x, z) in [("shack1", "overworld", -5, 8), ("shack2", "shack1", 90, 90)] {
            zones.insert(
                name.into(),
                ZoneEntry::interior(InteriorData {
                    item_bytes: vec![],
                    rotation: 0,
                    cx: x,
                    cz: z,
                    tx: 0,
                    tz: 0,
                    outer_zone: parent.into(),
                    kind: ZoneKind::House,
                }),
            );
        }
        drop(zones);
        let at = ClaimLocation::new("overworld", -5, 8, 9, 9).unwrap();
        world
            .land_claims
            .write()
            .unwrap()
            .insert(at.key(), LandClaim::new(at, "User", 3, 100));
        assert!(!world.can_build("shack2", 90, -90, "stranger", false, 101));
        assert!(world.can_build("shack2", 90, -90, "USER", false, 101));
        world
            .zones
            .write()
            .unwrap()
            .get_mut("shack1")
            .unwrap()
            .interior
            .as_mut()
            .unwrap()
            .outer_zone = "shack2".into();
        assert!(!world.can_build("shack2", 0, 0, "User", true, 101));
        assert!(!world.can_build("missing", 0, 0, "User", false, 101));
    }

    #[test]
    fn expiry_is_exact_and_durations_fit_wire_shorts() {
        let at = ClaimLocation::new("overworld", 0, 0, 0, 0).unwrap();
        for (id, days) in [("3-day Land Claim", 3), ("8-day Land Claim", 8)] {
            assert_eq!(claim_days(id), Some(days));
            let claim = LandClaim::new(at.clone(), "User", days, 100);
            assert!(claim.active(claim.expires_at - 1));
            assert!(!claim.active(claim.expires_at));
            assert_eq!(claim.remaining(100), [days as i16, 0, 0, 0]);
        }
        let permanent = LandClaim::new(
            at.clone(),
            "User",
            claim_days("Admin Land Claim").unwrap(),
            100,
        );
        assert_eq!(permanent.expires_at, PERMANENT_EXPIRY);
        assert!(permanent.active(u64::MAX));
        assert_eq!(
            permanent.remaining(u64::MAX),
            [ADMIN_WIRE_DAYS as i16, 0, 0, 0]
        );
        let mut zone = Vec::new();
        permanent.pack_zone(&mut zone);
        let (_, offset) = unpack_string(&zone, 0);
        assert_eq!(
            i16::from_le_bytes(zone[offset + 10..offset + 12].try_into().unwrap()),
            9999
        );
        assert_eq!(claim_days("Old Land Claim"), None);
        let at = ClaimLocation::new("overworld", i16::MIN, i16::MAX, 0, 0).unwrap();
        assert!(!at.covers("overworld", i16::MAX, i16::MIN));
    }

    #[test]
    fn chunk_zone_and_outside_expiry_layouts_have_their_distinct_orders() {
        let at = ClaimLocation::new("overworld", 1, 2, 3, 4).unwrap();
        let mut claim = LandClaim::new(at, "User", 3, 1_700_000_000);
        claim.whitelist = ["USER2".into(), "".into()];
        let mut chunk = vec![];
        claim.pack_chunk(&mut chunk, 1_700_000_000);
        let (key, mut offset) = unpack_string(&chunk, 0);
        assert_eq!(key, "overworld,1,2,3,4");
        for expected in ["user", "user2", ""] {
            let (name, next) = unpack_string(&chunk, offset);
            offset = next;
            assert_eq!(name, expected);
        }
        assert_eq!(&chunk[offset..], &[3, 0, 0, 0, 0, 0, 0, 0]);

        let mut zone = vec![];
        claim.pack_zone(&mut zone);
        let (_, offset) = unpack_string(&zone, 0);
        let at = DateTime::<Utc>::from_timestamp(claim.expires_at as i64, 0).unwrap();
        let fields: Vec<_> = zone[offset..offset + 12]
            .chunks_exact(2)
            .map(|b| i16::from_le_bytes([b[0], b[1]]))
            .collect();
        assert_eq!(
            fields,
            [
                at.second() as i16,
                at.minute() as i16,
                at.hour() as i16,
                at.day() as i16,
                at.month() as i16,
                at.year() as i16
            ]
        );
        let (owner, _) = unpack_string(&zone, offset + 12);
        assert_eq!(owner, "user");

        let outside = claim.outside_add(1_700_000_000);
        assert_eq!(&outside[..2], &[36, 0]);
        let (_, offset) = unpack_string(&outside, 2);
        assert_eq!(&outside[offset..offset + 8], &[0, 0, 0, 0, 0, 0, 3, 0]);
        let (owner, end) = unpack_string(&outside, offset + 8);
        assert_eq!(owner, "user");
        assert_eq!(end, outside.len());
    }
}
