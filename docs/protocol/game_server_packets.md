# Game Server Packet Reference

Derived from Ghidra analysis of `GameServerSender` (C→S) and
`GameServerReceiver$$OnReceive` (S→C), cross-referenced with the HAMP
server implementation and Il2CppDumper output.

See also: [game_server_login.md](game_server_login.md) for the full login
handshake sequence, [chunk_data.md](chunk_data.md) for chunk wire format,
[online_player_data.md](online_player_data.md) for player data format,
[position_rotation.md](position_rotation.md) for PackedPosition/PackedRotation,
[inventory_item.md](inventory_item.md) for InventoryItem wire format.

## Wire primitives

| Notation   | Meaning                                          |
|------------|--------------------------------------------------|
| `u8`       | `Packet.PutByte` / `GetByte` — unsigned byte     |
| `i16`      | `Packet.PutShort` / `GetShort` — signed 16-bit   |
| `i32`      | `Packet.PutLong` / `GetLong` — signed 32-bit     |
| `str`      | `Packet.PutString` / `GetString` — UTF-16LE, 2-byte length prefix |
| `Pos`      | PackedPosition — 4 × i16 (chunk_x, chunk_z, local_x, local_z) |
| `Rot`      | PackedRotation — 4 × i16 (qx, qy, qz, qw × 100) |
| `Item`     | InventoryItem — key-value dict (see inventory_item.md) |

### Connection.Send priority

The third argument to `Connection.Send(conn, packet, priority)`:
- `0` — fire-and-forget (ping)
- `1` — normal (position, chat)
- `2` — reliable (most gameplay packets)
- `3` — low-priority/cosmetic (attack anims, cue updates, exp popups)

---

## C→S Packets (GameServerSender)

Packet ID is the first byte written after `Packet.ctor()`.

### 0x01 — Ping
```
[u8  0x01]
```
**Method:** `SendPing()`
**Priority:** 0
**Server action:** Echo back as S→C 0x01. Updates `last_server_ping` timestamp.

### 0x03 — Initial Player Data
```
[u8   0x03]
[Pos  position]              — player world position
[str  zone_name]             — current zone
[u8   body_slot]             — avatar body type (0=normal, 1=??, 2=dead?)
[i32  level]                 — player level
[Item × 3]                   — 3 equipment slots (PackForWeb)
[i32  hp_max]
[i32  hp]
[i32  hp_regen]
[i16  creature_count]
  creature_count × [str name] — companion creature names
[i16  time_scaled]           — client time × 1000 (host only)
[u8   has_zone_data]         — 1 if host, includes zone data below (host only)
[ZoneData]                   — zone data (PackForWeb) (host only)
[i16  trail_count]           — zone trail entries (host only)
  trail_count × {
    [str  trail_zone]
    [ZoneData]
  }
[u8   mob_count]             — mob IDs list
  mob_count × [str mob_id]
```
**Method:** `SendInitialPlayerData()`
**Priority:** 0
**Notes:** Sent automatically after receiving S→C 0x02 (LOGIN_SUCCESS).
The body is NOT directly compatible with OnlinePlayerData (see
online_player_data.md for details).

### 0x06 — Game Chat
```
[u8   0x06]
[str  message]
```
**Method:** `SendGameChat(message)`
**Priority:** 1

### 0x09 — Guard Die Notification
```
[u8   0x09]
[str  mob_name]
[str  owner_name]
```
**Method:** `SendGuardDieNotif(mob_name, owner_name)`
**Priority:** 2

### 0x0A — Request Zone Data
```
[u8   0x0A]
[str  zone_name]
[u8   type]                  — change_zone_type enum
[Pos  position]              — only if type == 2 or 3
```
**Method:** `RequestZoneData(zone_name, type, position)`
**Priority:** 2
**Notes:** Starts a `ZoneDataTimeout` coroutine. Type values come from
`ZoneDataControl.change_zone_type` enum.

