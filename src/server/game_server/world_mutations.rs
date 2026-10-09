//! Parse mutation packets once, validate before changing state or broadcasting.
use std::collections::BTreeMap;

use chrono::{DateTime, Utc};

use super::land_claims::{
    ClaimLocation, LandClaim, MAX_CLIENT_EXPIRY, claim_days, is_admin_claim, outside_edit,
    outside_remove, unix_now,
};
use super::world_state::{Chunk, ChunkElement, InteriorData, WorldState, ZoneEntry};
use super::{InteriorInfo, Session, SessionMode, ZoneData, parse_shack_info, special_generators};
use crate::defs::packet::{ServerPacket, pack_string};
use crate::utils::text::username_key;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Item {
    shorts: BTreeMap<String, i16>,
    strings: BTreeMap<String, String>,
    ints: BTreeMap<String, i32>,
}

impl Item {
    pub(super) fn id(&self) -> &str {
        self.strings
            .get("item_id")
            .map(String::as_str)
            .unwrap_or("")
    }

    pub(super) fn decode(bytes: &[u8]) -> Option<Self> {
        let mut reader = Reader { bytes, at: 0 };
        let item = reader.item()?;
        (reader.at == bytes.len()).then_some(item)
    }

    pub(super) fn named(name: &str) -> Self {
        Self {
            strings: BTreeMap::from([("item_id".into(), name.into())]),
            ..Self::default()
        }
    }

    pub(super) fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend((self.shorts.len() as i16).to_le_bytes());
        for (key, value) in &self.shorts {
            out.extend(pack_string(key));
            out.extend(value.to_le_bytes());
        }
        out.extend((self.strings.len() as i16).to_le_bytes());
        for (key, value) in &self.strings {
            out.extend(pack_string(key));
            out.extend(pack_string(value));
        }
        out.extend((self.ints.len() as i16).to_le_bytes());
        for (key, value) in &self.ints {
            out.extend(pack_string(key));
            out.extend(value.to_le_bytes());
        }
        out
    }

    pub(super) fn claim_expiry(&mut self, expires_at: u64) {
        let at = DateTime::<Utc>::from_timestamp(expires_at.min(MAX_CLIENT_EXPIRY) as i64, 0)
            .expect("claim expiry");
        self.strings
            .insert("has_respawn_landclaim_spawn".into(), "true".into());
        self.strings.insert(
            "UTC_dateTime_landclaim_spawn".into(),
            at.format("%Y-%m-%dT%H:%M:%S.0000000Z").to_string(),
        );
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl Reader<'_> {
    fn take(&mut self, size: usize) -> Option<&[u8]> {
        let end = self.at.checked_add(size)?;
        let result = self.bytes.get(self.at..end)?;
        self.at = end;
        Some(result)
    }
    fn byte(&mut self) -> Option<u8> {
        Some(self.take(1)?[0])
    }
    fn short(&mut self) -> Option<i16> {
        Some(i16::from_le_bytes(self.take(2)?.try_into().ok()?))
    }
    fn count(&mut self) -> Option<usize> {
        usize::try_from(self.short()?).ok()
    }
    fn string(&mut self) -> Option<String> {
        let length = self.count()?;
        if length % 2 != 0 {
            return None;
        }
        let bytes = self.take(length)?;
        String::from_utf16(
            &bytes
                .chunks_exact(2)
                .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
                .collect::<Vec<_>>(),
        )
        .ok()
    }
    fn item(&mut self) -> Option<Item> {
        let mut item = Item::default();
        let count = self.count()?;
        for _ in 0..count {
            let key = self.string()?;
            let value = self.short()?;
            if item.shorts.insert(key, value).is_some() {
                return None;
            }
        }
        let count = self.count()?;
        for _ in 0..count {
            let key = self.string()?;
            let value = self.string()?;
            if item.strings.insert(key, value).is_some() {
                return None;
            }
        }
        let count = self.count()?;
        for _ in 0..count {
            let key = self.string()?;
            let value = i32::from_le_bytes(self.take(4)?.try_into().ok()?);
            if item.ints.insert(key, value).is_some() {
                return None;
            }
        }
        Some(item)
    }
    fn location(&mut self) -> Option<ClaimLocation> {
        ClaimLocation::new(
            &self.string()?,
            self.short()?,
            self.short()?,
            self.short()?,
            self.short()?,
        )
    }
}

#[derive(Clone, Debug)]
enum Action {
    Build(Item, u8, String),
    Remove(Item, u8, String),
    Replace {
        new: Item,
        old: Item,
        rotation: u8,
        cache: String,
    },
    Whitelist {
        slot: u8,
        user: String,
        keys: [String; 9],
    },
}

#[derive(Clone, Debug)]
struct Mutation {
    location: ClaimLocation,
    action: Action,
}

impl Mutation {
    fn places_admin_claim(&self) -> bool {
        match &self.action {
            Action::Build(item, ..) => is_admin_claim(item.id()),
            Action::Replace { new, old, .. } => {
                is_admin_claim(new.id()) && !is_admin_claim(old.id())
            }
            _ => false,
        }
    }

    fn parse(opcode: u8, bytes: &[u8]) -> Option<Self> {
        let mut reader = Reader { bytes, at: 0 };
        let result = match opcode {
            32 => {
                reader.string()?; // validator, never an identity or whitelist authorization
                let item = reader.item()?;
                let rotation = reader.byte()?;
                let location = reader.location()?;
                let cache = reader.string()?;
                Self {
                    location,
                    action: Action::Build(item, rotation, cache),
                }
            }
            33 => {
                reader.string()?;
                let location = reader.location()?;
                let rotation = reader.byte()?;
                let item = reader.item()?;
                let cache = reader.string()?;
                Self {
                    location,
                    action: Action::Remove(item, rotation, cache),
                }
            }
            34 => {
                reader.string()?;
                let new = reader.item()?;
                let old = reader.item()?;
                let rotation = reader.byte()?;
                let location = reader.location()?;
                let cache = reader.string()?;
                Self {
                    location,
                    action: Action::Replace {
                        new,
                        old,
                        rotation,
                        cache,
                    },
                }
            }
            35 => {
                // This opcode has NO validator or player-name prefix.
                let location = reader.location()?;
                let slot = reader.byte()?;
                if !(1..=2).contains(&slot) {
                    return None;
                }
                let user = username_key(&reader.string()?);
                if user.chars().any(|c| c.is_control() || c == '<' || c == '>') {
                    return None;
                }
                let mut keys: [String; 9] = Default::default();
                for key in &mut keys {
                    *key = reader.string()?;
                }
                Self {
                    location,
                    action: Action::Whitelist { slot, user, keys },
                }
            }
            _ => return None,
        };
        (reader.at == bytes.len()).then_some(result)
    }

