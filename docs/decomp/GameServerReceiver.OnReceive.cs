// GameServerReceiver.OnReceive — Clean C# Reimplementation
//
// Source: Ghidra decompilation of GameServerReceiver.OnReceive (7,731 lines)
//         combined with IDA variant for cross-reference.
//
// IMPORTANT ARCHITECTURE NOTE:
//   This class handles packets in TWO distinct roles:
//
//   CLIENT ROLE — received by the local player from the server (S→C).
//     Most cases fall here. The client reads server-pushed data and updates
//     local game state (positions, chat, zone data, etc.).
//
//   HOST ROLE — received by the host client from players who relayed their
//     C→S packets through the server back to us (relay architecture).
//     The server does not interpret these; it forwards them verbatim.
//     Host-role cases are marked [HOST-RECEIVES] below.
//
// Wire primitives (all little-endian):
//   GetByte()   → u8
//   GetShort()  → i16
//   GetLong()   → i32
//   GetFloat()  → f32
//   GetString() → u16 length prefix + UTF-16LE chars
//   Pos         → 4 × i16 (x, y, z, w)
//   Rot         → 4 × i16
//   Item        → key-value dictionary (see ReadInventoryItem helper)
//
// Switch offset: the decompiler shows `switch(GetByte(p) - 1)` so
//   case 0 = packet 0x01, case 1 = packet 0x02, ... case N = packet 0xN+1.
//   This file uses the corrected packet IDs directly.

using System;
using System.Collections;
using System.Collections.Generic;
using UnityEngine;

public partial class GameServerReceiver : MonoBehaviour
{
    // -------------------------------------------------------------------------
    // State fields (non-exhaustive — only those referenced in OnReceive)
    // -------------------------------------------------------------------------

    private DateTime last_server_ping;
    private string server_name;
    private bool is_host;
    private bool pvp_enabled;
    private string validator;
    private int n_others;

    // Unique-ID tracking (host role)
    private Dictionary<string, List<int>> unique_ids_given_away = new();
    private List<int> online_unique_ids_ = new();
    private bool requesting_unique_ids;

    // Chunk cache (LRU, max 41 entries)
    private List<CachedChunk> cached_chunks = new();

    // Mob / combat tracking
    private Dictionary<int, SharedCreature> active_combatants = new();
    private List<int> awaiting_mob_claim = new();
    private List<int> inquired_mob_ids = new();
    private List<int> chunks_mid_request = new();

    // Land claims
    private List<LandClaimTimer> outdoor_land_claim_chunk_timers = new();

    // Pool / minigame
    private PoolGameControl pool_game_control;
    private bool is_in_mp_game;

    // -------------------------------------------------------------------------
    // OnReceive — main dispatch
    // -------------------------------------------------------------------------