### 0x0C — Request Chunk
```
[u8   0x0C]
[str  zone_name]
[i16  chunk_x]
[i16  chunk_z]
[u8   dimension]             — 0=overworld, 1=cave, 2=heaven, 3=hell, 4=pure, 5=other
[str  sub_zone]              — cached chunk sub-zone key or ""
```
**Method:** `SendRequestChunkAt(zone, chunkX, chunkZ)`
**Priority:** 2
**Notes:** Dimension byte is determined by checking the current zone's item
name against dimension-detection functions (IsCaveObject, IsHeavenDimension,
etc.). Requests are deduplicated via `chunks_mid_request` list.

### 0x11 — Player Position
```
[u8   0x11]
[Pos  current]               — current position
[Pos  target]                — movement target
[Rot  rotation]              — facing direction
[u8   nearby_check]          — 1 if moved >13 units since last check, else 0
```
**Method:** `SendPlayerPosition()` (called from `FixedUpdate`)
**Priority:** 1
**Notes:** If player has no SharedCreature (dead/spectating), sends
`prev_player_pos` as position and identity quaternion as rotation.
The `nearby_check` flag tells the server to recheck nearby players.

### 0x14 — Change Zone
```
[u8   0x14]
[str  zone_name]
[Pos  position]
[u8   on_map_change]         — 1 if triggered by map transition
```
**Method:** `SendChangeZone(zone, position, on_map_change)`
**Priority:** 2

### 0x15 — Start Teleport
```
[u8   0x15]
[str  tele_str]              — teleporter identifier string
```
**Method:** `SendStartTeleport(tele_str)`
**Priority:** 2

### 0x16 — End Teleport
```
[u8   0x16]
[Pos  new_position]
```
**Method:** `SendEndTeleport(new_position)`
**Priority:** 2

### 0x18 — Change Equipment
```
[u8   0x18]
[u8   slot_type]             — equipment slot
[Item new_item]              — InventoryItem (PackForWeb)
```
**Method:** `SendChangeEquipment(type, new_item)`
**Priority:** 2

### 0x19 — Update Parent Creatures
```
[u8   0x19]
[i16  creature_count]
  creature_count × [str creature_name]
```
**Method:** `SendUpdateParentCreatures()`
**Priority:** 2
**Notes:** Sends the player's current companion creature list from
`CreatureMorpher.GetCulledParentList`.

### 0x1A — Request Container
```
[u8   0x1A]
[u8   chest_request_type]    — enum: open, craft, etc.
[str  fn_validator]
```
**Method:** `SendRequestCurrContainer(type, fn_validator)`
**Priority:** 2

### 0x1E — Close Basket (Container)
```
[u8   0x1E]
[i32  basket_id]
[BasketContents]             — packed container contents
[str  fn_validator]
```
**Method:** `SendCloseBasket(basket_id, contents, fn_validator)`
**Priority:** 2

### 0x20 — Build Furniture
```
[u8   0x20]
[Item item]                  — the item being placed
[u8   rotation]
[str  zone]
[i16  chunk_x]
[i16  chunk_z]
[i16  inner_x]
[i16  inner_z]
[str  mp_cache_key]
[str  fn_validator]
```
**Method:** `SendBuildFurniture(...)`
**Priority:** 2

### 0x21 — Remove Object
```
[u8   0x21]
[str  zone]
[i16  chunk_x]
[i16  chunk_z]
[i16  inner_x]
[i16  inner_z]
[u8   rotation]              — element rotation
[Item element_item]
[str  mp_cache_key]
[str  fn_validator]
```
**Method:** `SendRemoveObject(...)`
**Priority:** 2

### 0x22 — Replace Buildable
```
[u8   0x22]
[Item new_item]
[Item old_element_item]
[u8   old_element_rot]
[str  zone]
[i16  chunk_x]
[i16  chunk_z]
[i16  inner_x]
[i16  inner_z]
[str  mp_cache_key]
[str  fn_validator]
```
**Method:** `SendReplaceBuildable(...)`
**Priority:** 2