    fn packet(&self, builder: &str) -> Vec<u8> {
        let mut out = Vec::new();
        match &self.action {
            Action::Build(item, rotation, cache) => {
                out.push(32);
                out.extend(item.encode());
                out.push(*rotation);
                pack_location(&mut out, &self.location);
                out.extend(pack_string(&username_key(builder)));
                out.extend(pack_string(cache));
            }
            Action::Remove(item, rotation, cache) => {
                out.push(33);
                pack_location(&mut out, &self.location);
                out.push(*rotation);
                out.extend(item.encode());
                out.extend(pack_string(cache));
            }
            Action::Replace {
                new,
                old,
                rotation,
                cache,
            } => {
                out.push(34);
                out.extend(new.encode());
                out.extend(old.encode());
                out.push(*rotation);
                pack_location(&mut out, &self.location);
                out.extend(pack_string(cache));
            }
            Action::Whitelist { slot, user, keys } => {
                out.push(35);
                pack_location(&mut out, &self.location);
                out.push(*slot);
                out.extend(pack_string(&username_key(user)));
                for key in keys {
                    out.extend(pack_string(key));
                }
            }
        }
        out
    }
}

fn pack_location(out: &mut Vec<u8>, at: &ClaimLocation) {
    out.extend(pack_string(&at.zone));
    for coordinate in [at.cx, at.cz, at.ix, at.iz] {
        out.extend(coordinate.to_le_bytes());
    }
}

enum ClaimChange {
    Add(LandClaim),
    Edit(ClaimLocation, u8, String),
    Remove(ClaimLocation),
}

struct Applied {
    peers: Vec<Vec<u8>>,
    origin: Vec<Vec<u8>>,
    claim: Option<ClaimChange>,
}