    public void OnReceive(Packet incoming)
    {
        byte packetId = Packet.GetByte(incoming);

        switch (packetId)
        {
            // -----------------------------------------------------------------
            // 0x01 — PING (server heartbeat)
            // -----------------------------------------------------------------
            case 0x01:
                last_server_ping = DateTime.UtcNow;
                break;

            // -----------------------------------------------------------------
            // 0x02 — LOGIN_SUCCESS
            // Client learns it has successfully joined the game session.
            // -----------------------------------------------------------------
            case 0x02:
                server_name   = Packet.GetString(incoming);
                is_host        = Packet.GetByte(incoming) != 0;
                pvp_enabled    = Packet.GetByte(incoming) != 0;
                validator      = Packet.GetString(incoming);
                n_others       = Packet.GetShort(incoming);

                SendInitialPlayerData();
                break;

            // -----------------------------------------------------------------
            // 0x03 — INITIAL_PLAYER_DATA (C→S, forwarded to host)
            // [HOST-RECEIVES] A joining player sends their current state.
            // Wire: Pos(at) + Str(zone_name) + u8(body_slot) + i32(level)
            //       + Item×3 + i32(hp_max) + i32(hp) + i32(hp_regen)
            //       + i16(creature_count) + creature_count×Str(name)
            //       + [host-only trailing: time_scaled, has_zone_data,
            //          ZoneData, trail_count, mob_ids]
            // NOTE: The trailing host-only data is harmless when building OPD
            //       because the client stops reading after creature names.
            // -----------------------------------------------------------------
            case 0x03:
            {
                Pos   at        = Packet.GetPos(incoming);
                string zoneName = Packet.GetString(incoming);
                byte  bodySlot  = Packet.GetByte(incoming);

                // Everything remaining = level + items + hp + creatures + trailing.
                // Pass through directly when constructing OnlinePlayerData.
                byte[] rest = Packet.GetRemaining(incoming);

                var opd = new OnlinePlayerData
                {
                    at            = at,
                    to            = at,               // same position, not yet moving
                    rot           = Rot.Identity,
                    is_dead       = bodySlot,
                    currently_using    = "",
                    sitting_in_chair   = "",
                    // level, items, hp, creatures read from `rest`
                };

                // Register the player and broadcast their OPD to others.
                RegisterNewPlayer(opd, rest, zoneName);
                break;
            }

            // -----------------------------------------------------------------
            // 0x04 — UNIQUE_ID_ASSIGNMENT (C→S / S→C dual use)
            // [HOST-RECEIVES] A joining player requests unique object IDs.
            // Host generates 25 IDs and replies with 0x04.
            // Also received as S→C when this client is the requester (see 0x2A).
            // -----------------------------------------------------------------
            case 0x04:
            {
                string playerName = Packet.GetString(incoming);
                var newIds = GenerateUniqueIds(25);

                if (!unique_ids_given_away.ContainsKey(playerName))
                    unique_ids_given_away[playerName] = new List<int>();
                unique_ids_given_away[playerName].AddRange(newIds);

                SendUniqueIdAssignment(playerName, newIds);
                break;
            }

            // -----------------------------------------------------------------
            // 0x05 — FULLY_IN_GAME
            // Server confirms the client is fully loaded into the session.
            // -----------------------------------------------------------------
            case 0x05:
            {
                // Add any pending unique IDs.
                int idCount = Packet.GetShort(incoming);
                for (int i = 0; i < idCount; i++)
                    online_unique_ids_.Add(Packet.GetLong(incoming));

                ReceiveDaynight(incoming);

                bool perksDisabled = Packet.GetByte(incoming) != 0;
                bool isModerator   = Packet.GetByte(incoming) != 0;
                int  maxCompanions = Packet.GetShort(incoming);

                RecreateAllCompanions();

                byte lastByte = Packet.GetByte(incoming);
                if (lastByte == 0)
                    RequestZoneData();
                break;
            }

            // -----------------------------------------------------------------
            // 0x06 — GAME_CHAT
            // -----------------------------------------------------------------
            case 0x06:
            {
                string sender      = Packet.GetString(incoming);
                string displayName = Packet.GetString(incoming);
                string message     = Packet.GetString(incoming);
                bool   isSystem    = Packet.GetByte(incoming) != 0;

                ShowChatMessage(sender, displayName, message, isSystem);
                AddToRecentlySeenPlayers(sender, displayName);
                break;
            }

            // -----------------------------------------------------------------
            // 0x07 — PLAYER_LOGIN_LOGOUT
            // Another player has joined or left the session.
            // -----------------------------------------------------------------
            case 0x07:
            {
                int    playerId   = Packet.GetLong(incoming);
                string playerName = Packet.GetString(incoming);
                byte   status     = Packet.GetByte(incoming); // 0=logout, 1=login

                ShowPlayerLogInOrOut(playerName, status == 1);

                if (status == 1)
                    n_others++;
                else
                {
                    n_others--;
                    // Recycle any unique IDs we gave this player.
                    if (unique_ids_given_away.TryGetValue(playerName, out var ids))
                    {
                        RecycleUniqueIds(ids);
                        unique_ids_given_away.Remove(playerName);
                    }
                }
                break;
            }

            // -----------------------------------------------------------------
            // 0x08 — COMPANION_DEATH_CHAT
            // Notification that a companion died; displayed as chat.
            // -----------------------------------------------------------------
            case 0x08:
            {
                string playerName = Packet.GetString(incoming);
                string message    = Packet.GetString(incoming);
                ShowCompanionDeathMessage(playerName, message);
                break;
            }

            // -----------------------------------------------------------------
            // 0x09 — GUARD_DIED
            // A guard companion was killed.
            // -----------------------------------------------------------------
            case 0x09:
            {
                string guardName       = Packet.GetString(incoming);
                string additionalInfo  = Packet.GetString(incoming);
                CompanionController.OnGuardDie(guardName, additionalInfo);
                break;
            }

            // -----------------------------------------------------------------
            // 0x0A — REQUEST_ZONE_DATA (C→S, forwarded to host)
            // [HOST-RECEIVES] Another player needs zone data.
            // Wire: Str(zone_name) + Str(requester_name) + u8(type)
            // Host loads the zone from disk and replies with 0x0B.
            // -----------------------------------------------------------------
            case 0x0A:
            {
                string zoneName      = Packet.GetString(incoming);
                string requesterName = Packet.GetString(incoming);
                byte   requestType   = Packet.GetByte(incoming);

                ZoneData zone = LoadZoneFromDisk(zoneName);
                SendZoneData(requesterName, zone, requestType);
                break;
            }

            // -----------------------------------------------------------------
            // 0x0B — ZONE_DATA (S→C)
            // Server (or host) is delivering requested zone data.
            // -----------------------------------------------------------------
            case 0x0B:
            {
                StopZoneDataTimeoutCoroutine();

                bool hasData = Packet.GetByte(incoming) != 0;
                if (hasData)
                    ProcessIncomingZoneData(incoming);
                else
                    UnknownZoneGotoSpawn();
                break;
            }

            // -----------------------------------------------------------------
            // 0x0C — REQUEST_CHUNK (C→S, forwarded to host)
            // [HOST-RECEIVES] Player needs chunk data.
            // Wire: Str(requester) + Str(zone) + Str(sub_zone)
            //       + i16(chunk_x) + i16(chunk_z)
            // Host packs chunk into 0x0D response including bandit camps.
            // -----------------------------------------------------------------
            case 0x0C:
            {
                string requester = Packet.GetString(incoming);
                string zone      = Packet.GetString(incoming);
                string subZone   = Packet.GetString(incoming);
                short  chunkX    = Packet.GetShort(incoming);
                short  chunkZ    = Packet.GetShort(incoming);

                ChunkData chunk = GetChunk(zone, subZone, chunkX, chunkZ);
                SendChunkData(requester, chunk, chunkX, chunkZ);
                break;
            }

            // -----------------------------------------------------------------
            // 0x0D — CHUNK_DATA (S→C)
            // Received chunk data; cache it (LRU, max 41 entries).
            // -----------------------------------------------------------------
            case 0x0D:
            {
                ChunkData chunk = UnpackChunkData(incoming);

                if (cached_chunks.Count >= 41)
                    cached_chunks.RemoveAt(0); // evict oldest (LRU front)
                cached_chunks.Add(new CachedChunk(chunk));

                LoadBanditCamps(chunk);
                chunks_mid_request.Remove(chunk.key);
                break;
            }

            // -----------------------------------------------------------------
            // 0x11 — POSITION_UPDATE
            // Another player moved.
            // Wire: Str(player_name) + Pos(at) + Pos(to) + Rot
            // -----------------------------------------------------------------
            case 0x11:
            {
                string playerName = Packet.GetString(incoming);
                Pos    at         = Packet.GetPos(incoming);
                Pos    to         = Packet.GetPos(incoming);
                Rot    rot        = Packet.GetRot(incoming);

                SharedCreature creature = FindOnlinePlayer(playerName);
                creature?.UpdateMovement(at, to, rot);
                break;
            }

            // -----------------------------------------------------------------
            // 0x12 — BATCH_PLAYER_UPDATES
            // Server bulk-syncs which players are near/far.
            // Wire: i16(new_count) × NewPlayerNearby data
            //       + i16(gone_count) × NearbyPlayerWentAway data
            // -----------------------------------------------------------------
            case 0x12:
            {
                short newCount = Packet.GetShort(incoming);
                for (int i = 0; i < newCount; i++)
                    NewPlayerNearby(incoming);

                short goneCount = Packet.GetShort(incoming);
                for (int i = 0; i < goneCount; i++)
                    NearbyPlayerWentAway(Packet.GetString(incoming));
                break;
            }

            // -----------------------------------------------------------------
            // 0x13 — SINGLE_PLAYER_UPDATE
            // One player entered or left the local area.
            // Wire: u8(type) — 0=gone, 1=new; then player data or name
            // -----------------------------------------------------------------
            case 0x13:
            {
                byte type = Packet.GetByte(incoming);
                if (type == 0)
                    NearbyPlayerWentAway(Packet.GetString(incoming));
                else
                    NewPlayerNearby(incoming);
                break;
            }

            // -----------------------------------------------------------------
            // 0x15 — TELE_START
            // Server is teleporting this client to a destination.
            // Wire: Str(destination)
            // -----------------------------------------------------------------
            case 0x15:
            {
                string destination = Packet.GetString(incoming);
                GameServerInterface.StartTeleportPlayer(destination);
                break;
            }

            // -----------------------------------------------------------------
            // 0x16 — TELE_END
            // Teleport complete; server gives final position.
            // Wire: Str(player_name) + Pos
            // -----------------------------------------------------------------
            case 0x16:
            {
                string playerName = Packet.GetString(incoming);
                Pos    pos        = Packet.GetPos(incoming);
                GameServerInterface.EndTeleportPlayer(playerName, pos);
                break;
            }

            // -----------------------------------------------------------------
            // 0x17 — DAYNIGHT_UPDATE
            // Server pushed a day/night cycle update.
            // -----------------------------------------------------------------
            case 0x17:
                ReceiveDaynight(incoming);
                break;

            // -----------------------------------------------------------------
            // 0x18 — EQUIPMENT_CHANGE
            // Another player changed an equipment slot.
            // Wire: Str(player_name) + u8(slot) + Item
            // -----------------------------------------------------------------
            case 0x18:
            {
                string playerName = Packet.GetString(incoming);
                byte   slot       = Packet.GetByte(incoming);
                Item   item       = ReadInventoryItem(incoming);
                PlayerChangeEquip(playerName, slot, item);
                break;
            }

            // -----------------------------------------------------------------
            // 0x19 — CREATURE_CHANGE
            // Another player's companion list changed.
            // Wire: Str(player_name) + i16(count) + count×Str(name)
            // -----------------------------------------------------------------
            case 0x19:
            {
                string playerName = Packet.GetString(incoming);
                short  count      = Packet.GetShort(incoming);
                var    names      = new string[count];
                for (int i = 0; i < count; i++)
                    names[i] = Packet.GetString(incoming);
                OtherPlayerChangeCreatures(playerName, names);
                break;
            }

            // -----------------------------------------------------------------
            // 0x1B — OPEN_CONTAINER (S→C)
            // Server is delivering container contents to display.
            // Wire: Str(container_id) + BasketContents
            // -----------------------------------------------------------------
            case 0x1B:
            {
                HideAllPopups();
                string       containerId = Packet.GetString(incoming);
                BasketContents contents  = ReadBasketContents(incoming);
                SucceedOpenWorldContainer(containerId, contents);
                break;
            }

            // -----------------------------------------------------------------
            // 0x1C — REQUEST_CONTAINER (C→S, forwarded to host)
            // [HOST-RECEIVES] Player wants to open a container.
            // Wire: Str(requester) + Str(container_id)
            // Host loads from disk and sends 0x1B response.
            // -----------------------------------------------------------------
            case 0x1C:
            {
                string requester    = Packet.GetString(incoming);
                string containerId  = Packet.GetString(incoming);

                BasketContents contents = LoadContainerFromDisk(containerId);
                SendContainerContents(requester, containerId, contents);
                break;
            }

            // -----------------------------------------------------------------
            // 0x1D — LOOT_CHEST (S→C)
            // Server is delivering randomly generated loot chest contents.
            // Wire: Str(container_id)
            // -----------------------------------------------------------------
            case 0x1D:
            {
                HideAllPopups();
                string containerId = Packet.GetString(incoming);
                BasketContents loot = GenerateLootChest(containerId);
                SucceedOpenWorldContainer(containerId, loot);
                break;
            }

            // -----------------------------------------------------------------
            // 0x1E — SAVE_CONTAINER (S→C broadcast)
            // Another player saved a container; update local cache.
            // Wire: Str(container_id) + BasketContents
            // -----------------------------------------------------------------
            case 0x1E:
            {
                string         containerId = Packet.GetString(incoming);
                BasketContents contents    = ReadBasketContents(incoming);
                contents.SaveToAllAsContainer(containerId);
                break;
            }

            // -----------------------------------------------------------------
            // 0x20 — PLAYER_BUILD
            // A player placed a world object.
            // Wire: Item + Rot + Str(zone) + Pos(coords) + keys...
            // Special: if item type is land claim, call AddLandClaimsToNearbyChunks.
            // -----------------------------------------------------------------
            case 0x20:
            {
                Item   item   = ReadInventoryItem(incoming);
                Rot    rot    = Packet.GetRot(incoming);
                string zone   = Packet.GetString(incoming);
                Pos    coords = Packet.GetPos(incoming);
                var    keys   = ReadBuildKeys(incoming);

                if (IsLandClaim(item))
                    AddLandClaimsToNearbyChunks(zone, coords, item, keys);
                else
                    PlayerBuildAt(zone, coords, rot, item, keys);
                break;
            }

            // -----------------------------------------------------------------
            // 0x21 — PLAYER_REMOVE
            // A player removed a world object.
            // Wire: Str(zone) + Pos(coords) + Rot + Item + key
            // Special: if land claim, call RemoveLandClaimsFromNearbyChunks.
            // -----------------------------------------------------------------
            case 0x21:
            {
                string zone   = Packet.GetString(incoming);
                Pos    coords = Packet.GetPos(incoming);
                Rot    rot    = Packet.GetRot(incoming);
                Item   item   = ReadInventoryItem(incoming);
                string key    = Packet.GetString(incoming);

                if (IsLandClaim(item))
                    RemoveLandClaimsFromNearbyChunks(zone, coords);
                else
                    PlayerRemoveAt(zone, coords, rot, item, key);
                break;
            }

            // -----------------------------------------------------------------
            // 0x22 — PLAYER_REPLACE
            // A player replaced one world object with another (e.g. upgrade).
            // Wire: Item(new) + Item(old) + Rot + Str(zone) + Pos(coords) + key
            // Special: if music box, strip old song data before replacing.
            // -----------------------------------------------------------------
            case 0x22:
            {
                Item   newItem = ReadInventoryItem(incoming);
                Item   oldItem = ReadInventoryItem(incoming);
                Rot    rot     = Packet.GetRot(incoming);
                string zone    = Packet.GetString(incoming);
                Pos    coords  = Packet.GetPos(incoming);
                string key     = Packet.GetString(incoming);

                if (IsMusicBox(oldItem))
                    RemoveSongFromMusicBox(zone, coords);

                PlayerReplaceAt(zone, coords, rot, newItem, oldItem, key);
                break;
            }

            // -----------------------------------------------------------------
            // 0x23 — CHANGE_LAND_CLAIM_USER
            // Ownership of a land claim changed.
            // Wire: Str(zone) + Pos(coords) + u8(status) + Str(username)
            //       + 9×Str(extra_strings)
            // -----------------------------------------------------------------
            case 0x23:
            {
                string zone    = Packet.GetString(incoming);
                Pos    coords  = Packet.GetPos(incoming);
                byte   status  = Packet.GetByte(incoming);
                string username = Packet.GetString(incoming);
                var    extras  = new string[9];
                for (int i = 0; i < 9; i++)
                    extras[i] = Packet.GetString(incoming);

                ModifyLandClaimTimer(zone, coords, status, username, extras);
                break;
            }

            // -----------------------------------------------------------------
            // 0x24 — LAND_CLAIM_TIMER
            // Update, add, or remove a land claim chunk timer.
            // Wire: u8(type) + Str(chunk_key)
            //   type 0 = update existing, 1 = add new, 2 = remove
            // -----------------------------------------------------------------
            case 0x24:
            {
                byte   type     = Packet.GetByte(incoming);
                string chunkKey = Packet.GetString(incoming);

                switch (type)
                {
                    case 0: UpdateLandClaimTimer(chunkKey);                      break;
                    case 1: outdoor_land_claim_chunk_timers.Add(
                                new LandClaimTimer(chunkKey));                   break;
                    case 2: RemoveLandClaimTimer(chunkKey);                      break;
                }
                break;
            }

            // -----------------------------------------------------------------
            // 0x25 — ZONE_DATA_REFRESH
            // Full zone refresh: clear old land claims, apply new zone data.
            // -----------------------------------------------------------------
            case 0x25:
            {
                outdoor_land_claim_chunk_timers.Clear();
                ZoneData newZone = UnpackZoneData(incoming);
                UpdateZoneItemOnChangedOutside(newZone);
                break;
            }

            // -----------------------------------------------------------------
            // 0x26 — REQUEST_ZONE_TRAIL / ZONE_TRAIL_DATA (dual-use packet)
            //
            // [HOST-RECEIVES] Player requests trail data for multiple zones.
            //   Wire: i16(zone_count) × Str(zone_name)
            //   Host loads ZoneData for each and sends 0x26 response.
            //
            // [CLIENT-RECEIVES] Server delivers packed zone trail data.
            //   Wire: i16(zone_count) × packed ZoneData
            //
            // Disambiguation: check whether is_host; or read the first field
            // and decide based on whether it looks like a count vs. zone data.
            // -----------------------------------------------------------------
            case 0x26:
            {
                if (is_host)
                {
                    // Host receives the request.
                    short zoneCount = Packet.GetShort(incoming);
                    var   zoneNames = new string[zoneCount];
                    for (int i = 0; i < zoneCount; i++)
                        zoneNames[i] = Packet.GetString(incoming);

                    var zoneDataList = new List<ZoneData>();
                    foreach (var name in zoneNames)
                        zoneDataList.Add(LoadZoneFromDisk(name));

                    SendZoneTrailData(zoneDataList);
                }
                else
                {
                    // Client receives the response.
                    short zoneCount = Packet.GetShort(incoming);
                    for (int i = 0; i < zoneCount; i++)
                        ProcessIncomingZoneTrail(incoming);
                }
                break;
            }

            // -----------------------------------------------------------------
            // 0x27 — CLAIM_OBJECT
            // A player has started interacting with (claiming) a world object.
            // Wire: i32(player_id) + Str(player_name2) [equipment slot values]
            // -----------------------------------------------------------------
            case 0x27:
            {
                int    playerId    = Packet.GetLong(incoming);
                string playerName2 = Packet.GetString(incoming);

                OnlinePlayer player = FindOnlinePlayerById(playerId);
                if (player != null)
                    player.UpdateEquipmentSlotValues(playerName2);
                break;
            }

            // -----------------------------------------------------------------
            // 0x28 — RELEASE_INTERACTING
            // A player stopped interacting with a world object.
            // Wire: Str(player_name)
            // -----------------------------------------------------------------
            case 0x28:
            {
                string playerName = Packet.GetString(incoming);
                OnlinePlayer player = FindOnlinePlayer(playerName);
                if (player != null)
                    player.currently_using = "";
                break;
            }

            // -----------------------------------------------------------------
            // 0x29 — REQUEST_UNIQUE_IDS (C→S, forwarded to host)
            // [HOST-RECEIVES] Player needs a batch of unique object IDs.
            // Wire: Str(player_name)
            // Host generates 10 IDs and sends 0x2A response.
            // -----------------------------------------------------------------
            case 0x29:
            {
                string playerName = Packet.GetString(incoming);
                var    newIds     = GenerateUniqueIds(10);

                if (!unique_ids_given_away.ContainsKey(playerName))
                    unique_ids_given_away[playerName] = new List<int>();
                unique_ids_given_away[playerName].AddRange(newIds);

                SendUniqueIdBatch(playerName, newIds);
                break;
            }

            // -----------------------------------------------------------------
            // 0x2A — RECEIVE_UNIQUE_IDS (S→C)
            // Server is delivering a batch of unique IDs to this client.
            // Wire: i16(count) + count×i32(id)
            // -----------------------------------------------------------------
            case 0x2A:
            {
                short count = Packet.GetShort(incoming);
                for (int i = 0; i < count; i++)
                    online_unique_ids_.Add(Packet.GetLong(incoming));
                requesting_unique_ids = false;
                break;
            }

            // -----------------------------------------------------------------
            // 0x2B — USED_UNIQUE_ID
            // A player consumed one of the IDs we gave them.
            // Wire: Str(player_name) + i32(unique_id)
            // -----------------------------------------------------------------
            case 0x2B:
            {
                string playerName = Packet.GetString(incoming);
                int    uniqueId   = Packet.GetLong(incoming);

                if (unique_ids_given_away.TryGetValue(playerName, out var ids))
                    ids.Remove(uniqueId);
                break;
            }

            // -----------------------------------------------------------------
            // 0x2D — MUSIC_BOX_NOTE
            // A player pressed or released a music box key.
            // Wire: Str(player) + u8(type: 0=release,1=press)
            //       + u8(octave) + u8(key) + u8(instrument)
            // -----------------------------------------------------------------
            case 0x2D:
            {
                string player     = Packet.GetString(incoming);
                byte   noteType   = Packet.GetByte(incoming); // 0=release, 1=press
                byte   octave     = Packet.GetByte(incoming);
                byte   key        = Packet.GetByte(incoming);
                byte   instrument = Packet.GetByte(incoming);

                if (noteType == 1)
                    online_finger_pressed(player, octave, key, instrument);
                else
                    remove_online_finger_note(player, octave, key);
                break;
            }

            // -----------------------------------------------------------------
            // 0x2E — REQUEST_TELE_PAGE (C→S, forwarded to host)
            // [HOST-RECEIVES] Player requests a page of teleporter data.
            // Wire: Str(requester) + u8(sub_type)
            //   sub_type 0 = by page, else = by location
            // -----------------------------------------------------------------
            case 0x2E:
            {
                string requester = Packet.GetString(incoming);
                byte   subType   = Packet.GetByte(incoming);

                if (subType == 0)
                    PackPageOfTeleportersByPage(requester);
                else
                    PackPageOfTeleportersByLocation(requester);
                break;
            }

            // -----------------------------------------------------------------
            // 0x2F — TELE_PAGE_RESPONSE (S→C)
            // Received a page of teleporter listing data.
            // Wire: u8(page) + u8(has_more)
            //       + i16(count) × teleporter_data (L/mid/R layout)
            // -----------------------------------------------------------------
            case 0x2F:
            {
                HideAllPopups();
                LayoutCraftingTab();

                byte  page    = Packet.GetByte(incoming);
                byte  hasMore = Packet.GetByte(incoming);
                short count   = Packet.GetShort(incoming);

                for (int i = 0; i < count; i++)
                {
                    TeleporterData left  = ReadTeleporterData(incoming);
                    TeleporterData mid   = ReadTeleporterData(incoming);
                    TeleporterData right = ReadTeleporterData(incoming);
                    DrawOnlineTeleporterSlot(left, mid, right);
                }
                break;
            }

            // -----------------------------------------------------------------
            // 0x30 — TELE_SCREENSHOT_RECEIVE (S→C broadcast)
            // A player uploaded a screenshot for a teleporter.
            // Wire: Str(zone) + Pos(coords) + i32(byte_count) + byte_count×u8
            // -----------------------------------------------------------------
            case 0x30:
            {
                string zone      = Packet.GetString(incoming);
                Pos    coords    = Packet.GetPos(incoming);
                int    byteCount = Packet.GetLong(incoming);
                byte[] jpegData  = Packet.GetBytes(incoming, byteCount);

                string teleId = GetCustomTeleId(zone, coords);
                SaveTeleporterJpegToDisk(teleId, jpegData);
                break;
            }

            // -----------------------------------------------------------------
            // 0x31 — REQUEST_TELE_SCREENSHOT (C→S, forwarded to host)
            // [HOST-RECEIVES] Player wants a teleporter screenshot.
            // Wire: Str(requester) + Str(zone) + Pos(coords)
            // Host loads JPEG from disk and sends 0x32 response.
            // -----------------------------------------------------------------
            case 0x31:
            {
                string requester = Packet.GetString(incoming);
                string zone      = Packet.GetString(incoming);
                Pos    coords    = Packet.GetPos(incoming);

                byte[] jpegData = LoadTeleporterJpegFromDisk(zone, coords);
                SendTeleporterScreenshot(requester, zone, coords, jpegData);
                break;
            }

            // -----------------------------------------------------------------
            // 0x32 — TELE_SCREENSHOT_DATA (S→C)
            // Received screenshot data for a specific teleporter.
            // Wire: Str(tele_id) + i32(byte_count) + byte_count×u8
            // -----------------------------------------------------------------
            case 0x32:
            {
                string teleId    = Packet.GetString(incoming);
                int    byteCount = Packet.GetLong(incoming);
                byte[] jpegData  = Packet.GetBytes(incoming, byteCount);

                Texture2D tex = CreateTexture(jpegData);
                CacheTeleporterSprite(teleId, Sprite.Create(tex, ...));
                UpdateTeleporterUiIfOpen(teleId);
                break;
            }

            // -----------------------------------------------------------------
            // 0x33 — FINISHED_EDITING_TELE
            // A player saved metadata for a custom teleporter.
            // Wire: Str(title) + Str(description) + Str(zone) + Pos(coords)
            // -----------------------------------------------------------------
            case 0x33:
            {
                string title   = Packet.GetString(incoming);
                string desc    = Packet.GetString(incoming);
                string zone    = Packet.GetString(incoming);
                Pos    coords  = Packet.GetPos(incoming);

                string teleId = GetCustomTeleId(zone, coords);
                StoreTeleporterMetadata(teleId, title, desc);
                break;
            }

            // -----------------------------------------------------------------
            // 0x35 — CHALLENGE_MINIGAME
            // Another player is challenging us to a minigame.
            // Wire: Str(challenger) + u8(response_type)
            // -----------------------------------------------------------------
            case 0x35:
            {
                string challenger    = Packet.GetString(incoming);
                byte   responseType  = Packet.GetByte(incoming);
                SendMinigameResponse(challenger, responseType);
                break;
            }

            // -----------------------------------------------------------------
            // 0x36 — MINIGAME_RESPONSE
            // Response to our minigame challenge.
            // Wire: Str(player) + u8(response) + u8(status)
            // -----------------------------------------------------------------
            case 0x36:
            {
                string player   = Packet.GetString(incoming);
                byte   response = Packet.GetByte(incoming);
                byte   status   = Packet.GetByte(incoming);
                ShowMinigameResponsePopup(player, response, status);
                break;
            }

            // -----------------------------------------------------------------
            // 0x37 — BEGIN_MINIGAME
            // Minigame session is starting.
            // Wire: Str(owner) + u8(status) + u8(has_layout)
            //   status 0 = declined → show decline popup
            //   status 1 = leaving  → show leave popup
            //   status 2 = accept   → OpenPoolTable; if has_layout: ArrangeBalls + StartMpGame
            // -----------------------------------------------------------------
            case 0x37:
            {
                string owner     = Packet.GetString(incoming);
                byte   status    = Packet.GetByte(incoming);
                bool   hasLayout = Packet.GetByte(incoming) != 0;

                switch (status)
                {
                    case 0: ShowDeclinePopup(owner);  break;
                    case 1: ShowLeavePopup(owner);    break;
                    case 2:
                        OpenPoolTable(owner);
                        if (hasLayout)
                        {
                            ArrangeBalls(incoming);
                            StartMpGame();
                        }
                        break;
                }
                break;
            }

            // -----------------------------------------------------------------
            // 0x38 — EXIT_MINIGAME
            // The other player exited the minigame.
            // -----------------------------------------------------------------
            case 0x38:
                is_in_mp_game = false;
                PressClose();
                ShowMinigameExitPopup();
                break;

            // -----------------------------------------------------------------
            // 0x39 — POOL_CUE_POSITION
            // Live update of opponent's cue angle.
            // Wire: i16(angle×100) → divide by 100 for float angle
            // -----------------------------------------------------------------
            case 0x39:
            {
                short angleScaled = Packet.GetShort(incoming);
                pool_game_control?.TryUpdateCuePosition(angleScaled / 100f);
                break;
            }

            // -----------------------------------------------------------------
            // 0x3A — POOL_SHOOT
            // Opponent took a shot.
            // Wire: i16(degree×100) + i16(power×100) + PoolGameRecording
            // -----------------------------------------------------------------
            case 0x3A:
            {
                float degree       = Packet.GetShort(incoming) / 100f;
                float power        = Packet.GetShort(incoming) / 100f;
                PoolGameRecording recording = UnpackPoolGameRecording(incoming);

                pool_game_control.SetRecording(recording);
                if (pool_game_control.state == 18)
                    pool_game_control.ShowMpRecording(degree, power);
                break;
            }

            // -----------------------------------------------------------------
            // 0x3B — POOL_SYNC_READY
            // Opponent confirmed ready to sync pool state.
            // -----------------------------------------------------------------
            case 0x3B:
                pool_game_control?.OnOtherPlayerReady();
                break;

            // -----------------------------------------------------------------
            // 0x3C — POOL_PLACE_WHITE_BALL
            // Opponent placed the cue ball.
            // Wire: i16(x×100) + i16(y×100)
            // -----------------------------------------------------------------
            case 0x3C:
            {
                float x = Packet.GetShort(incoming) / 100f;
                float y = Packet.GetShort(incoming) / 100f;
                pool_game_control?.PlaceWhiteBallAt(x, y);
                break;
            }

            // -----------------------------------------------------------------
            // 0x3D — POOL_PLAY_AGAIN
            // Opponent requested a rematch; re-rack the balls.
            // Wire: 14 × BallPosition (each with x/y)
            // -----------------------------------------------------------------
            case 0x3D:
            {
                var positions = new BallPosition[14];
                for (int i = 0; i < 14; i++)
                    positions[i] = ReadBallPosition(incoming);
                pool_game_control?.ArrangeBalls(positions);
                pool_game_control?.RestartMpGame();
                break;
            }

            // -----------------------------------------------------------------
            // 0x3E — SIT_IN_CHAIR
            // A player sat in or stood up from a chair.
            // Wire: Str(player_name) + Str(chair_id)
            //   chair_id "" = stood up → EndSittingInChair
            // -----------------------------------------------------------------
            case 0x3E:
            {
                string playerName = Packet.GetString(incoming);
                string chairId    = Packet.GetString(incoming);

                if (string.IsNullOrEmpty(chairId))
                    EndSittingInChair(playerName);
                else
                    TrySitInChairObj(playerName, chairId);
                break;
            }

            // -----------------------------------------------------------------
            // 0x3F — CLAIM_MOBS_RESPONSE (S→C)
            // Server responds to our mob claim requests.
            // Wire: i16(count) × (i32(combat_id) + u8(response))
            //   response ok + mob not yet active → SpawnLocalMob
            // -----------------------------------------------------------------
            case 0x3F:
            {
                short count = Packet.GetShort(incoming);
                for (int i = 0; i < count; i++)
                {
                    int  combatId = Packet.GetLong(incoming);
                    byte response = Packet.GetByte(incoming);

                    awaiting_mob_claim.Remove(combatId);

                    if (response == 0 && !active_combatants.ContainsKey(combatId))
                        SpawnLocalMob(combatId);
                }
                break;
            }

            // -----------------------------------------------------------------
            // 0x40 — DELOAD_MOB
            // A mob should be removed from our simulation.
            // Wire: i32(combat_id)
            //   - Guard/compound type → destroy immediately
            //   - Companion + chunk loaded → send 0x44 (companion leaving chunk)
            //   - Otherwise → destroy
            // -----------------------------------------------------------------
            case 0x40:
            {
                int combatId = Packet.GetLong(incoming);
                SharedCreature mob = FindMob(combatId);
                if (mob == null) break;

                if (mob.IsGuardOrCompound())
                    DestroyMob(combatId);
                else if (mob.IsCompanion() && IsChunkLoaded(mob.chunk))
                    SendCompanionLeavingChunk(combatId); // sends 0x44
                else
                    DestroyMob(combatId);
                break;
            }

            // -----------------------------------------------------------------
            // 0x41 — MOB_POSITIONS_UPDATE
            // Batch position update for mobs under another player's authority.
            // Wire: Str(player_name) + i16(mob_count)
            //       × (i32(id) + Pos + Rot)
            // If a mob_id is unknown, add to inquired list and send 0x42.
            // -----------------------------------------------------------------
            case 0x41:
            {
                string playerName = Packet.GetString(incoming);
                short  mobCount   = Packet.GetShort(incoming);

                for (int i = 0; i < mobCount; i++)
                {
                    int combatId = Packet.GetLong(incoming);
                    Pos pos      = Packet.GetPos(incoming);
                    Rot rot      = Packet.GetRot(incoming);

                    if (active_combatants.TryGetValue(combatId, out var creature))
                        creature.UpdateMovement(pos, pos, rot);
                    else if (!inquired_mob_ids.Contains(combatId))
                    {
                        inquired_mob_ids.Add(combatId);
                        RequestMobData(playerName, combatId);
                    }
                }
                break;
            }

            // -----------------------------------------------------------------
            // 0x42 — REQUEST_MOB_DATA (C→S, forwarded to host)
            // [HOST-RECEIVES] Player needs full data for specific mobs.
            // Wire: Str(requester) + i16(count) × i32(combat_id)
            // Host replies with 0x43 containing CreatureStruct + position.
            // -----------------------------------------------------------------
            case 0x42:
            {
                string requester = Packet.GetString(incoming);
                short  count     = Packet.GetShort(incoming);
                var    ids       = new int[count];
                for (int i = 0; i < count; i++)
                    ids[i] = Packet.GetLong(incoming);

                SendMobDataResponse(requester, ids);
                break;
            }

            // -----------------------------------------------------------------
            // 0x43 — MOB_DATA_RESPONSE (S→C)
            // Full mob data for requested mobs.
            // Wire: Str(player) + i16(count)
            //       × (i32(id) + u8(has_data) + [CreatureStruct + Pos])
            // -----------------------------------------------------------------
            case 0x43:
            {
                string player = Packet.GetString(incoming);
                short  count  = Packet.GetShort(incoming);

                for (int i = 0; i < count; i++)
                {
                    int  combatId = Packet.GetLong(incoming);
                    bool hasData  = Packet.GetByte(incoming) != 0;

                    inquired_mob_ids.Remove(combatId);

                    if (hasData)
                    {
                        CreatureStruct cs  = ReadCreatureStruct(incoming);
                        Pos            pos = Packet.GetPos(incoming);
                        SpawnNetMob(combatId, cs, pos);
                    }
                }
                break;
            }

            // -----------------------------------------------------------------
            // 0x45 — MOB_RESPAWN
            // A mob has respawned (after death timer expired).
            // Wire: i32(combat_id)
            // Host: save state, delete old, SpawnLocalMob, restore rotation.
            // -----------------------------------------------------------------
            case 0x45:
            {
                int combatId = Packet.GetLong(incoming);
                SharedCreature old = FindMob(combatId);
                if (old == null) break;

                Pos           savedPos = old.pos;
                Rot           savedRot = old.rot;
                CreatureStruct cs      = old.creatureStruct;

                DestroyMob(combatId);
                int newId = SpawnLocalMob(combatId, cs, savedPos);
                SetMobRotation(newId, savedRot);
                break;
            }

            // -----------------------------------------------------------------
            // 0x46 — ATTACK_ANIM
            // A mob played its attack animation.
            // Wire: i32(combat_id)
            // -----------------------------------------------------------------
            case 0x46:
            {
                int combatId = Packet.GetLong(incoming);
                FindMob(combatId)?.VisuallyAttack();
                break;
            }

            // -----------------------------------------------------------------
            // 0x47 — HIT_MOB
            // A mob was hit.
            // Wire: i32(defender_id) + i32(dmg) + i32(dmg2) + u8(flags)
            //       + i32(attacker_id)  [attacker remapped: self → local player id]
            // -----------------------------------------------------------------
            case 0x47:
            {
                int  defenderId = Packet.GetLong(incoming);
                int  dmg        = Packet.GetLong(incoming);
                int  dmg2       = Packet.GetLong(incoming);
                byte flags      = Packet.GetByte(incoming);
                int  attackerId = Packet.GetLong(incoming);

                // Attacker id may be a "self" sentinel; remap to local player id.
                attackerId = RemapSelfId(attackerId);

                if (active_combatants.TryGetValue(defenderId, out var defender))
                    ApplyHitToMob(defender, dmg, dmg2, flags, attackerId);
                break;
            }

            // -----------------------------------------------------------------
            // 0x48 — MOB_DIE
            // A mob died.
            // Wire: i32(dead_mob_id) + Pos(coords)
            //       + i32(death_timing) + i32(respawn_timing)
            //       + Str(killer_name) + u8(flags) + Item(element_item)
            // Complex: handles drops, respawn scheduling, host-side cleanup.
            // -----------------------------------------------------------------
            case 0x48:
            {
                int    deadMobId     = Packet.GetLong(incoming);
                Pos    coords        = Packet.GetPos(incoming);
                int    deathTiming   = Packet.GetLong(incoming);
                int    respawnTiming = Packet.GetLong(incoming);
                string killerName    = Packet.GetString(incoming);
                byte   flags         = Packet.GetByte(incoming);
                Item   elementItem   = ReadInventoryItem(incoming);

                OnMobDie(deadMobId, coords, deathTiming, respawnTiming,
                         killerName, flags, elementItem);
                break;
            }

            // -----------------------------------------------------------------
            // 0x4A — MOB_STAT_UPDATE
            // A mob's stats changed (level up, hp change, etc.).
            // Wire: i32(mob_id) + i32(hp_max) + i32(hp)
            //       + i32(level_prev) + i32(hp_regen)
            // -----------------------------------------------------------------
            case 0x4A:
            {
                int mobId    = Packet.GetLong(incoming);
                int hpMax    = Packet.GetLong(incoming);
                int hp       = Packet.GetLong(incoming);
                int levelPrev = Packet.GetLong(incoming);
                int hpRegen  = Packet.GetLong(incoming);

                SharedCreature mob = FindMob(mobId);
                if (mob == null) break;

                bool levelChanged = mob.level != levelPrev;
                mob.UpdateStats(hpMax, hp, hpRegen);

                if (levelChanged)
                {
                    ShowLevelupParticles(mob);
                    mob.RedrawOverhead();
                }
                break;
            }

            // -----------------------------------------------------------------
            // 0x4B — INCREASE_HP
            // A combatant was healed.
            // Wire: i32(combat_id) + i32(amount)
            //   combat_id may be "self" sentinel; remap to local player id.
            // -----------------------------------------------------------------
            case 0x4B:
            {
                int combatId = Packet.GetLong(incoming);
                int amount   = Packet.GetLong(incoming);

                combatId = RemapSelfId(combatId);

                if (active_combatants.TryGetValue(combatId, out var combatant))
                    combatant.IncreaseHp(amount);
                break;
            }

            // -----------------------------------------------------------------
            // 0x4C — SHOW_EXP_RECEIVE
            // Show a floating overhead notification (e.g. "+50 XP").
            // Wire: Str(text) + Pos
            // -----------------------------------------------------------------
            case 0x4C:
            {
                string text = Packet.GetString(incoming);
                Pos    pos  = Packet.GetPos(incoming);
                GameController.ShowOverheadNotif(text, pos);
                break;
            }

            // -----------------------------------------------------------------
            // 0x4E — COMPANION_CHANGE_EQUIP
            // A companion's equipment changed (hat/body/hand).
            // Wire: i32(mob_id) + Item(hat) + Item(body) + Item(hand)
            // Only applies if companion type == 6 and not a breeder companion.
            // -----------------------------------------------------------------
            case 0x4E:
            {
                int  mobId = Packet.GetLong(incoming);
                Item hat   = ReadInventoryItem(incoming);
                Item body  = ReadInventoryItem(incoming);
                Item hand  = ReadInventoryItem(incoming);

                SharedCreature mob = FindMob(mobId);
                if (mob != null && mob.companionType == 6 && !mob.isBreeder)
                    mob.OnEquipmentChanged(hat, body, hand);
                break;
            }

            // -----------------------------------------------------------------
            // 0x4F — RENAME_COMPANION
            // A companion was renamed.
            // Wire: i32(mob_id) + Str(new_name)
            // -----------------------------------------------------------------
            case 0x4F:
            {
                int    mobId   = Packet.GetLong(incoming);
                string newName = Packet.GetString(incoming);

                SharedCreature mob = FindMob(mobId);
                if (mob == null) break;

                mob.AssignOverheadName(newName);
                mob.parentName = newName;
                mob.RedrawLevelDisplay();
                break;
            }

            // -----------------------------------------------------------------
            // 0x50 — DESTROY_COMPANION
            // A companion should be destroyed (e.g. returned to inventory).
            // Wire: i32(mob_id)
            // -----------------------------------------------------------------
            case 0x50:
            {
                int mobId = Packet.GetLong(incoming);
                SharedCreature mob = FindMob(mobId);
                if (mob != null)
                    Destroy(mob.gameObject);
                break;
            }

            // -----------------------------------------------------------------
            // 0x51 — APPLY_PERK
            // A perk effect is being applied to a target.
            // Wire: Str(effect_name) + i32(caster_level) + i32(target_id)
            //       + PerkData + i32(perk_level) + i32(caster_id) + u8(reapply)
            // -----------------------------------------------------------------
            case 0x51:
            {
                string   effectName  = Packet.GetString(incoming);
                int      casterLevel = Packet.GetLong(incoming);
                int      targetId    = Packet.GetLong(incoming);
                PerkData perkData    = ReadPerkData(incoming);
                int      perkLevel   = Packet.GetLong(incoming);
                int      casterId    = Packet.GetLong(incoming);
                bool     reapply     = Packet.GetByte(incoming) != 0;

                PerkReceiver target = FindPerkReceiver(targetId);
                target?.InitializeDurationTimer(effectName, casterLevel,
                                                perkData, perkLevel,
                                                casterId, reapply);
                break;
            }

            // -----------------------------------------------------------------
            // 0x52 — LAUNCH_PROJECTILE_PERK
            // A perk that launches a projectile is being triggered.
            // Wire: PerkData + i32(perk_level) + i32(target_id)
            //       + i32(caster_id) + i32(caster_level) + Pos(from) + Pos(to)
            // -----------------------------------------------------------------
            case 0x52:
            {
                PerkData perkData    = ReadPerkData(incoming);
                int      perkLevel   = Packet.GetLong(incoming);
                int      targetId    = Packet.GetLong(incoming);
                int      casterId    = Packet.GetLong(incoming);
                int      casterLevel = Packet.GetLong(incoming);
                Pos      fromPos     = Packet.GetPos(incoming);
                Pos      toPos       = Packet.GetPos(incoming);

                PerkControl.LaunchProjectile(perkData, perkLevel, targetId,
                                             casterId, casterLevel, fromPos, toPos);
                break;
            }

            // -----------------------------------------------------------------
            // 0x53 — QUICK_TAG
            // A mob was quick-tagged or un-tagged.
            // Wire: i32(mob_id) + u8(active: 1=tagged, 0=untagged)
            // -----------------------------------------------------------------
            case 0x53:
            {
                int  mobId  = Packet.GetLong(incoming);
                bool active = Packet.GetByte(incoming) == 1;

                SharedCreature mob = FindMob(mobId);
                if (mob != null)
                    mob.is_quick_tagged = active;
                break;
            }

            // -----------------------------------------------------------------
            // 0x54 — ALL_PRE_APPLIED_PERKS
            // Bulk delivery of all perks currently on a target (e.g. on spawn).
            // Wire: i32(target_mob_id) + i16(count)
            //       × (i32(caster_id) + i32(stack_level) + PerkData
            //          + i32(perk_level) + Str(effect_name) + extra)
            // -----------------------------------------------------------------
            case 0x54:
            {
                int   targetMobId = Packet.GetLong(incoming);
                short count       = Packet.GetShort(incoming);

                PerkReceiver target = FindPerkReceiver(targetMobId);

                for (int i = 0; i < count; i++)
                {
                    int      casterId   = Packet.GetLong(incoming);
                    int      stackLevel = Packet.GetLong(incoming);
                    PerkData perkData   = ReadPerkData(incoming);
                    int      perkLevel  = Packet.GetLong(incoming);
                    string   effectName = Packet.GetString(incoming);
                    var      extra      = ReadPerkExtra(incoming);

                    target?.InitializeDurationTimer(effectName, stackLevel,
                                                    perkData, perkLevel,
                                                    casterId, false);
                }
                break;
            }

            // -----------------------------------------------------------------
            // 0x55 — CREATE_PERK_DROP
            // A perk drop (collectible perk orb) appeared in the world.
            // Wire: Pos + Str(effect_name) + PerkData + i32(perk_level)
            //       + i32(caster_id) + i32(caster_level)
            // -----------------------------------------------------------------
            case 0x55:
            {
                Pos      pos         = Packet.GetPos(incoming);
                string   effectName  = Packet.GetString(incoming);
                PerkData perkData    = ReadPerkData(incoming);
                int      perkLevel   = Packet.GetLong(incoming);
                int      casterId    = Packet.GetLong(incoming);
                int      casterLevel = Packet.GetLong(incoming);

                PerkControl.CreateDrop(pos, effectName, perkData,
                                       perkLevel, casterId, casterLevel);
                break;
            }

            // -----------------------------------------------------------------
            // 0x56 — RESPAWN
            // A player respawned after death.
            // Wire: Str(player_name) + OnlinePlayerData
            // Only spawns visually if player is nearby but not yet in combatants.
            // -----------------------------------------------------------------
            case 0x56:
            {
                string         playerName = Packet.GetString(incoming);
                OnlinePlayerData opd       = ReadOnlinePlayerData(incoming);

                if (IsNearby(playerName) && !IsInActiveCombatants(playerName))
                    MobControl.SpawnOtherPlayer(playerName, opd);
                break;
            }

            // -----------------------------------------------------------------
            // 0x58 — UPDATE_SYNCED_TARGETS
            // Update a mob's target list (for mob AI sync).
            // Wire: i32(mob_id) + i16(count) × i32(target_id)
            // -----------------------------------------------------------------
            case 0x58:
            {
                int   mobId = Packet.GetLong(incoming);
                short count = Packet.GetShort(incoming);

                var targets = new List<int>(count);
                for (int i = 0; i < count; i++)
                    targets.Add(Packet.GetLong(incoming));

                SharedCreature mob = FindMob(mobId);
                if (mob != null)
                    mob.target_ids = targets;
                break;
            }

            // -----------------------------------------------------------------
            // 0x5A — BANDIT_FLAG_DESTROYED
            // The bandit camp flag for a camp instance was destroyed.
            // Wire: Str(camp_instance)
            // -----------------------------------------------------------------
            case 0x5A:
            {
                string campInstance = Packet.GetString(incoming);
                BanditCamp camp     = FindBanditCamp(campInstance);
                if (camp != null)
                {
                    camp.is_flag_destroyed = true;
                    if (is_host)
                        camp.SaveToDisk();
                }
                break;
            }

            default:
                Debug.LogWarning($"[GameServerReceiver] Unknown packet id: 0x{packetId:X2}");
                break;
        }
    }