### 0x23 — Change Land Claim User
```
[u8   0x23]
[str  zone]
[i16  chunk_x]
[i16  chunk_z]
[i16  inner_x]
[i16  inner_z]
[i16  user_index]
[str  new_username]
[i16  mp_cache_key_count]
  count × [str key]
```
**Method:** `SendChangeLandClaimUser(...)`
**Priority:** 2

### 0x26 — Login Attempt
```
[u8   0x26]
[str  random_join_code]      — room token from JumpToGame
[str  username]              — from PlayerData global
```
**Method:** `SendLoginAttempt(random_join_code)`
**Priority:** 2
**Notes:** See game_server_login.md for full login sequence.

### 0x27 — Claim Object
```
[u8   0x27]
[str  obj_str]               — object identifier to claim
```
**Method:** `SendClaimObject(obj_str)`
**Priority:** 2

### 0x28 — Release Interacting Object
```
[u8   0x28]
```
**Method:** `SendReleaseInteractingObject()`
**Priority:** 2
**Notes:** Empty payload — just the packet ID byte.

### 0x29 — Request More Unique IDs
```
[u8   0x29]
```
**Method:** `RequestMoreUniqueIds()`
**Priority:** 2

### 0x2B — Used Unique ID
```
[u8   0x2B]
[i32  unique_id]             — the ID that was consumed
```
**Method:** `SendUsedUniqueId(unique_id)`
**Priority:** 2

### 0x2D — Music Box Note Press
```
[u8   0x2D]
[u8   octave]
[u8   key]
[u8   instrument]
[u8   type]                  — press/release
```
**Method:** `SendMusicBoxRealtimeNotePress(octave, key, instrument, type)`
**Priority:** 3

### 0x2E — Request Teleporter Page
```
[u8   0x2E]
[u8   sub_type]              — 0 = by page number, 1 = by location
```
**Sub-type 0 (by page number):**
```
[i16  page]
[u8   in_search_page]
```
**Sub-type 1 (by location):**
```
[str  zone]
[i16  chunk_x]
[i16  chunk_z]
[i16  inner_x]
[i16  inner_z]
```
**Methods:** `RequestPageOfTeleportersByPageNumber(page, in_search_page)`,
`RequestPageOfTeleportersByTeleStr(zone, ...)`
**Priority:** 2

### 0x2F — Teleporter Page Response (host → server)
```
[u8   0x2F]
[str  user_requesting]
[i16  page]
[u8   has_more_pages]
[u8   teleporter_count]      — 0-3 per page
  count × {
    [str  display_name]
    [str  zone_name]
    [str  coords_concat]     — "zone|,|chunkX|,|localX|,|chunkZ|,|localZ"
    [str  description]
    [i16  chunk_x]
    [i16  chunk_z]
    [i16  inner_x]
    [i16  inner_z]
  }
```
**Method:** `PackPageOfTeleporters(user_requesting, page)`
**Priority:** 2
**Notes:** Host-side only. Reads teleporter data from PlayerData slots.

### 0x30 — Teleporter Screenshot Upload
```
[u8   0x30]
[str  zone]
[i16  chunk_x]
[i16  chunk_z]
[i16  inner_x]
[i16  inner_z]
[i32  byte_count]
[u8 × byte_count]            — JPEG screenshot data
```
**Method:** `SendTeleporterScreenshot(...)`
**Priority:** 2

### 0x31 — Request Teleporter Screenshot
```
[u8   0x31]
[str  zone]
[str  to_zone]
[i16  chunk_x]
[i16  chunk_z]
[i16  inner_x]
[i16  inner_z]
```
**Method:** `RequestTeleporterScreenshot(teleporter)`
**Priority:** 3

### 0x33 — Finished Editing Teleporter
```
[u8   0x33]
[str  title]
[str  description]
[str  zone]
[i16  chunk_x]
[i16  chunk_z]
[i16  inner_x]
[i16  inner_z]
```
**Method:** `SendFinishedEditingTeleporter(...)`
**Priority:** 2