fn apply(
    world: &WorldState,
    mut mutation: Mutation,
    user: &str,
    admin: bool,
    now: u64,
) -> Result<Applied, &'static str> {
    if mutation.places_admin_claim() && !admin {
        return Err("admin claim requires server administrator permission");
    }
    let at = &mutation.location;
    if world.claim_root(&at.zone, at.cx, at.cz).is_none() {
        return Err("unknown zone");
    }
    if !world
        .chunks
        .read()
        .unwrap()
        .get(&at.zone)
        .is_some_and(|zone| zone.contains_key(&(at.cx, at.cz)))
    {
        world.get_chunk_wire(&at.zone, at.cx, at.cz);
    }
    let _guard = world.mutation_lock.lock().unwrap();
    let key = at.key();
    let mut result = Applied {
        peers: Vec::new(),
        origin: Vec::new(),
        claim: None,
    };
    if let Action::Whitelist {
        slot,
        user: trusted,
        ..
    } = &mutation.action
    {
        let mut claims = world.land_claims.write().unwrap();
        let claim = claims
            .get_mut(&key)
            .filter(|claim| claim.active(now))
            .ok_or("claim is inactive")?;
        if !claim.owned_by(user) && !admin {
            return Err("only the claim owner can edit its whitelist");
        }
        claim.whitelist[usize::from(*slot - 1)] = username_key(trusted);
        result.claim = Some(ClaimChange::Edit(at.clone(), *slot, trusted.clone()));
        // Echo canonical identity to the editor too: their local edit used raw input casing.
        result.origin.push(mutation.packet(user));
        result.peers.push(mutation.packet(user));
        return Ok(result);
    }

    if !world.can_build(&at.zone, at.cx, at.cz, user, admin, now) {
        return Err("land is claimed by another player");
    }

    // Generate the target chunk through the existing world generator, without creating
    // placeholder neighbors or overwriting their terrain. No mutation lock recursion.
    // Every build target should already have been requested by a normal client.
    let mut chunks = world.chunks.write().unwrap();
    let zone = chunks.entry(at.zone.clone()).or_default();
    let chunk = zone
        .entry((at.cx, at.cz))
        .or_insert_with(|| Chunk::blank(at.cx, at.cz, &at.zone));
    let matching = |item: &Item, rotation: u8, chunk: &Chunk| {
        chunk.elements.iter().position(|element| {
            element.cell_x == at.ix as u8
                && element.cell_z == at.iz as u8
                && element.rotation == rotation
                && Item::decode(&element.item_data).as_ref() == Some(item)
        })
    };

    match &mut mutation.action {
        Action::Build(item, rotation, _) => {
            if item.id().is_empty() {
                return Err("missing item identity");
            }
            if matching(item, *rotation, chunk).is_some() {
                return Err("object already exists");
            }
            if let Some(days) = claim_days(item.id()) {
                if !world.can_place_claim(at, user, admin, now) {
                    return Err("land claim would overlap foreign land");
                }
                let mut claims = world.land_claims.write().unwrap();
                if claims.get(&key).is_some_and(|claim| claim.active(now)) {
                    return Err("claim already exists");
                }
                let claim = LandClaim::new(at.clone(), user, days, now);
                let original = item.clone();
                item.claim_expiry(claim.expires_at);
                if original != *item {
                    result.origin.push(
                        Mutation {
                            location: at.clone(),
                            action: Action::Replace {
                                new: item.clone(),
                                old: original,
                                rotation: *rotation,
                                cache: String::new(),
                            },
                        }
                        .packet(user),
                    );
                }
                claims.insert(key.clone(), claim.clone());
                result.claim = Some(ClaimChange::Add(claim));
            }
            chunk.elements.push(ChunkElement {
                cell_x: at.ix as u8,
                cell_z: at.iz as u8,
                rotation: *rotation,
                item_data: item.encode(),
            });
        }
        Action::Remove(item, rotation, _) => {
            let position = matching(item, *rotation, chunk).ok_or("object no longer matches")?;
            chunk.elements.remove(position);
            if claim_days(item.id()).is_some() {
                world.land_claims.write().unwrap().remove(&key);
                result.claim = Some(ClaimChange::Remove(at.clone()));
            }
        }
        Action::Replace {
            new, old, rotation, ..
        } => {
            if new.id().is_empty() {
                return Err("missing item identity");
            }
            let position = match matching(old, *rotation, chunk) {
                Some(position) => position,
                None if claim_days(old.id()).is_some()
                    && new.id() == "Old Land Claim"
                    && matching(new, *rotation, chunk).is_some() =>
                {
                    return Ok(result);
                } // expiry already applied
                None => return Err("object no longer matches"),
            };
            if claim_days(old.id()).is_some() {
                let claim = world.land_claims.read().unwrap().get(&key).cloned();
                if new.id() == "Old Land Claim" {
                    if claim.as_ref().is_some_and(|claim| claim.active(now)) {
                        return Err("claim has not expired");
                    }
                    world.land_claims.write().unwrap().remove(&key);
                    result.claim = Some(ClaimChange::Remove(at.clone()));
                } else {
                    if new.id() != old.id() {
                        return Err("cannot replace an active claim with another item");
                    }
                    let claim = claim.ok_or("claim is inactive")?;
                    let original = new.clone();
                    new.claim_expiry(claim.expires_at);
                    if original != *new {
                        result.origin.push(
                            Mutation {
                                location: at.clone(),
                                action: Action::Replace {
                                    new: new.clone(),
                                    old: original,
                                    rotation: *rotation,
                                    cache: String::new(),
                                },
                            }
                            .packet(user),
                        );
                    }
                }
            } else if claim_days(new.id()).is_some() {
                return Err("claims must be placed through the build operation");
            }
            chunk.elements[position].item_data = new.encode();
        }
        Action::Whitelist { .. } => unreachable!(),
    }
    drop(chunks);

    // Keep the interior registry and teleporter listing in step with object mutations.
    match &mutation.action {
        Action::Build(item, rotation, _)
        | Action::Replace {
            new: item,
            rotation,
            ..
        } => {
            let item_bytes = item.encode();
            if let Some((id, name)) = parse_shack_info(&item_bytes) {
                let mut zones = world.zones.write().unwrap();
                let zone_name = format!("shack{id}");
                if let Some(existing) = zones
                    .get_mut(&zone_name)
                    .and_then(|entry| entry.interior.as_mut())
                {
                    existing.item_bytes = item_bytes;
                    existing.rotation = *rotation;
                    existing.cx = at.cx;
                    existing.cz = at.cz;
                    existing.tx = at.ix;
                    existing.tz = at.iz;
                    existing.outer_zone = at.zone.clone();
                } else {
                    zones.insert(
                        zone_name,
                        ZoneEntry::interior(InteriorData {
                            kind: special_generators::zone_kind_from_item_id(&name),
                            item_bytes,
                            rotation: *rotation,
                            cx: at.cx,
                            cz: at.cz,
                            tx: at.ix,
                            tz: at.iz,
                            outer_zone: at.zone.clone(),
                        }),
                    );
                }
            }
        }
        Action::Remove(item, ..) if item.id() == "Teleporter" => {
            world.remove_teleporter(&at.zone, at.cx, at.cz, at.ix, at.iz)
        }
        _ => {}
    }
    result.peers.push(mutation.packet(user));
    Ok(result)
}

/// Called while Session.world_updates is held, so clients observe mutation order.
fn notify_claim(session: &Session, world: &WorldState, change: &ClaimChange, now: u64) {
    let at = match change {
        ClaimChange::Add(claim) => &claim.location,
        ClaimChange::Edit(at, ..) | ClaimChange::Remove(at) => at,
    };
    let packet = match change {
        ClaimChange::Add(claim) => claim.outside_add(now),
        ClaimChange::Edit(at, slot, user) => outside_edit(&at.key(), *slot, user),
        ClaimChange::Remove(at) => outside_remove(&at.key()),
    };
    let recipients: Vec<_> = session
        .players
        .lock()
        .unwrap()
        .iter()
        .filter_map(|(name, player)| {
            let zone = player.zone.lock().unwrap().clone();
            (zone != at.zone
                && world
                    .claim_root(&zone, 0, 0)
                    .is_some_and(|(root, cx, cz)| at.covers(&root, cx, cz)))
            .then(|| name.clone())
        })
        .collect();
    for recipient in recipients {
        session.send_to(&recipient, &packet);
    }
}

pub(super) fn zone_body(world: &WorldState, zone: &str) -> Vec<u8> {
    let claims = world.claims_at(zone, 0, 0, unix_now());
    let zones = world.zones.read().unwrap();
    let interior = zones
        .get(zone)
        .and_then(|entry| entry.interior.as_ref())
        .map(|entry| InteriorInfo {
            item_bytes: &entry.item_bytes,
            rotation: entry.rotation,
            cx: entry.cx,
            cz: entry.cz,
            tx: entry.tx,
            tz: entry.tz,
            outer_zone: &entry.outer_zone,
        });
    let reply = ZoneData {
        zone_name: zone,
        interior,
        claims: &claims,
    }
    .to_payload();
    // Strip opcode, result/map-change flags, and zone-name wrapper, retaining
    // exactly the ZoneData.UnpackFromWeb body (without the trailing transition).
    let start = 3 + pack_string(zone).len();
    reply[start..reply.len() - 1].to_vec()
}