    // =========================================================================
    // Helper method stubs
    // (Implementations exist elsewhere in the codebase)
    // =========================================================================

    // --- Session / init ---
    private void SendInitialPlayerData() { }
    private void ReceiveDaynight(Packet p) { }
    private void RequestZoneData() { }
    private void RegisterNewPlayer(OnlinePlayerData opd, byte[] rest, string zone) { }
    private void RecreateAllCompanions() { }

    // --- Unique IDs ---
    private List<int> GenerateUniqueIds(int count) => new();
    private void SendUniqueIdAssignment(string player, List<int> ids) { }
    private void SendUniqueIdBatch(string player, List<int> ids) { }
    private void RecycleUniqueIds(List<int> ids) { }

    // --- Players ---
    private void ShowChatMessage(string sender, string display, string msg, bool system) { }
    private void AddToRecentlySeenPlayers(string sender, string display) { }
    private void ShowPlayerLogInOrOut(string name, bool login) { }
    private void NewPlayerNearby(Packet p) { }
    private void NearbyPlayerWentAway(string name) { }
    private SharedCreature FindOnlinePlayer(string name) => null;
    private OnlinePlayer FindOnlinePlayerById(int id) => null;
    private bool IsNearby(string name) => false;
    private bool IsInActiveCombatants(string name) => false;
    private void ShowCompanionDeathMessage(string player, string msg) { }