### 0x34 — New Teleporter Search
```
[u8   0x34]
[str  search_term]
```
**Method:** `SendNewTeleSearch(search_term)`
**Priority:** 2

### 0x35 — Challenge Minigame Owner
```
[u8   0x35]
[str  username]
[u8   minigame_type]
```
**Method:** `TryChallengeMinigameOwner(username, minigame_type)`
**Priority:** 2

### 0x36 — Minigame Response
```
[u8   0x36]
[u8   response]              — accept/decline
[str  challenger]
[u8   minigame_type]
```
**Method:** `SendMinigameResponse(response, challenger, minigame_type)`
**Priority:** 2

### 0x37 — Begin Minigame
```
[u8   0x37]
[str  owner]
[u8   response]
[u8   minigame_type]
[u8 × 14]                    — ball_layout (14 ball positions, pool only)
```
**Method:** `SendBeginMinigame(owner, response, minigame_type, ball_layout)`
**Priority:** 2
**Notes:** `ball_layout` is only sent when non-null (pool game initial setup).

### 0x38 — Exit Minigame
```
[u8   0x38]
```
**Method:** `SendExitMinigame()`
**Priority:** 2

### 0x39 — Update Pool Cue Position
```
[u8   0x39]
[i32  cue_angle]             — degrees × 100
```
**Method:** `SendUpdatePoolCuePosition()`
**Priority:** 3

### 0x3A — Pool Shoot
```
[u8   0x3A]
[i32  degree]                — angle × 100
[i16  power]                 — power × 100
[i32  recording_size]
[u8 × recording_size]        — recording data
```
**Method:** `SendPoolShoot(deg, power, recording_data)`
**Priority:** 2

### 0x3B — Pool Sync Ready
```
[u8   0x3B]
```
**Method:** `SendPoolSyncReady()`
**Priority:** 2

### 0x3C — Pool Place White Ball
```
[u8   0x3C]
[i16  local_x]               — x × 100
[i16  local_y]               — y × 100
```
**Method:** `SendPoolPlaceWhiteBall(localPosition)`
**Priority:** 2

### 0x3D — Pool Play Again
```
[u8   0x3D]
```
**Method:** `SendPlayPoolAgain()`
**Priority:** 2

### 0x3E — Sit In Chair / Finished Sitting
```
[u8   0x3E]
[str  chair_id]              — interactable ID, or "" to finish sitting
```
**Methods:** `SendSitInChair(chair_id)`, `SendFinishedSittingInChair()` (sends "")
**Priority:** 2

### 0x3F — Try Claim Mobs
```
[u8   0x3F]
[i16  mob_count]
  count × {
    [str  combat_id]
    [str  display_name]
    [u8   creature_type]
    [i16  level]
    [i32  hp_max]
    [i32  hp]
  }
```
**Method:** `SendTryClaimMobs(attempt_request)`
**Priority:** 2

### 0x40 — Deload Mob
```
[u8   0x40]
[str  combat_id]
```
**Method:** `SendDeloadMob(combat_id)`
**Priority:** 2

### 0x41 — My Mob Positions
```
[u8   0x41]
[u8   mob_count]
  count × {
    [str  creature_id]
    [Pos  position]
    [Pos  target]
    [i16  rotation_y]        — y euler angle
  }
```
**Method:** `SendMyMobPositions()` (called from `FixedUpdate`)
**Priority:** 2

### 0x46 — Attack Animation
```
[u8   0x46]
[str  combat_id]
```
**Method:** `SendAttackAnimation(combat_id)`
**Priority:** 3

### 0x47 — Hit Mob
```
[u8   0x47]
[str  defender_combat_id]
[i32  real_dmg]
[i32  fake_dmg]
[u8   hit_col]               — Combatant.hit_col enum
[u8   missed]
[u8   dodged]
[str  attacker_combat_id]
[u8   mob_type]
[str  fn_validator]
```
**Method:** `SendHitMob(...)`
**Priority:** 2

