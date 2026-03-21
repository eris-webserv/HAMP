# Game Server Login Sequence

Derived from Ghidra analysis of `GameServerReceiver$$OnReceive` (cases 0x02, 0x05, 0x0B)
and `GameServerSender$$SendLoginAttempt`, `GameServerSender$$SendInitialPlayerData`.

## Flow

```
Client                          Server
  |                               |
  |--- 0x66 (handshake probe) -->|
  |<-- batch(0, [0x09, 0x01]) ---|
  |                               |
  |--- C→S 0x26 LOGIN ---------->|
  |<-- S→C 0x02 LOGIN_SUCCESS ---|  (client auto-calls SendInitialPlayerData)
  |                               |
  |--- C→S 0x03 PLAYER_DATA ---->|
  |<-- S→C 0x13 NEARBY (others)--|  (existing players sent to newcomer)
  |<-- S→C 0x05 FULLY_IN_GAME --|  (last_byte=0 → client sends REQ_ZONE)
  |                               |
  |--- C→S 0x0A REQ_ZONE_DATA ->|
  |<-- S→C 0x0B ZONE_ASSIGNMENT-|  (status=0 → UnknownZoneGotoSpawn)
  |                               |
  |--- C→S 0x0C REQ_CHUNK ----->|  (repeated for each needed chunk)
  |<-- S→C 0x0D CHUNK_DATA -----|
  |                               |
  |--- C→S 0x11 POSITION ------>|  (continuous movement updates)
  |<-- S→C 0x11 POSITION (relay)|  (other players' positions)
```

## JumpToGame (S→C 0x25, friend server) — Connection Redirect

Sent by the friend server to redirect a client to a game server.
The client then disconnects from any existing game server and connects to the new one.

```
[str  host_display_name]        — compared with own name for UI text
[str  random_join_code]         — room token, sent back in C→S 0x26
[str  ip_address]               — game server IP
[str  ip_address_type]          — connection mode, e.g. "P2P"
[i16  port]                     — game server port
[u8   password_flag]            — read but unused (always 0)
```

### Client processing:
1. Strips a character from `host_display_name` (String.Replace with StringLiteral_736)
2. If cleaned name == own display name → shows "Connecting..."
3. Otherwise → shows "Joining [name]'s world..."
4. Calls `GameServerConnector.ConnectToGameServer(ip_address, ip_address_type, port, random_join_code)`
5. On connect success → calls `GameServerSender.SendLoginAttempt(random_join_code)`

## C→S 0x26 — Login Attempt

RE: `GameServerSender$$SendLoginAttempt`

```
[str  random_join_code]         — room token from JumpToGame
[str  username]                 — from PlayerData global (player's username)
```

## S→C 0x02 — Login Success

```
[str  server_name]              — server/world display name
[u8   is_host]                  — 0 = client, 1 = host
[u8   ignored]                  — always 0
[str  validator_code]           — "" for no validation
[i16  validator_variation]      — 0
[i16  n_others]                 — if is_host && n_others > 0: n × str usernames
```

## S→C 0x05 — Fully In Game

Sent after receiving C→S 0x03 (player data). Triggers zone request.

```
[i16  n_ids]                    — unique IDs to assign
  n × [i64  unique_id]
[i16  daynight]                 — time × 1000 as i16 (12000 = noon)
[i16  n_perks]                  — perk count
  n × [str  perk_name]
[u8   is_moderator]
[u8   max_companions]
[u8   last_byte]                — 0 → client sends C→S 0x0A (REQ_ZONE_DATA)
[u8   pvp]
[u8   ignored]
```

## S→C 0x0B — Zone Assignment

```
[u8   status]                   — 0 → UnknownZoneGotoSpawn(true, false)
[u8   is_host]                  — 0 = non-host
```

## C→S 0x0A — Request Zone Data

```
[str  zone_name]
[u8   type]                     — if type == 2 or 3: followed by packed_position
```

## S→C 0x13 — Player Update (nearby/gone)

```
[u8   type]                     — 1 = new player, 0 = player gone
```

### type = 1 (new player nearby)
```
[str  username]
[str  display_name]
[OnlinePlayerData]              — see online_player_data.md
```

### type = 0 (player gone)
```
[str  username]
[u8   mob_count]                — number of mobs to despawn
  mob_count × [str  mob_id]
```

## S→C 0x11 — Position Update (relay)

```
[str  username]                 — who moved
[PackedPosition  at]            — current position
[PackedPosition  to]            — target position
[PackedRotation  rot]           — rotation
```

## Heartbeat / Ping

- C→S 0x01 → S→C 0x01 (echo)
- C→S 0x0F → S→C 0x0F (echo)