    // --- Zone / chunk ---
    private ZoneData LoadZoneFromDisk(string name) => null;
    private void SendZoneData(string requester, ZoneData zone, byte type) { }
    private void StopZoneDataTimeoutCoroutine() { }
    private void ProcessIncomingZoneData(Packet p) { }
    private void UnknownZoneGotoSpawn() { }
    private ChunkData GetChunk(string zone, string sub, short x, short z) => null;
    private void SendChunkData(string requester, ChunkData chunk, short x, short z) { }
    private ChunkData UnpackChunkData(Packet p) => null;
    private void LoadBanditCamps(ChunkData chunk) { }
    private ZoneData UnpackZoneData(Packet p) => null;
    private void UpdateZoneItemOnChangedOutside(ZoneData zone) { }
    private void ProcessIncomingZoneTrail(Packet p) { }
    private void SendZoneTrailData(List<ZoneData> zones) { }

    // --- Containers ---
    private BasketContents ReadBasketContents(Packet p) => null;
    private void HideAllPopups() { }
    private void SucceedOpenWorldContainer(string id, BasketContents c) { }
    private BasketContents LoadContainerFromDisk(string id) => null;
    private void SendContainerContents(string req, string id, BasketContents c) { }
    private BasketContents GenerateLootChest(string id) => null;