### 0x48 — Mob Die
```
[u8   0x48]
[str  dead_mob_id]
[i16  delay]                 — delay × 100
[i16  splat_delay]           — splat_delay × 100
[str  origin_zone]
[i16  origin_chunk_x]
[i16  origin_chunk_z]
[i16  origin_inner_x]
[i16  origin_inner_z]
[i16  respawn_secs]
[str  killer_combat_id]
[u8   mob_type]
[u8   darksword_kill]
[u8   aether_banish]
[Item original_element_item]
[str  fn_validator]
```
**Method:** `SendMobDie(...)`
**Priority:** 2

### 0x4B — Increase HP
```
[u8   0x4B]
[str  combat_id]
[i32  amount_inc]
[str  fn_validator]
```
**Method:** `SendIncreaseHp(combat_id, amount_inc, fn_validator)`
**Priority:** 2

### 0x4C — Show Exp Receive
```
[u8   0x4C]
[str  text]
[Pos  position]
```
**Method:** `SendShowExpReceive(text, position)`
**Priority:** 3

### 0x4E — Companion Change Equip
```
[u8   0x4E]
[str  combat_name]
[Item hat]
[Item body]
[Item hand]
```
**Method:** `SendCompanionChangeEquip(combat_name, hat, body, hand)`
**Priority:** 2

### 0x4F — Rename Companion
```
[u8   0x4F]
[str  combat_name]
[str  new_companion_name]
```
**Method:** `SendRenameCompanion(combat_name, new_companion_name)`
**Priority:** 2

### 0x50 — Destroy Companion
```
[u8   0x50]
[str  combat_name]
```
**Method:** `SendDestroyCompanion(combat_name)`
**Priority:** 2

### 0x51 — Apply Perk
```
[u8   0x51]
[str  effect_name]
[str  target_id]
[str  caster_id]
[i32  caster_level]
[Item perk_data]             — PerkData (PackForWeb)
[i16  perk_level]
[u8   on_duration_reapply]
[str  fn_validator]
```
**Method:** `SendApplyPerk(...)`
**Priority:** 2

### 0x52 — Launch Projectile Perk
```
[u8   0x52]
[Item perk_data]
[i16  perk_level]
[str  target_id]
[str  caster_id]
[i32  caster_level]
[Pos  target_pos]
[Pos  caster_pos]
[str  fn_validator]
```
**Method:** `SendLaunchProjectilePerk(...)`
**Priority:** 2

### 0x53 — Quick Tag
```
[u8   0x53]
[u8   active]                — 1 = tag on, 0 = tag off
[str  fn_validator]
```
**Method:** `SendQuickTag(active, fn_validator)`
**Priority:** 2

### 0x54 — All Pre-Applied Perks
```
[u8   0x54]
[str  send_to_user]
[str  fn_validator]
```
**Method:** `SendAllPreAppliedPerks(send_to_user, fn_validator)`
**Priority:** 2

### 0x55 — Create Perk Drop
```
[u8   0x55]
[Pos  position]
[str  effect_name]
[Item perk_data]
[i16  perk_level]
[str  caster_id]
[i32  caster_level]
[str  fn_validator]
```
**Method:** `SendCreatePerkDrop(...)`
**Priority:** 2

### 0x56 — Respawn
```
[u8   0x56]
```
**Method:** `SendRespawn()`
**Priority:** 2

### 0x57 — Returning Back to Breeder
```
[u8   0x57]
```
**Method:** `SendReturningBackToBreeder()`
**Priority:** 2

### 0x58 — Update Synced Target IDs
```
[u8   0x58]
[u8   mob_count]
  count × {
    [str  creature_id]
    [str  target_combat_id]  — "" if no target
  }
```
**Method:** `UpdateSyncedTargetIds(caller)`
**Priority:** 2

### 0x59 — Created Local Mob
```
[u8   0x59]
[str  minion_combat_id]
```
**Method:** `SendCreatedLocalMob(minion_combat_id)`
**Priority:** 2
**Notes:** Checks connection status and `completely_logged_in` before sending.

