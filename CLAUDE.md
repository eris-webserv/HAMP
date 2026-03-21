# HAMP — Project Notes

## Game session types

There are two distinct session types. Do not conflate them:

### Dummy worlds (relay sessions)
- Used exclusively by the **friend server** for player-to-player joining.
- When player A grants player B's join request, the friend server spawns a
  relay session (`spawn_relay_session`). The host client owns all world data;
  the server just forwards packets between players.
- No server-side world state — chunks, containers, etc. are served by the
  host client.

### Admin-spawned game servers (managed sessions)
- Standalone game servers with server-owned world state (`WorldState`).
- Spawned via admin terminal (`startworld`) on a spoofed user.
- The server serves chunks, tracks positions, and will eventually persist
  world state to disk (not yet implemented).
- Players connect the same way (JoinReq → auto-accept → JumpToGame) but the
  server handles everything — no host client involved.

### Future: world state persistence
Admin-spawned managed servers should eventually save/load world state (chunks,
containers, placed objects) to disk. This is not yet implemented. When adding
it, follow the Python reference implementation's approach of per-chunk JSON
files and a `_containers.json` for basket contents.
