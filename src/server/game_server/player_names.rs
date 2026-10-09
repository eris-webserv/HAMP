/// Display spelling is independent of the lowercase identity used for routing
/// and land permissions. Legacy clients can send capitals in username_lower.
pub(super) struct DisplayName {
    received: String,
    display: String,
    resolved: bool,
}

impl DisplayName {
    pub(super) fn new(received: &str) -> Self {
        Self {
            received: received.into(),
            display: received.into(),
            resolved: false,
        }
    }

    pub(super) fn resolve(&mut self, lookup: impl FnOnce(&str) -> Option<String>) -> String {
        if !self.resolved {
            // Keep the received spelling in the request as well: friend servers
            // that fall back to the requested name must not lose its capitals.
            if let Some(display) = lookup(&self.received).filter(|name| !name.is_empty()) {
                self.display = display;
                self.resolved = true;
            }
        }
        self.display.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Session, SessionMode, registry_client::RegistryHandle};
    use super::*;
    use crate::defs::packet::{craft_batch, pack_string, unpack_string};
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::Arc;

    fn connect(
        session: &Arc<Session>,
        registry: Option<RegistryHandle>,
        name: &str,
    ) -> (TcpStream, std::thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let mut client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        client
            .set_read_timeout(Some(std::time::Duration::from_secs(4)))
            .unwrap();
        let (stream, address) = listener.accept().unwrap();
        let session = session.clone();
        let handler = std::thread::spawn(move || {
            super::super::handle_client(stream, address, session, registry)
        });
        let mut login = vec![38];
        login.extend(pack_string("test"));
        login.extend(pack_string(name));
        client.write_all(&craft_batch(2, &login)).unwrap();
        (client, handler)
    }

    fn packet(client: &mut TcpStream, opcode: u8) -> Vec<u8> {
        for _ in 0..30 {
            let mut header = [0u8; 9];
            client
                .read_exact(&mut header)
                .unwrap_or_else(|error| panic!("receiving opcode {opcode}: {error}"));
            assert_eq!(header[4], 3);
            let length = u32::from_le_bytes(header[5..].try_into().unwrap()) as usize;
            let mut payload = vec![0u8; length];
            client.read_exact(&mut payload).unwrap();
            if payload[0] == opcode {
                return payload;
            }
        }
        panic!("Expected opcode {opcode}");
    }

    fn assert_names(payload: &[u8], offset: usize, identity: &str, display: &str) -> usize {
        let (id, offset) = unpack_string(payload, offset);
        let (name, offset) = unpack_string(payload, offset);
        assert_eq!(id, identity);
        assert_eq!(name, display);
        offset
    }

    fn initial_data(client: &mut TcpStream) {
        let mut payload = vec![3];
        payload.extend([0u8; 8]);
        payload.extend(pack_string("overworld"));
        payload.push(0);
        client.write_all(&craft_batch(2, &payload)).unwrap();
    }

    fn wire_flow(registry: Option<RegistryHandle>, received: &str, expected: &str) {
        let world = Arc::new(super::super::land_claims::tests::world());
        world.zones.write().unwrap().insert(
            "shack1".into(),
            super::super::world_state::ZoneEntry::interior(
                super::super::world_state::InteriorData {
                    item_bytes: vec![0; 6],
                    rotation: 0,
                    cx: 0,
                    cz: 0,
                    tx: 0,
                    tz: 0,
                    outer_zone: "overworld".into(),
                    kind: super::super::special_generators::ZoneKind::House,
                },
            ),
        );
        let session = Session::new(
            "test",
            SessionMode::Managed(world.clone()),
            false,
            false,
            vec!["USER.NAME".into()],
            false,
        );
        let (mut observer, observer_handler) = connect(&session, registry.clone(), "user2");
        packet(&mut observer, 2);
        let (mut player, player_handler) = connect(&session, registry, received);
        packet(&mut player, 2);
        let join = packet(&mut observer, 7);
        let offset = assert_names(&join, 1, "user.name", expected);
        assert_eq!(join[offset], 1);
        assert!(session.players.lock().unwrap().contains_key("user.name"));
        assert!(world.players.read().unwrap().contains_key("user.name"));
        assert!(session.is_admin("User.Name"));

        initial_data(&mut observer);
        initial_data(&mut player);
        let nearby = packet(&mut observer, 19);
        assert_eq!(nearby[1], 1);
        assert_names(&nearby, 2, "user.name", expected);
        // Confirm the newcomer also receives the existing player's data.
        packet(&mut player, 19);
        let mut chat = vec![6];
        chat.extend(pack_string("hello"));
        player.write_all(&craft_batch(2, &chat)).unwrap();
        for client in [&mut observer, &mut player] {
            let chat = packet(client, 6);
            let offset = assert_names(&chat, 1, "user.name", expected);
            assert_eq!(unpack_string(&chat, offset).0, "hello");
        }

        // Leave and return to exercise the separate zone-change spawn path.
        let mut away = vec![20];
        away.extend(pack_string("shack1"));
        player.write_all(&craft_batch(2, &away)).unwrap();
        packet(&mut observer, 20);
        let mut zone = vec![20];
        zone.extend(pack_string("overworld"));
        player.write_all(&craft_batch(2, &zone)).unwrap();
        packet(&mut observer, 20);
        let nearby = packet(&mut observer, 19);
        assert_eq!(nearby[1], 1);
        assert_names(&nearby, 2, "user.name", expected);
        player.shutdown(std::net::Shutdown::Both).unwrap();
        player_handler.join().unwrap();
        let leave = packet(&mut observer, 7);
        let offset = assert_names(&leave, 1, "user.name", expected);
        assert_eq!(leave[offset], 0);
        observer.shutdown(std::net::Shutdown::Both).unwrap();
        observer_handler.join().unwrap();
    }

    #[test]
    fn standalone_game_keeps_legacy_capitalization_in_all_display_packets() {
        wire_flow(None, "UsEr.NaMe", "UsEr.NaMe");
    }

    #[test]
    fn unavailable_registry_keeps_legacy_capitalization_in_all_display_packets() {
        wire_flow(
            Some(RegistryHandle::display_fixture(&[])),
            "UsEr.NaMe",
            "UsEr.NaMe",
        );
    }

    #[test]
    fn friend_server_punctuated_name_is_used_with_lowercase_game_identity() {
        wire_flow(
            Some(RegistryHandle::display_fixture(&[
                ("user.name", "UsEr.NaMe"),
                ("user2", "User2"),
            ])),
            "user.name",
            "UsEr.NaMe",
        );
    }

    #[test]
    fn legacy_spelling_survives_missing_registry_and_failed_lookups() {
        for name in ["User", "ASDFASDF", "User.Name", "user2", "Ålice"] {
            let mut display = DisplayName::new(name);
            assert_eq!(display.resolve(|_| None), name);
            assert_eq!(
                display.resolve(|requested| {
                    assert_eq!(requested, name);
                    Some(requested.into())
                }),
                name
            );
        }
    }

    #[test]
    fn punctuated_name_is_cached_and_a_failed_lookup_can_be_retried() {
        let mut display = DisplayName::new("user.name");
        assert_eq!(display.resolve(|_| Some(String::new())), "user.name");
        assert_eq!(
            display.resolve(|requested| {
                assert_eq!(requested, "user.name");
                Some("User.Name".into())
            }),
            "User.Name"
        );
        assert_eq!(
            display.resolve(|_| panic!("cached display must not require another lookup")),
            "User.Name"
        );
    }
}