### 0x5A — Bandit Flag Destroyed
```
[u8   0x5A]
[str  bandit_camp_instance]
```
**Method:** `SendBanditFlagDestroyed(bandit_camp_instance)`
**Priority:** 2

---

## S→C Packets (GameServerReceiver.OnReceive)

These are received by the client from the game server (or host relay).

### 0x01 — Ping Reply
```
[u8   0x01]
```
**Handler:** Updates `GameServerConnector.last_server_ping` to `UtcNow`.

### 0x02 — Login Success
```
[u8   0x02]
[str  server_name]
[u8   is_host]               — 0 = client, 1 = host
[u8   ignored]
[str  validator_code]
[i16  validator_variation]
[i16  n_others]              — if is_host && n_others > 0: n × str usernames
```
**Handler:** Stores server state, starts pinging, calls `SendInitialPlayerData()`.
See [game_server_login.md](game_server_login.md) for full flow.

### 0x04 — Unique ID Assignment
```
[u8   0x04]
[str  player_name]
[i32 × 25]                   — 25 unique IDs
```
**Handler:** Host generates and sends unique IDs to joining player.

### 0x05 — Fully In Game
```
[u8   0x05]
[i16  n_ids]
  n × [i32 unique_id]
[... daynight data]           — via ReceiveDaynight()
[i16  n_perks]
  n × [str perk_name]
[u8   is_moderator]
[u8   max_companions]
[u8   last_byte]              — 0 → client sends C→S 0x0A
[u8   pvp]
[u8   ignored]
```
**Handler:** Sets up game state, recreates companions, requests zone data.

### 0x06 — Chat Message
```
[u8   0x06]
[str  sender_name]
[str  display_name]
[str  message]
[u8   is_system]              — system message flag
```
**Handler:** Creates `chat_log`, adds to game chat, updates friend recently-seen.

### 0x07 — Player Login/Logout
```
[u8   0x07]
[str  player_id]
[str  display_name]
[u8   status]                 — 0 = logout, 1 = login
```
**Handler:** Shows login/logout message, updates `n_others_in_game`,
recycles unique IDs on logout.

### 0x08 — Companion Death Chat
```
[u8   0x08]
[str  player_name]
[str  companion_death_message]
```
**Handler:** Displays companion death in chat with icon.

### 0x09 — Guard Died
```
[u8   0x09]
[str  guard_name]
[str  additional_info]
```
**Handler:** Calls `CompanionController.OnGuardDie()`.

### 0x0B — Zone Assignment
```
[u8   0x0B]
[u8   status]                 — 0 → UnknownZoneGotoSpawn, 1 → process zone data
[u8   is_host]
```
**Handler:** Stops zone_data_timeout. Processes zone data or spawns at default.

### 0x0D — Chunk Data
See [chunk_data.md](chunk_data.md) for full format.

### 0x11 — Position Update (relay)
```
[u8   0x11]
[str  username]
[Pos  at]                     — current position
[Pos  to]                     — target position
[Rot  rot]                    — rotation
```
**Handler:** Updates `SharedCreature` movement target and rotation for the
named player. See [position_rotation.md](position_rotation.md).

### 0x12 — Batch Player Updates
```
[u8   0x12]
[i16  n_new]
  n × [OnlinePlayerData]     — see online_player_data.md
[i16  n_gone]
  n × [str username]
```
**Handler:** Calls `NewPlayerNearby` / `NearbyPlayerWentAway` for each.

### 0x13 — Single Player Update
```
[u8   0x13]
[u8   type]                   — 0 = gone, 1 = new nearby
```
- type 1: `[str username] [str display] [OnlinePlayerData]`
- type 0: `[str username] [u8 mob_count] mob_count × [str mob_id]`

### 0x15 — Teleport Start (relay)
```
[u8   0x15]
[str  destination]
```
**Handler:** `GameServerInterface.StartTeleportPlayer()`