fn rollback(world: &WorldState, mutation: &Mutation, user: &str) -> Option<Vec<u8>> {
    let at = &mutation.location;
    let chunks = world.chunks.read().unwrap();
    let chunk = chunks.get(&at.zone)?.get(&(at.cx, at.cz))?;
    let contains = |item: &Item, rotation: u8| {
        chunk.elements.iter().any(|element| {
            element.cell_x == at.ix as u8
                && element.cell_z == at.iz as u8
                && element.rotation == rotation
                && Item::decode(&element.item_data).as_ref() == Some(item)
        })
    };
    let action = match &mutation.action {
        Action::Build(item, rotation, _) => Action::Remove(item.clone(), *rotation, String::new()),
        Action::Remove(item, rotation, _) if contains(item, *rotation) => {
            Action::Build(item.clone(), *rotation, String::new())
        }
        Action::Replace {
            new, old, rotation, ..
        } if contains(old, *rotation) => Action::Replace {
            new: old.clone(),
            old: new.clone(),
            rotation: *rotation,
            cache: String::new(),
        },
        _ => return None,
    };
    Some(
        Mutation {
            location: at.clone(),
            action,
        }
        .packet(user),
    )
}

fn correct(session: &Session, world: &WorldState, user: &str, mutation: &Mutation) {
    let at = &mutation.location;
    // A full snapshot replaces data but doesn't rebuild an already-complete
    // chunk in the stock client. Undo its optimistic visible change first.
    if let Some(packet) = rollback(world, mutation, user) {
        session.send_to(user, &packet);
    }
    for dx in -1i32..=1 {
        for dz in -1i32..=1 {
            let (Ok(x), Ok(z)) = (
                i16::try_from(i32::from(at.cx) + dx),
                i16::try_from(i32::from(at.cz) + dz),
            ) else {
                continue;
            };
            session.send_to(user, &world.get_chunk_wire(&at.zone, x, z));
        }
    }
    if at.zone != "overworld" {
        let mut packet = vec![37];
        packet.extend(zone_body(world, &at.zone));
        session.send_to(user, &packet);
    }
}

/// True tells the connection handler to stop dispatching and disconnect the player.
pub(super) fn handle(session: &Session, user: &str, opcode: u8, bytes: &[u8]) -> bool {
    let Some(mutation) = Mutation::parse(opcode, bytes) else {
        return false;
    };
    let _updates = session.world_updates.lock().unwrap();
    if mutation.places_admin_claim() && !session.is_admin(user) {
        // The client already placed the object optimistically. Undo it before
        // disconnecting, without generating, persisting, or relaying the placement.
        let undo = match &mutation.action {
            Action::Build(item, rotation, _) => {
                Some(Action::Remove(item.clone(), *rotation, String::new()))
            }
            Action::Replace {
                new, old, rotation, ..
            } => Some(Action::Replace {
                new: old.clone(),
                old: new.clone(),
                rotation: *rotation,
                cache: String::new(),
            }),
            _ => None,
        };
        if let Some(action) = undo {
            session.send_to(
                user,
                &Mutation {
                    location: mutation.location.clone(),
                    action,
                }
                .packet(user),
            );
        }
        super::kick_player(
            session,
            user,
            "Only server administrators may place Admin Land Claims.",
        );
        return true;
    }
    match &session.mode {
        SessionMode::Relay => session.broadcast(&mutation.packet(user), Some(user)),
        SessionMode::Managed(world) => {
            let player_zone = session
                .players
                .lock()
                .unwrap()
                .get(user)
                .map(|player| player.zone.lock().unwrap().clone());
            if player_zone.as_deref() != Some(&mutation.location.zone) {
                return false;
            }
            let now = unix_now();
            expire_locked(session, world, now);
            match apply(world, mutation.clone(), user, session.is_admin(user), now) {
                Ok(result) => {
                    for packet in result.peers {
                        session.broadcast(&packet, Some(user));
                    }
                    for packet in result.origin {
                        session.send_to(user, &packet);
                    }
                    if let Some(change) = result.claim {
                        notify_claim(session, world, &change, now);
                    }
                }
                Err(reason) => {
                    eprintln!("[GAME] Rejected mutation from {user}: {reason}");
                    correct(session, world, user, &mutation);
                }
            }
        }
    }
    false
}

pub(super) fn expire(session: &Session, now: u64) {
    let SessionMode::Managed(world) = &session.mode else {
        return;
    };
    let _updates = session.world_updates.lock().unwrap();
    expire_locked(session, world, now);
}

fn expire_locked(session: &Session, world: &WorldState, now: u64) {
    let mut packets = Vec::new();
    let expired = {
        let _mutation = world.mutation_lock.lock().unwrap();
        let mut claims = world.land_claims.write().unwrap();
        let expired: Vec<_> = claims
            .values()
            .filter(|claim| !claim.active(now))
            .cloned()
            .collect();
        let mut chunks = world.chunks.write().unwrap();
        for claim in &expired {
            let at = &claim.location;
            claims.remove(&at.key());
            if let Some(chunk) = chunks
                .get_mut(&at.zone)
                .and_then(|zone| zone.get_mut(&(at.cx, at.cz)))
            {
                for element in &mut chunk.elements {
                    if element.cell_x != at.ix as u8 || element.cell_z != at.iz as u8 {
                        continue;
                    }
                    let Some(old) = Item::decode(&element.item_data) else {
                        continue;
                    };
                    if claim_days(old.id()).is_none() {
                        continue;
                    }
                    let new = Item::named("Old Land Claim");
                    element.item_data = new.encode();
                    packets.push(
                        Mutation {
                            location: at.clone(),
                            action: Action::Replace {
                                new,
                                old,
                                rotation: element.rotation,
                                cache: String::new(),
                            },
                        }
                        .packet(""),
                    );
                }
            }
        }
        expired
    };
    for packet in packets {
        session.broadcast(&packet, None);
    }
    for claim in expired {
        notify_claim(session, world, &ClaimChange::Remove(claim.location), now);
    }
}