    // --- World building ---
    private Item ReadInventoryItem(Packet p) => null;
    private object ReadBuildKeys(Packet p) => null;
    private bool IsLandClaim(Item item) => false;
    private bool IsMusicBox(Item item) => false;
    private void PlayerBuildAt(string zone, Pos pos, Rot rot, Item item, object keys) { }
    private void PlayerRemoveAt(string zone, Pos pos, Rot rot, Item item, string key) { }
    private void PlayerReplaceAt(string zone, Pos pos, Rot rot, Item n, Item o, string k) { }
    private void AddLandClaimsToNearbyChunks(string zone, Pos pos, Item item, object keys) { }
    private void RemoveLandClaimsFromNearbyChunks(string zone, Pos pos) { }
    private void RemoveSongFromMusicBox(string zone, Pos pos) { }
    private void ModifyLandClaimTimer(string z, Pos p, byte s, string u, string[] e) { }
    private void UpdateLandClaimTimer(string key) { }
    private void RemoveLandClaimTimer(string key) { }

    // --- Transport / teleporters ---
    private void PackPageOfTeleportersByPage(string req) { }
    private void PackPageOfTeleportersByLocation(string req) { }
    private void LayoutCraftingTab() { }
    private TeleporterData ReadTeleporterData(Packet p) => null;
    private void DrawOnlineTeleporterSlot(TeleporterData l, TeleporterData m, TeleporterData r) { }
    private string GetCustomTeleId(string zone, Pos coords) => "";
    private void SaveTeleporterJpegToDisk(string id, byte[] data) { }
    private byte[] LoadTeleporterJpegFromDisk(string zone, Pos coords) => null;
    private void SendTeleporterScreenshot(string req, string zone, Pos pos, byte[] data) { }
    private void CacheTeleporterSprite(string id, Sprite sprite) { }
    private void UpdateTeleporterUiIfOpen(string id) { }
    private void StoreTeleporterMetadata(string id, string title, string desc) { }
    private Texture2D CreateTexture(byte[] data) => null;