### 0x16 — Teleport End (relay)
```
[u8   0x16]
[str  player_name]
[Pos  new_position]
```
**Handler:** `GameServerInterface.EndTeleportPlayer()`

### 0x17 — Daynight Update
```
[u8   0x17]
[... daynight data]
```
**Handler:** `ReceiveDaynight()`

### 0x18 — Equipment Change (relay)
```
[u8   0x18]
[str  player_name]
[u8   slot]
[Item new_item]
```
**Handler:** `GameServerInterface.PlayerChangeEquip()`

### 0x19 — Creature Change (relay)
```
[u8   0x19]
[str  player_name]
[i16  creature_count]
  count × [str creature_name]
```
**Handler:** `GameServerInterface.OtherPlayerChangeCreatures()`

### 0x1B — Open Container
```
[u8   0x1B]
[i32  container_id]
[BasketContents]
```
**Handler:** `inventory_ctr.SucceedOpenWorldContainer()`

### 0x1E — Save Container
```
[u8   0x1E]
[i32  container_id]
[BasketContents]
[str  unused]
```
**Handler:** `BasketContents.SaveToAllAsContainer()`

### 0x20 — Player Build (relay)
```
[u8   0x20]
[Item item]
[u8   rotation]
[str  zone]
[i16  chunk_x] [i16  chunk_z]
[i16  inner_x] [i16  inner_z]
[str  cache_key]
[str  item_data]
```
**Handler:** Adds to chunk or calls `ConstructionControl.PlayerBuildAt()`.

### 0x21 — Player Remove (relay)
```
[u8   0x21]
[str  zone]
[i16  chunk_x] [i16  chunk_z]
[i16  inner_x] [i16  inner_z]
[u8   rotation]
[Item element_item]
[str  removal_info]
```
**Handler:** Removes from chunk or calls `ConstructionControl.PlayerRemoveAt()`.

### 0x2A — Receive Unique IDs
```
[u8   0x2A]
[i16  id_count]
  count × [i32 unique_id]
```
**Handler:** Adds to `ConstructionControl.online_unique_ids_`, clears
`requesting_unique_ids` flag.

### 0x2F — Teleporter Page (from host)
```
[u8   0x2F]
[i16  page]
[u8   has_more]
[u8   count]                  — 0-3 teleporters
  count × {
    [str  display_name]
    [str  zone]
    [str  coords_concat]
    [str  description]
    [i16  chunkX] [i16  chunkZ]
    [i16  innerX] [i16  innerZ]
  }
```
**Handler:** Populates teleporter list UI with navigation buttons.

### 0x32 — Receive Teleporter Screenshot
```
[u8   0x32]
[str  teleporter_id]
[i32  byte_size]
[u8 × byte_size]             — JPEG data
```
**Handler:** Creates Texture2D/Sprite, caches in `cached_teleporter_textures`.

---

## Packet ID Summary Table