#[cfg(test)]
mod tests {
    use super::super::land_claims::tests::world;
    use super::*;
    use std::sync::Arc;

    fn at(cx: i16, cz: i16) -> ClaimLocation {
        ClaimLocation::new("overworld", cx, cz, 4, 5).unwrap()
    }
    fn build(location: ClaimLocation, id: &str) -> Mutation {
        Mutation {
            location,
            action: Action::Build(Item::named(id), 2, "cache".into()),
        }
    }
    fn stored(world: &WorldState, at: &ClaimLocation, id: &str) -> Item {
        let chunks = world.chunks.read().unwrap();
        chunks[&at.zone][&(at.cx, at.cz)]
            .elements
            .iter()
            .filter(|el| el.cell_x == at.ix as u8 && el.cell_z == at.iz as u8)
            .filter_map(|el| Item::decode(&el.item_data))
            .find(|item| item.id() == id)
            .unwrap()
    }
    fn request(mutation: &Mutation) -> Vec<u8> {
        let packet = mutation.packet("User");
        if let Action::Whitelist { .. } = mutation.action {
            return packet[1..].to_vec();
        }
        let mut bytes = pack_string("validator");
        if let Action::Build(_, _, cache) = &mutation.action {
            // Remove the server-only builder identity, retaining the final cache string.
            bytes.extend(
                &packet[1..packet.len() - pack_string(cache).len() - pack_string("user").len()],
            );
            bytes.extend(pack_string(cache));
        } else {
            bytes.extend(&packet[1..]);
        }
        bytes
    }

    #[test]
    fn mutation_packets_round_trip_and_keep_exact_client_layouts() {
        let actions = [
            build(at(-1, 2), "3-day Land Claim"),
            Mutation {
                location: at(-1, 2),
                action: Action::Remove(Item::named("Stone"), 3, "cache".into()),
            },
            Mutation {
                location: at(-1, 2),
                action: Action::Replace {
                    new: Item::named("Old Land Claim"),
                    old: Item::named("3-day Land Claim"),
                    rotation: 1,
                    cache: "cache".into(),
                },
            },
            Mutation {
                location: at(-1, 2),
                action: Action::Whitelist {
                    slot: 2,
                    user: "user".into(),
                    keys: std::array::from_fn(|i| format!("key{i}")),
                },
            },
        ];
        for mutation in actions {
            let outgoing = mutation.packet("User");
            let bytes = request(&mutation);
            let parsed = Mutation::parse(outgoing[0], &bytes).unwrap();
            assert_eq!(parsed.location, mutation.location);
            assert_eq!(parsed.packet("User"), outgoing);
            for length in 0..bytes.len() {
                assert!(Mutation::parse(outgoing[0], &bytes[..length]).is_none());
            }
            let mut extra = bytes.clone();
            extra.push(0);
            assert!(Mutation::parse(outgoing[0], &extra).is_none());
            if outgoing[0] != 35 {
                assert!(Mutation::parse(outgoing[0], &outgoing[1..]).is_none());
            }
        }
    }

    #[test]
    fn malformed_fields_cannot_be_used_as_authorization() {
        let mutation = Mutation {
            location: at(0, 0),
            action: Action::Whitelist {
                slot: 1,
                user: "USER".into(),
                keys: Default::default(),
            },
        };
        let parsed = Mutation::parse(35, &request(&mutation)).unwrap();
        assert!(matches!(parsed.action, Action::Whitelist { user, .. } if user == "user"));
        let mut invalid = mutation.clone();
        invalid.action = Action::Whitelist {
            slot: 0,
            user: "user".into(),
            keys: Default::default(),
        };
        assert!(Mutation::parse(35, &request(&invalid)).is_none());
        let mut bytes = request(&mutation);
        bytes[0] = 1;
        assert!(Mutation::parse(35, &bytes).is_none()); // odd UTF16 length
        assert!(ClaimLocation::new("overworld", 0, 0, 10, 0).is_none());
    }

    #[test]
    fn item_metadata_preserves_separate_typed_dictionaries() {
        let mut item = Item::named("Stone");
        item.shorts.insert("shared_key".into(), 12);
        item.strings.insert("shared_key".into(), "value".into());
        item.ints.insert("shared_key".into(), 123456);
        assert_eq!(Item::decode(&item.encode()), Some(item));
    }

    #[test]
    fn user_and_user2_build_edit_and_remove_through_the_authoritative_path() {
        for owner in ["User", "user2"] {
            let world = world();
            let anchor = at(-2, 3);
            let now = unix_now();
            let added = apply(
                &world,
                build(anchor.clone(), "3-day Land Claim"),
                owner,
                false,
                now,
            )
            .unwrap();
            assert_eq!(added.peers[0][0], 32);
            assert_eq!(added.origin[0][0], 34); // server-controlled expiry replaces optimistic metadata
            let claim = world.land_claims.read().unwrap()[&anchor.key()].clone();
            assert_eq!(claim.owner, owner.to_lowercase());
            assert_eq!(claim.expires_at, now + 3 * 86_400);
            let item = stored(&world, &anchor, "3-day Land Claim");
            assert_eq!(item.strings["has_respawn_landclaim_spawn"], "true");
            let date = DateTime::parse_from_rfc3339(&item.strings["UTC_dateTime_landclaim_spawn"])
                .unwrap();
            assert_eq!(date.timestamp() as u64, claim.expires_at);
            for slot in [1, 2] {
                apply(
                    &world,
                    Mutation {
                        location: anchor.clone(),
                        action: Action::Whitelist {
                            slot,
                            user: "GUEST".into(),
                            keys: Default::default(),
                        },
                    },
                    &owner.to_uppercase(),
                    false,
                    now,
                )
                .unwrap();
                assert!(world.can_build("overworld", -1, 4, "Guest", false, now));
                let mut neighbour = at(-1, 4);
                neighbour.ix = slot as i16;
                apply(&world, build(neighbour, "Stone"), "Guest", false, now).unwrap();
            }
            let edit = Mutation {
                location: anchor.clone(),
                action: Action::Whitelist {
                    slot: 1,
                    user: "enemy".into(),
                    keys: Default::default(),
                },
            };
            assert!(apply(&world, edit, "guest", false, now).is_err());
            assert!(apply(&world, build(at(-2, 4), "Wood"), "enemy", false, now).is_err());
            apply(
                &world,
                Mutation {
                    location: anchor.clone(),
                    action: Action::Remove(item, 2, "cache".into()),
                },
                "GUEST",
                false,
                now,
            )
            .unwrap();
            assert!(world.land_claims.read().unwrap().is_empty()); // whitelist drill permission
        }
    }