    // --- Minigame / pool ---
    private void SendMinigameResponse(string player, byte type) { }
    private void ShowMinigameResponsePopup(string player, byte resp, byte status) { }
    private void ShowDeclinePopup(string owner) { }
    private void ShowLeavePopup(string owner) { }
    private void OpenPoolTable(string owner) { }
    private void ArrangeBalls(Packet p) { }
    private void StartMpGame() { }
    private void PressClose() { }
    private void ShowMinigameExitPopup() { }
    private PoolGameRecording UnpackPoolGameRecording(Packet p) => null;
    private BallPosition ReadBallPosition(Packet p) => default;

    // --- Chairs ---
    private void TrySitInChairObj(string player, string chairId) { }
    private void EndSittingInChair(string player) { }

    // --- Mob / combat ---
    private SharedCreature FindMob(int id) => null;
    private BanditCamp FindBanditCamp(string instance) => null;
    private void SpawnLocalMob(int id) { }
    private int SpawnLocalMob(int id, CreatureStruct cs, Pos pos) => id;
    private void SpawnNetMob(int id, CreatureStruct cs, Pos pos) { }
    private void DestroyMob(int id) { }
    private void SetMobRotation(int id, Rot rot) { }
    private void SendCompanionLeavingChunk(int id) { }
    private void RequestMobData(string player, int id) { }
    private void SendMobDataResponse(string requester, int[] ids) { }
    private bool IsChunkLoaded(object chunk) => false;
    private void ApplyHitToMob(SharedCreature def, int d, int d2, byte flags, int att) { }
    private void OnMobDie(int id, Pos pos, int dt, int rt, string k, byte f, Item e) { }
    private void ShowLevelupParticles(SharedCreature mob) { }
    private int RemapSelfId(int id) => id;
    private CreatureStruct ReadCreatureStruct(Packet p) => default;

    // --- Perks ---
    private PerkData ReadPerkData(Packet p) => null;
    private object ReadPerkExtra(Packet p) => null;
    private PerkReceiver FindPerkReceiver(int id) => null;

    // --- Music ---
    private void online_finger_pressed(string player, byte oct, byte key, byte inst) { }
    private void remove_online_finger_note(string player, byte oct, byte key) { }

    // --- Equip / companions ---
    private void PlayerChangeEquip(string player, byte slot, Item item) { }
    private void OtherPlayerChangeCreatures(string player, string[] names) { }

    // --- Land claims ---
    private void ShowLevelupParticles(object x) { }
}