| ID | Direction | Name | Category |
|----|-----------|------|----------|
| 0x01 | both | Ping | Connection |
| 0x02 | S→C | Login Success | Auth |
| 0x03 | C→S | Initial Player Data | Auth |
| 0x04 | S→C | Unique ID Assignment | World |
| 0x05 | S→C | Fully In Game | Auth |
| 0x06 | both | Game Chat | Social |
| 0x07 | S→C | Player Login/Logout | Social |
| 0x08 | S→C | Companion Death Chat | Social |
| 0x09 | both | Guard Died | Combat |
| 0x0A | C→S | Request Zone Data | World |
| 0x0B | S→C | Zone Assignment | World |
| 0x0C | C→S | Request Chunk | World |
| 0x0D | S→C | Chunk Data | World |
| 0x11 | both | Position Update | Movement |
| 0x12 | S→C | Batch Player Updates | Players |
| 0x13 | S→C | Single Player Update | Players |
| 0x14 | C→S | Change Zone | World |
| 0x15 | both | Teleport Start | Movement |
| 0x16 | both | Teleport End | Movement |
| 0x17 | S→C | Daynight Update | World |
| 0x18 | both | Equipment Change | Inventory |
| 0x19 | both | Creature Change | Companions |
| 0x1A | C→S | Request Container | Inventory |
| 0x1B | S→C | Open Container | Inventory |
| 0x1E | both | Close/Save Container | Inventory |
| 0x20 | both | Build Furniture | Construction |
| 0x21 | both | Remove Object | Construction |
| 0x22 | C→S | Replace Buildable | Construction |
| 0x23 | C→S | Change Land Claim User | Construction |
| 0x26 | C→S | Login Attempt | Auth |
| 0x27 | C→S | Claim Object | World |
| 0x28 | C→S | Release Interacting | World |
| 0x29 | C→S | Request More Unique IDs | World |
| 0x2A | S→C | Receive Unique IDs | World |
| 0x2B | C→S | Used Unique ID | World |
| 0x2D | C→S | Music Box Note | Minigame |
| 0x2E | C→S | Request Teleporter Page | Teleporters |
| 0x2F | both | Teleporter Page Data | Teleporters |
| 0x30 | C→S | Teleporter Screenshot Upload | Teleporters |
| 0x31 | C→S | Request Teleporter Screenshot | Teleporters |
| 0x32 | S→C | Receive Teleporter Screenshot | Teleporters |
| 0x33 | C→S | Finished Editing Teleporter | Teleporters |
| 0x34 | C→S | New Tele Search | Teleporters |
| 0x35 | C→S | Challenge Minigame Owner | Minigame |
| 0x36 | C→S | Minigame Response | Minigame |
| 0x37 | C→S | Begin Minigame | Minigame |
| 0x38 | C→S | Exit Minigame | Minigame |
| 0x39 | C→S | Pool Cue Position | Minigame |
| 0x3A | C→S | Pool Shoot | Minigame |
| 0x3B | C→S | Pool Sync Ready | Minigame |
| 0x3C | C→S | Pool Place White Ball | Minigame |
| 0x3D | C→S | Pool Play Again | Minigame |
| 0x3E | C→S | Sit In Chair | Interaction |
| 0x3F | C→S | Try Claim Mobs | Combat |
| 0x40 | C→S | Deload Mob | Combat |
| 0x41 | C→S | My Mob Positions | Combat |
| 0x46 | C→S | Attack Animation | Combat |
| 0x47 | C→S | Hit Mob | Combat |
| 0x48 | C→S | Mob Die | Combat |
| 0x4B | C→S | Increase HP | Combat |
| 0x4C | C→S | Show Exp Receive | Combat |
| 0x4E | C→S | Companion Change Equip | Companions |
| 0x4F | C→S | Rename Companion | Companions |
| 0x50 | C→S | Destroy Companion | Companions |
| 0x51 | C→S | Apply Perk | Combat |
| 0x52 | C→S | Launch Projectile Perk | Combat |
| 0x53 | C→S | Quick Tag | Combat |
| 0x54 | C→S | All Pre-Applied Perks | Combat |
| 0x55 | C→S | Create Perk Drop | Combat |
| 0x56 | C→S | Respawn | Combat |
| 0x57 | C→S | Returning to Breeder | Companions |
| 0x58 | C→S | Update Synced Targets | Combat |
| 0x59 | C→S | Created Local Mob | Combat |
| 0x5A | C→S | Bandit Flag Destroyed | Combat |

---

## Server implementation status (HAMP)

The HAMP server currently implements handling for:
- **Auth:** 0x26 login, 0x02 login success, 0x05 fully-in-game
- **Movement:** 0x11 position relay
- **World:** 0x0A zone request, 0x0B zone assignment, 0x0C/0x0D chunk req/data
- **Players:** 0x13 nearby player updates
- **Connection:** 0x01/0x0F heartbeat

Packets not yet handled are relayed as-is in dummy world (P2P) mode or
silently dropped in managed server mode. See CLAUDE.md for session type
differences.