    #[test]
    fn unauthorized_mutations_leave_objects_and_claims_unchanged_and_have_inverses() {
        let world = world();
        let anchor = at(0, 0);
        let now = unix_now();
        apply(
            &world,
            build(anchor.clone(), "8-day Land Claim"),
            "User",
            false,
            now,
        )
        .unwrap();
        let item = stored(&world, &anchor, "8-day Land Claim");
        let before = world.chunks.read().unwrap()["overworld"][&(0, 0)].to_wire();
        let actions = [
            build(anchor.clone(), "Stone"),
            Mutation {
                location: anchor.clone(),
                action: Action::Remove(item.clone(), 2, "cache".into()),
            },
            Mutation {
                location: anchor.clone(),
                action: Action::Replace {
                    new: Item::named("Old Land Claim"),
                    old: item,
                    rotation: 2,
                    cache: "cache".into(),
                },
            },
        ];
        for (mutation, opcode) in actions.into_iter().zip([33, 32, 34]) {
            assert!(apply(&world, mutation.clone(), "enemy", false, now).is_err());
            assert_eq!(
                world.chunks.read().unwrap()["overworld"][&(0, 0)].to_wire(),
                before
            );
            assert_eq!(rollback(&world, &mutation, "enemy").unwrap()[0], opcode);
        }
    }

    #[test]
    fn exact_object_matching_prevents_deleting_an_unrelated_stack_element() {
        let world = world();
        let anchor = at(0, 0);
        let now = unix_now();
        apply(&world, build(anchor.clone(), "Stone"), "user", false, now).unwrap();
        apply(&world, build(anchor.clone(), "Wood"), "user", false, now).unwrap();
        let missing = Mutation {
            location: anchor.clone(),
            action: Action::Remove(Item::named("Chair"), 2, String::new()),
        };
        assert!(apply(&world, missing, "user", false, now).is_err());
        assert_eq!(stored(&world, &anchor, "Stone").id(), "Stone");
        assert_eq!(stored(&world, &anchor, "Wood").id(), "Wood");
    }

    #[test]
    fn incoming_mutations_expire_due_posts_before_the_next_background_tick() {
        let world = Arc::new(world());
        let anchor = at(0, 0);
        apply(
            &world,
            build(anchor.clone(), "3-day Land Claim"),
            "User",
            false,
            unix_now() - 3 * 86_400 - 1,
        )
        .unwrap();
        let session = Session::new(
            "test",
            SessionMode::Managed(world.clone()),
            false,
            false,
            vec![],
            false,
        );
        let mut owner = attach(&session, "user", "overworld");
        // The client already sees an Old post when the server watchdog hasn't ticked.
        let mutation = Mutation {
            location: anchor.clone(),
            action: Action::Remove(Item::named("Old Land Claim"), 2, String::new()),
        };
        handle(&session, "user", 33, &request(&mutation));
        assert_eq!(receive(&mut owner)[0], 34); // preflight expiry
        assert!(world.land_claims.read().unwrap().is_empty());
        let chunks = world.chunks.read().unwrap();
        assert!(
            !chunks["overworld"][&(0, 0)].elements.iter().any(|element| {
                element.cell_x == anchor.ix as u8
                    && element.cell_z == anchor.iz as u8
                    && Item::decode(&element.item_data).is_some_and(|item| {
                        claim_days(item.id()).is_some() || item.id() == "Old Land Claim"
                    })
            })
        );
    }

    #[test]
    fn foreign_overlap_admin_placement_and_expiry_are_enforced() {
        let world = Arc::new(world());
        let anchor = at(0, 0);
        let now = unix_now();
        apply(
            &world,
            build(anchor.clone(), "3-day Land Claim"),
            "User",
            false,
            now,
        )
        .unwrap();
        assert!(
            apply(
                &world,
                build(at(2, 2), "3-day Land Claim"),
                "enemy",
                false,
                now
            )
            .is_err()
        );
        assert!(
            apply(
                &world,
                build(at(3, 3), "Admin Land Claim"),
                "enemy",
                false,
                now
            )
            .is_err()
        );
        apply(
            &world,
            build(at(3, 3), "Admin Land Claim"),
            "admin",
            true,
            now,
        )
        .unwrap();
        let old = stored(&world, &anchor, "3-day Land Claim");
        let automatic = Mutation {
            location: anchor.clone(),
            action: Action::Replace {
                new: Item::named("Old Land Claim"),
                old,
                rotation: 2,
                cache: String::new(),
            },
        };
        assert!(apply(&world, automatic.clone(), "User", false, now).is_err());
        let session = Session::new(
            "test",
            SessionMode::Managed(world.clone()),
            false,
            false,
            vec![],
            false,
        );
        expire(&session, now + 3 * 86_400 - 1);
        assert!(
            world
                .land_claims
                .read()
                .unwrap()
                .contains_key(&anchor.key())
        );
        expire(&session, now + 3 * 86_400);
        assert!(
            !world
                .land_claims
                .read()
                .unwrap()
                .contains_key(&anchor.key())
        );
        assert_eq!(
            stored(&world, &anchor, "Old Land Claim").id(),
            "Old Land Claim"
        );
        assert!(
            apply(&world, automatic, "User", false, now + 3 * 86_400)
                .unwrap()
                .peers
                .is_empty()
        );
        expire(&session, now + 3 * 86_400); // repeated ticks remain idempotent
        assert_eq!(world.land_claims.read().unwrap().len(), 1);
    }

    #[test]
    fn concurrent_foreign_claims_cannot_race_the_overlap_check() {
        let world = Arc::new(world());
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let threads: Vec<_> = [(0, "User"), (2, "user2")]
            .into_iter()
            .map(|(cx, owner)| {
                let world = world.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    apply(
                        &world,
                        build(at(cx, 0), "3-day Land Claim"),
                        owner,
                        false,
                        unix_now(),
                    )
                    .is_ok()
                })
            })
            .collect();
        assert_eq!(
            threads
                .into_iter()
                .filter_map(|thread| thread.join().ok())
                .filter(|ok| *ok)
                .count(),
            1
        );
        assert_eq!(world.land_claims.read().unwrap().len(), 1);
    }

    fn attach(session: &Session, user: &str, zone: &str) -> std::net::TcpStream {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let client = std::net::TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let (sink, _) = listener.accept().unwrap();
        client
            .set_read_timeout(Some(std::time::Duration::from_secs(2)))
            .unwrap();
        session.players.lock().unwrap().insert(
            user.into(),
            Arc::new(super::super::GamePlayer {
                display_name: std::sync::Mutex::new(super::super::player_names::DisplayName::new(user)),
                sink: std::sync::Mutex::new(sink),
                initial_data: std::sync::Mutex::new(None),
                zone: std::sync::Mutex::new(zone.into()),
            }),
        );
        client
    }
    fn receive(client: &mut std::net::TcpStream) -> Vec<u8> {
        use std::io::Read;
        let mut header = [0u8; 9];
        client.read_exact(&mut header).unwrap();
        assert_eq!(header[4], 3); // test packets fit one frame
        let length = u32::from_le_bytes(header[5..].try_into().unwrap()) as usize;
        let mut packet = vec![0; length];
        client.read_exact(&mut packet).unwrap();
        packet
    }

    #[test]
    fn unauthorized_admin_placement_is_undone_and_kicked_before_any_world_change() {
        use std::io::Read;
        for relay in [false, true] {
            let world = Arc::new(world());
            let mode = if relay {
                SessionMode::Relay
            } else {
                SessionMode::Managed(world.clone())
            };
            let session = Session::new("test", mode, false, false, vec![], false);
            let mut offender = attach(&session, "user", "overworld");
            let mut observer = attach(&session, "user2", "overworld");
            let before = world.chunks.read().unwrap()["overworld"].len();
            assert!(handle(
                &session,
                "user",
                32,
                &request(&build(at(50, 50), "Admin Land Claim"))
            ));
            assert_eq!(receive(&mut offender)[0], 33);
            assert_eq!(offender.read(&mut [0u8; 1]).unwrap(), 0); // queued rollback followed by FIN
            assert_eq!(world.chunks.read().unwrap()["overworld"].len(), before);
            assert!(world.land_claims.read().unwrap().is_empty());
            observer
                .set_read_timeout(Some(std::time::Duration::from_millis(50)))
                .unwrap();
            assert!(observer.read(&mut [0u8; 1]).is_err()); // no placement broadcast
        }
    }

    #[test]
    fn configured_admin_claims_never_expire_and_cannot_be_expired_by_a_client() {
        let config: crate::utils::config::Config =
            toml::from_str("admin_users = ['USER']").unwrap();
        let world = Arc::new(world());
        let session = Session::new(
            "test",
            SessionMode::Managed(world.clone()),
            false,
            false,
            config.admin_users,
            false,
        );
        let mut admin = attach(&session, "user", "overworld");
        let anchor = at(0, 0);
        assert!(session.is_admin("User"));
        assert!(!handle(
            &session,
            "user",
            32,
            &request(&build(anchor.clone(), "Admin Land Claim"))
        ));
        assert_eq!(receive(&mut admin)[0], 34);
        let claim = world.land_claims.read().unwrap()[&anchor.key()].clone();
        assert_eq!(
            claim.expires_at,
            super::super::land_claims::PERMANENT_EXPIRY
        );
        expire(&session, u64::MAX);
        assert!(world.can_build("overworld", 1, 1, "USER", false, u64::MAX));
        assert!(!world.can_build("overworld", 1, 1, "stranger", false, u64::MAX));
        let item = stored(&world, &anchor, "Admin Land Claim");
        assert_eq!(
            item.strings["UTC_dateTime_landclaim_spawn"],
            "9999-12-31T23:59:59.0000000Z"
        );
        let automatic = Mutation {
            location: anchor.clone(),
            action: Action::Replace {
                new: Item::named("Old Land Claim"),
                old: item.clone(),
                rotation: 2,
                cache: String::new(),
            },
        };
        assert!(apply(&world, automatic, "user", true, u64::MAX).is_err());
        assert_eq!(stored(&world, &anchor, "Admin Land Claim"), item);
    }

    #[test]
    fn converting_an_ordinary_item_into_an_admin_claim_also_kicks() {
        let world = Arc::new(world());
        let session = Session::new(
            "test",
            SessionMode::Managed(world.clone()),
            false,
            false,
            vec![],
            false,
        );
        let mut offender = attach(&session, "user", "overworld");
        let anchor = at(0, 0);
        apply(
            &world,
            build(anchor.clone(), "Stone"),
            "user",
            false,
            unix_now(),
        )
        .unwrap();
        let mutation = Mutation {
            location: anchor.clone(),
            action: Action::Replace {
                new: Item::named("Admin Land Claim"),
                old: Item::named("Stone"),
                rotation: 2,
                cache: String::new(),
            },
        };
        assert!(handle(&session, "user", 34, &request(&mutation)));
        assert_eq!(receive(&mut offender)[0], 34); // inverse replacement
        assert_eq!(stored(&world, &anchor, "Stone").id(), "Stone");
        assert!(world.land_claims.read().unwrap().is_empty());
    }

    #[test]
    fn kicked_connection_cannot_dispatch_more_mutations_in_the_same_batch() {
        use std::io::{Read, Write};
        let world = Arc::new(world());
        let session = Session::new(
            "test",
            SessionMode::Managed(world.clone()),
            false,
            false,
            vec![],
            false,
        );
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let mut client = std::net::TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        client
            .set_read_timeout(Some(std::time::Duration::from_secs(3)))
            .unwrap();
        let (stream, address) = listener.accept().unwrap();
        let handler_session = session.clone();
        let handler = std::thread::spawn(move || {
            super::super::handle_client(stream, address, handler_session, None)
        });
        let mut login = vec![38];
        login.extend(pack_string("test"));
        login.extend(pack_string("user"));
        let mut packets = crate::defs::packet::craft_batch(2, &login);
        for (opcode, mutation) in [
            (32, build(at(50, 50), "Admin Land Claim")),
            (32, build(at(51, 51), "Stone")),
        ] {
            let mut payload = vec![opcode];
            payload.extend(request(&mutation));
            packets.extend(crate::defs::packet::craft_batch(2, &payload));
        }
        client.write_all(&packets).unwrap();
        let mut response = Vec::new();
        client.read_to_end(&mut response).unwrap();
        handler.join().unwrap();
        assert!(!response.is_empty());
        assert!(session.players.lock().unwrap().is_empty());
        assert!(world.land_claims.read().unwrap().is_empty());
        assert_eq!(world.chunks.read().unwrap()["overworld"].len(), 1);
    }

    #[test]
    fn stock_wire_flow_updates_indoor_clients_and_corrects_rejected_optimistic_edits() {
        use super::super::special_generators::ZoneKind;
        let world = Arc::new(world());
        world.zones.write().unwrap().insert(
            "shack1".into(),
            ZoneEntry::interior(InteriorData {
                kind: ZoneKind::House,
                item_bytes: Item::named("Shack").encode(),
                rotation: 0,
                cx: 0,
                cz: 0,
                tx: 1,
                tz: 1,
                outer_zone: "overworld".into(),
            }),
        );
        let session = Session::new(
            "test",
            SessionMode::Managed(world.clone()),
            false,
            false,
            vec![],
            false,
        );
        let mut owner = attach(&session, "user", "overworld");
        let mut guest = attach(&session, "user2", "shack1");
        let anchor = at(0, 0);
        handle(
            &session,
            "user",
            32,
            &request(&build(anchor.clone(), "3-day Land Claim")),
        );
        assert_eq!(receive(&mut owner)[0], 34); // corrective timestamp, no duplicate build
        assert_eq!(receive(&mut guest)[0], 32);
        let outside = receive(&mut guest);
        assert_eq!(&outside[..2], &[36, 0]);
        let mut reader = Reader {
            bytes: &outside[2..],
            at: 0,
        };
        assert_eq!(reader.string().unwrap(), anchor.key());
        for _ in 0..4 {
            reader.short().unwrap();
        }
        assert_eq!(reader.string().unwrap(), "user");
        assert_eq!(reader.at, outside.len() - 2);
        let edit = Mutation {
            location: anchor.clone(),
            action: Action::Whitelist {
                slot: 2,
                user: "USER2".into(),
                keys: std::array::from_fn(|i| format!("key{i}")),
            },
        };
        handle(&session, "user", 35, &request(&edit));
        assert_eq!(receive(&mut owner)[0], 35);
        assert_eq!(receive(&mut guest)[0], 35);
        let outside = receive(&mut guest);
        assert_eq!(&outside[..2], &[36, 1]);
        let mut reader = Reader {
            bytes: &outside[2..],
            at: 0,
        };
        assert_eq!(reader.string().unwrap(), anchor.key());
        assert_eq!(reader.byte(), Some(2));
        assert_eq!(reader.string().unwrap(), "user2");
        assert_eq!(reader.at, outside.len() - 2);
        let body = zone_body(&world, "shack1");
        let mut reader = Reader {
            bytes: &body,
            at: 0,
        };
        assert_eq!(reader.item().unwrap().id(), "Shack");
        reader.byte().unwrap();
        for _ in 0..4 {
            reader.short().unwrap();
        }
        assert_eq!(reader.string().unwrap(), "overworld");
        assert_eq!(reader.short(), Some(1));
        assert_eq!(reader.string().unwrap(), anchor.key());
        for _ in 0..6 {
            reader.short().unwrap();
        }
        assert_eq!(reader.string().unwrap(), "user");
        assert_eq!(reader.string().unwrap(), "");
        assert_eq!(reader.string().unwrap(), "user2");
        assert_eq!(reader.at, body.len());
        // Whitelisted interior building works, but cannot create a claim indoors.
        let mut interior = anchor.clone();
        interior.zone = "shack1".into();
        handle(
            &session,
            "user2",
            32,
            &request(&build(interior.clone(), "Stone")),
        );
        assert_eq!(receive(&mut owner)[0], 32);
        handle(
            &session,
            "user2",
            32,
            &request(&build(interior.clone(), "8-day Land Claim")),
        );
        assert_eq!(receive(&mut guest)[0], 33); // visible optimistic post is removed
        for _ in 0..9 {
            assert_eq!(receive(&mut guest)[0], 13);
        }
        assert_eq!(receive(&mut guest)[0], 37); // authoritative inherited table
        let expiry = world.land_claims.read().unwrap()[&anchor.key()].expires_at;
        expire(&session, expiry);
        assert_eq!(receive(&mut owner)[0], 34);
        assert_eq!(receive(&mut guest)[0], 34);
        assert_eq!(&receive(&mut guest)[..2], &[36, 2]);
        assert!(world.land_claims.read().unwrap().is_empty());
    }
}
