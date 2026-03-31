// FriendServerReceiver.OnReceive — Clean C# Reimplementation
//
// Source: IDA decompilation of FriendServerReceiver.OnReceive (3,270 lines).
//
// Switch offset: `switch(GetByte(p) - 6)`, so case N = packet 0x06+N.
// All case comments use the corrected packet IDs.
//
// Role: purely client-side. Every case is received by the client FROM the
// friend server (S→C). There are no host-role forwarded packets here.
//
// Friend status values:
//   0 = offline
//   1 = online (in-game)
//   2 = offline-friend-who-sent-us-a-request (pending incoming)
//   3 = pending outgoing request
//
// UI screen IDs (FriendServerInterface.curr_screen):
//   1 = main friends list
//   2 = register / error
//   3 = login
//   4 = friends list (post-login)
//   5 = error / reconnect
//   6 = private chat
//   7 = server browser

using System;
using System.Collections.Generic;
using UnityEngine;

public partial class FriendServerReceiver : MonoBehaviour
{
    // -------------------------------------------------------------------------
    // State fields
    // -------------------------------------------------------------------------

    private List<Friend> friends = new();
    private List<Trophy> trophies = new();
    private Dictionary<string, Sprite> cached_server_icons = new();
    private List<string> requesting_server_icons = new();
    private List<ServerInfo> public_server_list = new();
    private int total_unread;
    private int give_gems_on_open;
    private bool show_warning_on_open;

    // -------------------------------------------------------------------------
    // OnReceive — main dispatch
    // -------------------------------------------------------------------------

    public void OnReceive(Packet incoming)
    {
        byte packetId = Packet.GetByte(incoming);

        switch (packetId)
        {
            // -----------------------------------------------------------------
            // 0x06 — CONNECTION_STATUS
            // Server is notifying client of connection state or triggering a
            // reconnect attempt. If the friend window is open on the login
            // screen, calls ShowFailedToConnect.
            // -----------------------------------------------------------------
            case 0x06:
            {
                bool inGame = IsInGameScene();
                if (inGame && FriendWindowIsOpen() && FriendServerInterface.Instance.curr_screen == 1)
                    FriendServerInterface.Instance.ShowFailedToConnect("", reconnect: true);

                TryReconnectConnector();
                break;
            }

            // -----------------------------------------------------------------
            // 0x07 — MATH_CHALLENGE (anti-bot)
            // Server sends a math problem; client must solve and reply.
            // Wire: MathProblem (packed)
            // -----------------------------------------------------------------
            case 0x07:
            {
                var problem = new MathProblem();
                problem.Unpack(incoming);
                int solution = problem.Solve();
                FriendServerSender.Instance.SendMathSolution(solution);
                return; // do not fall through
            }

            // -----------------------------------------------------------------
            // 0x08 — SIGNAL_INTENT_REQUEST
            // Server wants to know our login intent before proceeding.
            // If we have no saved username, signal intent=2 (register);
            // otherwise intent=1 (login).
            // -----------------------------------------------------------------
            case 0x08:
            {
                string savedUsername = PlayerData.Instance.GetGlobalString("username");
                int intent = string.IsNullOrEmpty(savedUsername) ? 2 : 1;
                FriendServerSender.Instance.SignalIntent(intent);
                return;
            }

            // -----------------------------------------------------------------
            // 0x09 — AUTH_REQUIRED
            // Server tells us what auth action to take.
            // Wire: u8(type) — 2=go to register screen, 1=send login attempt
            // -----------------------------------------------------------------
            case 0x09:
            {
                byte authType = Packet.GetByte(incoming);
                if (authType == 2)
                    FriendServerConnector.Instance.TryGotoRegisterScreen();
                else if (authType == 1)
                    FriendServerSender.Instance.SendAttemptLogin();
                break;
            }

            // -----------------------------------------------------------------
            // 0x0A — LOGIN_RESULT
            // Response to a login attempt.
            // If not in the login screen context, falls back to setting
            // connector state to 4 (reconnecting).
            //
            // Wire (when on login screen, curr_screen==3):
            //   u8(result_type):
            //     1 = success → Str(username) + Str(display) + Str(validator)
            //     2 = already logged in → Str(error_msg)
            //     3 = wrong password    → Str(error_msg)
            //     4 = not found         → Str(error_msg)
            // -----------------------------------------------------------------
            case 0x0A:
            {
                bool inGame = IsInGameScene();
                if (inGame && FriendWindowIsOpen() && FriendServerInterface.Instance.curr_screen == 3)
                {
                    byte resultType = Packet.GetByte(incoming);
                    switch (resultType)
                    {
                        case 1: // Success
                        {
                            string username  = Packet.GetString(incoming);
                            string display   = Packet.GetString(incoming);
                            string validator = Packet.GetString(incoming);

                            PopupControl.Instance.ShowMessage($"Welcome back, {display}!", dismissable: true);
                            PlayerData.Instance.SetGlobalString("username",      username);
                            PlayerData.Instance.SetGlobalString("display_name",  display);
                            PlayerData.Instance.SetGlobalString("validator",     validator);

                            FriendServerInterface.Instance.ChangeFriendScreen(1);
                            FriendServerSender.Instance.SendAttemptLogin();
                            break;
                        }
                        case 2: // Already logged in
                        case 3: // Wrong password
                        case 4: // Account not found
                        default:
                        {
                            string errorMsg = Packet.GetString(incoming);
                            PopupControl.Instance.ShowMessage(errorMsg, dismissable: true);
                            FriendServerInterface.Instance.ChangeFriendScreen(2);
                            break;
                        }
                    }
                }
                else
                {
                    // Not in the right UI context — set connector to reconnecting state (4).
                    FriendServerConnector.SetState(4);
                }
                break;
            }

            // -----------------------------------------------------------------
            // 0x0B — FRIENDS_LIST (full sync, sent once on login)
            // Large composite packet containing all friend data, ping targets,
            // gem info, and trophies.
            //
            // Wire:
            //   u8(connection_ok: 1=connected/2=failed)
            //   — if ok: set connected=true, StartPinging
            //   Clear friends list
            //   i16(online_count) × [Str(username)+Str(display)+u8(status_byte)
            //                        + if online: UnpackWorldString
            //                          else if has_last_online: Str(datetime)]
            //   i16(offline_count) × [Str(username)+Str(display)]  (status=2)
            //   i16(pending_count) × [Str(username)+Str(display)]  (status=3)
            //   i16(to_ping_count) × [Str(username)+Str(display)+i16(port)]
            //   i16(give_gems_on_open)
            //   u8(show_warning_on_open)
            //   i16(current_session_count)  [logged to console]
            //   Clear trophies list
            //   i16(trophy_count) × 6×Str(trophy fields)
            // -----------------------------------------------------------------
            case 0x0B:
            {
                byte connectionOk = Packet.GetByte(incoming);
                if (connectionOk == 1)
                {
                    FriendServerConnector.Instance.SetConnected(true);
                    FriendServerConnector.Instance.StartPinging();
                }

                // Clear and rebuild friends list.
                friends.Clear();

                short onlineCount = Packet.GetShort(incoming);
                for (int i = 0; i < onlineCount; i++)
                {
                    string username   = Packet.GetString(incoming);
                    string display    = Packet.GetString(incoming);
                    byte   statusByte = Packet.GetByte(incoming);

                    var friend = new Friend(username, statusByte == 1 ? 1 : 0, display);
                    friends.Add(friend);

                    if (statusByte == 1)
                        UnpackWorldString(friend, incoming);
                    else if (Packet.GetByte(incoming) == 1) // has_last_online flag
                    {
                        string lastOnlineStr = Packet.GetString(incoming);
                        if (DateTime.TryParseExact(lastOnlineStr, "o",
                            System.Globalization.CultureInfo.InvariantCulture,
                            System.Globalization.DateTimeStyles.RoundtripKind,
                            out DateTime lastOnline))
                        {
                            friend.last_online = lastOnline;
                        }
                    }
                }

                short offlineCount = Packet.GetShort(incoming);
                for (int i = 0; i < offlineCount; i++)
                {
                    string username = Packet.GetString(incoming);
                    string display  = Packet.GetString(incoming);
                    friends.Add(new Friend(username, 2, display));
                }

                short pendingCount = Packet.GetShort(incoming);
                for (int i = 0; i < pendingCount; i++)
                {
                    string username = Packet.GetString(incoming);
                    string display  = Packet.GetString(incoming);
                    friends.Add(new Friend(username, 3, display));
                }

                // Servers to ping for latency display.
                short toPingCount = Packet.GetShort(incoming);
                if (toPingCount > 0)
                {
                    var pingTargets = new List<ToPing>(toPingCount);
                    for (int i = 0; i < toPingCount; i++)
                    {
                        string username = Packet.GetString(incoming);
                        string address  = Packet.GetString(incoming);
                        short  port     = Packet.GetShort(incoming);
                        pingTargets.Add(new ToPing(username, address, port));
                    }
                    PingController.Instance.PingMany(pingTargets, OnPingManyComplete, this);
                }

                give_gems_on_open    = Packet.GetShort(incoming);
                show_warning_on_open = Packet.GetByte(incoming) != 0;
                short sessionCount   = Packet.GetShort(incoming);
                Debug.Log($"Session count: {sessionCount}");

                // Clear and rebuild trophies.
                trophies.Clear();
                short trophyCount = Packet.GetShort(incoming);
                for (int i = 0; i < trophyCount; i++)
                {
                    string a = Packet.GetString(incoming);
                    string b = Packet.GetString(incoming);
                    string c = Packet.GetString(incoming);
                    string d = Packet.GetString(incoming);
                    string e = Packet.GetString(incoming);
                    string f = Packet.GetString(incoming);
                    trophies.Add(new Trophy(a, b, c, d, e, f));
                }

                FriendServerConnector.Instance.CheckIfAutoLoginShouldBeDisabled();
                PlayerData.Instance.IncrementGlobalShort("login_count");

                if (IsInGameScene() && FriendWindowIsOpen())
                {
                    FriendServerInterface.Instance.ChangeFriendScreen(4);
                    FriendServerSender.Instance.UpdateWorldString();
                }
                break;
            }

            // -----------------------------------------------------------------
            // 0x0C — CONNECTION_STATE_CHANGE
            // Sets FriendServerConnector internal state to 4 (reconnecting).
            // No packet payload.
            // -----------------------------------------------------------------
            case 0x0C:
                FriendServerConnector.SetState(4);
                break;

            // -----------------------------------------------------------------
            // 0x0F — PING (server heartbeat)
            // -----------------------------------------------------------------
            case 0x0F:
                FriendServerConnector.Instance.last_server_ping = DateTime.UtcNow;
                break;

            // -----------------------------------------------------------------
            // 0x10 — JOIN_REQUEST_RESULT
            // Notification that someone just joined (or tried to join) our world.
            // Wire: u8(type) + Str(player_name)
            //   type 0 = joined successfully → show popup, add to friends list
            //   type 1–7 = various errors → show error popup
            //   if in-game and type != 0: ChangeFriendScreen(5)
            // -----------------------------------------------------------------
            case 0x10:
            {
                FriendServerSender.Instance.EndTimeout();
                byte   resultType  = Packet.GetByte(incoming);
                string playerName  = Packet.GetString(incoming);

                if (resultType == 0)
                {
                    // Someone successfully joined.
                    string popup = $"{playerName} joined your world!";
                    PopupControl.Instance.ShowMessage(popup, dismissable: true);

                    // Add/update as a friend entry.
                    friends.Add(new Friend(playerName, 2, playerName));

                    FriendServerConnector.Instance.CheckIfAutoLoginShouldBeDisabled();

                    if (IsInGameScene() && FriendWindowIsOpen())
                        FriendServerInterface.Instance.ChangeFriendScreen(4);
                }
                else
                {
                    // Various join-failed reasons.
                    string errorMsg = BuildJoinErrorMessage(resultType, playerName);
                    PopupControl.Instance.ShowMessage(errorMsg, dismissable: true);

                    if (IsInGameScene() && FriendWindowIsOpen())
                        FriendServerInterface.Instance.ChangeFriendScreen(5);
                }
                break;
            }

            // -----------------------------------------------------------------
            // 0x11 — INCOMING_FRIEND_REQUEST
            // Someone sent us a friend request.
            // Wire: Str(username) + Str(display_name)
            // -----------------------------------------------------------------
            case 0x11:
            {
                string username = Packet.GetString(incoming);
                string display  = Packet.GetString(incoming);

                var friend = new Friend(username, 3, display);
                friends.Add(friend);
                friend.last_online = DateTime.UtcNow;

                if (IsInGameScene())
                {
                    string msg  = $"{friend.username_punctuated} sent you a friend request!";
                    Sprite icon = FriendServerInterface.Instance.icon_got_friend_req;
                    GameplayGUIControl.Instance.ShowNotif(msg, icon, new OnNotifClick(2));
                    FriendServerInterface.Instance?.RedrawFriendsList();
                }
                break;
            }

            // -----------------------------------------------------------------
            // 0x12 — FRIEND_STATUS_UPDATE
            // A friend came online or went offline (full update).
            // Wire: Str(username) + u8(online: 1=online, 0=offline)
            //       + if online: UnpackWorldString
            //         else if u8(has_last_online)==1: Str(datetime_iso)
            // -----------------------------------------------------------------
            case 0x12:
            {
                FriendServerSender.Instance.EndTimeout();
                string username = Packet.GetString(incoming);
                byte   online   = Packet.GetByte(incoming);

                Friend friend = GetFriendByUsername(username);
                if (friend == null) break;

                if (online == 1)
                {
                    friend.status_t = 1;
                    UnpackWorldString(friend, incoming);
                }
                else
                {
                    friend.status_t = 0;
                    if (Packet.GetByte(incoming) == 1)
                    {
                        string lastOnlineStr = Packet.GetString(incoming);
                        if (DateTime.TryParseExact(lastOnlineStr, "o",
                            System.Globalization.CultureInfo.InvariantCulture,
                            System.Globalization.DateTimeStyles.RoundtripKind,
                            out DateTime dt))
                        {
                            friend.last_online = dt;
                        }
                    }
                }

                FriendServerConnector.Instance.CheckIfAutoLoginShouldBeDisabled();
                if (IsInGameScene() && FriendWindowIsOpen())
                    FriendServerInterface.Instance.RedrawFriendsList();
                break;
            }

            // -----------------------------------------------------------------
            // 0x13 — FRIEND_REQUEST_RESOLVED
            // Our outgoing friend request was accepted or declined.
            // Wire: Str(username) + Str(display_name) + u8(status: 1=accepted)
            // -----------------------------------------------------------------
            case 0x13:
            {
                string username = Packet.GetString(incoming);
                string display  = Packet.GetString(incoming);
                byte   status   = Packet.GetByte(incoming);

                Friend friend = GetFriendByUsername(username);
                if (friend == null) break;

                friend.recently_seen_players = display; // display name field

                if (status == 1)
                {
                    friend.status_t = 1;
                    UnpackWorldString(friend, incoming);

                    if (IsInGameScene())
                    {
                        string msg  = $"{display} accepted your friend request!";
                        Sprite icon = FriendServerInterface.Instance.icon_accepted_friend_req;
                        GameplayGUIControl.Instance.ShowNotif(msg, icon, new OnNotifClick(2));
                        FriendServerInterface.Instance?.RedrawFriendsList();
                    }
                }
                else
                {
                    friend.status_t = 0;
                }
                break;
            }

            // -----------------------------------------------------------------
            // 0x14 — FRIEND_REMOVED_US
            // A friend removed us from their list.
            // Wire: Str(username)
            // -----------------------------------------------------------------
            case 0x14:
            {
                FriendServerSender.Instance.EndTimeout();
                string username = Packet.GetString(incoming);

                Friend friend = GetFriendByUsername(username);
                friends.Remove(friend);

                FriendServerConnector.Instance.CheckIfAutoLoginShouldBeDisabled();
                if (IsInGameScene() && FriendWindowIsOpen())
                    FriendServerInterface.Instance.RedrawFriendsList();
                break;
            }

            // -----------------------------------------------------------------
            // 0x15 — WE_REMOVED_FRIEND (server confirmed)
            // Server confirmed our remove-friend action.
            // Wire: Str(username)
            // -----------------------------------------------------------------
            case 0x15:
            {
                string username = Packet.GetString(incoming);
                Friend friend = GetFriendByUsername(username);
                friends.Remove(friend);

                FriendServerConnector.Instance.CheckIfAutoLoginShouldBeDisabled();
                if (IsInGameScene() && FriendWindowIsOpen())
                    FriendServerInterface.Instance.RedrawFriendsList();
                break;
            }

            // -----------------------------------------------------------------
            // 0x16 — FRIEND_CAME_ONLINE
            // A friend just came online with world info.
            // Wire: Str(username) + UnpackWorldString data
            // Shows a "friend is online" notification.
            // -----------------------------------------------------------------
            case 0x16:
            {
                string username = Packet.GetString(incoming);
                Friend friend = GetFriendByUsername(username);
                if (friend == null) break;

                friend.status_t = 1;
                UnpackWorldString(friend, incoming);

                if (IsInGameScene())
                {
                    string msg  = $"{friend.username_punctuated} is now online!";
                    Sprite icon = FriendServerInterface.Instance.icon_friend_login;
                    GameplayGUIControl.Instance.ShowNotif(msg, icon, new OnNotifClick(2));
                }
                break;
            }

            // -----------------------------------------------------------------
            // 0x17 — FRIEND_WENT_OFFLINE
            // A friend logged off.
            // Wire: Str(username)
            // Clears their chat unread count, sets last_online = now.
            // -----------------------------------------------------------------
            case 0x17:
            {
                string username = Packet.GetString(incoming);
                Friend friend = GetFriendByUsername(username);
                if (friend == null) break;

                friend.status_t = 0;
                friend.last_online = DateTime.UtcNow;

                // Clear their unread chat messages.
                if (friend.chat != null)
                {
                    total_unread -= friend.chat.n_unread;
                    friend.chat.n_unread = 0;
                    friend.chat.entries.Clear();
                }

                if (IsInGameScene() && FriendWindowIsOpen())
                {
                    FriendServerInterface.Instance.RedrawGlobalNotificationCounter();
                    int screen = FriendServerInterface.Instance.curr_screen;
                    if (screen == 4)
                        FriendServerInterface.Instance.RedrawFriendsList();
                    else if (screen == 6)
                        FriendServerInterface.Instance.ChangeFriendScreen(4);
                }
                break;
            }

            // -----------------------------------------------------------------
            // 0x18 — FRIEND_BLOCKED_US
            // A friend blocked us (we are removed from their list).
            // Wire: Str(username)
            // -----------------------------------------------------------------
            case 0x18:
            {
                FriendServerSender.Instance.EndTimeout();
                string username = Packet.GetString(incoming);
                Friend friend = GetFriendByUsername(username);
                friends.Remove(friend);

                FriendServerConnector.Instance.CheckIfAutoLoginShouldBeDisabled();
                if (IsInGameScene() && FriendWindowIsOpen())
                    FriendServerInterface.Instance.RedrawFriendsList();
                break;
            }

            // -----------------------------------------------------------------
            // 0x19 — WE_BLOCKED_FRIEND (server confirmed)
            // Server confirmed our block action.
            // Wire: Str(username)
            // If the friend's chat screen is open, closes it.
            // -----------------------------------------------------------------
            case 0x19:
            {
                string username = Packet.GetString(incoming);
                Friend friend = GetFriendByUsername(username);
                friends.Remove(friend);

                FriendServerConnector.Instance.CheckIfAutoLoginShouldBeDisabled();

                if (IsInGameScene() && FriendWindowIsOpen())
                {
                    int screen = FriendServerInterface.Instance.curr_screen;
                    if (screen == 4)
                        FriendServerInterface.Instance.RedrawFriendsList();
                    else if (screen == 6 &&
                             FriendServerInterface.Instance.curr_chat_username == username)
                        FriendServerInterface.Instance.ChangeFriendScreen(4);
                }
                break;
            }

            // -----------------------------------------------------------------
            // 0x1A — FRIEND_CHAT_MESSAGE
            // A friend sent us a chat message.
            // Wire: Str(from_username) + Str(message)
            // If the friend window is open: FriendChatReceived;
            // else: increment unread counters.
            // -----------------------------------------------------------------
            case 0x1A:
            {
                string from    = Packet.GetString(incoming);
                string message = Packet.GetString(incoming);

                Friend friend = GetFriendByUsername(from);
                if (friend == null) break;

                string display = $"{friend.username_punctuated} says: {message}";
                var    log     = new chat_log(display, message, icon: null,
                                             is_invite: false, actions: null);
                friend.chat.AddLog(log);

                if (IsInGameScene() && FriendWindowIsOpen())
                    FriendServerInterface.Instance.FriendChatReceived(friend, log);
                else
                {
                    friend.chat.n_unread++;
                    total_unread++;
                }
                break;
            }

            // -----------------------------------------------------------------
            // 0x1B — RELAY_SESSION_STATUS
            // Notifies the client whether a relay (join) is in progress or ending.
            // Wire: u8(status: 1=joining, 0=leaving)
            // Only processes if in-game.
            // -----------------------------------------------------------------
            case 0x1B:
            {
                if (!IsInGameScene()) break;
                byte status = Packet.GetByte(incoming);
                if (status == 1)
                    PopupControl.Instance.ShowConnecting("Joining...");
                else if (status == 0)
                    PopupControl.Instance.ShowConnecting("Leaving...");
                break;
            }

            // -----------------------------------------------------------------
            // 0x1D — PUBLIC_SERVER_LIST
            // Server sends the list of public game servers.
            // Wire: u8(count) × ServerInfo (6×Str + 2×i16 + Str)
            // Inserts into public_server_list sorted by player count.
            // -----------------------------------------------------------------
            case 0x1D:
            {
                FriendServerSender.Instance.EndTimeout();

                var servers = new List<ServerInfo>();
                byte count = Packet.GetByte(incoming);
                for (int i = 0; i < count; i++)
                {
                    string name      = Packet.GetString(incoming);
                    string ip        = Packet.GetString(incoming);
                    string region    = Packet.GetString(incoming);
                    string desc      = Packet.GetString(incoming);
                    string owner     = Packet.GetString(incoming);
                    short  players   = Packet.GetShort(incoming);
                    short  maxPlayers = Packet.GetShort(incoming);
                    string gameMode  = Packet.GetString(incoming);
                    servers.Add(new ServerInfo(name, ip, region, desc, owner,
                                               players, maxPlayers, gameMode));
                }

                // Insertion-sort into public_server_list by player count (desc).
                public_server_list.Clear();
                foreach (var server in servers)
                    InsertSortedByPlayerCount(public_server_list, server);

                if (IsInGameScene() && FriendWindowIsOpen())
                    FriendServerInterface.Instance.RedrawServerList();
                break;
            }

            // -----------------------------------------------------------------
            // 0x1E — JOIN_WORLD_RESULT
            // Result of attempting to join a public/listed game world.
            // Wire: u8(result: 1=success, 0=fail)
            // If in-game on screen 1, navigates to screen 7 (server browser).
            // -----------------------------------------------------------------
            case 0x1E:
            {
                FriendServerSender.Instance.EndTimeout();
                byte result = Packet.GetByte(incoming);

                if (result == 1)
                    PopupControl.Instance.ShowMessage("Join successful!", dismissable: true);
                else if (result == 0)
                    PopupControl.Instance.ShowMessage("Failed to join.", dismissable: true);
                else
                    break;

                if (IsInGameScene() && FriendWindowIsOpen() &&
                    FriendServerInterface.Instance.curr_screen == 1)
                {
                    FriendServerInterface.Instance.ChangeFriendScreen(7);
                }
                break;
            }

            // -----------------------------------------------------------------
            // 0x1F — SERVER_ICON_DATA
            // Icon image for a public server listing.
            // Wire: Str(server_id) + u8(has_data)
            //       + if has_data: i16(byte_count) + byte_count×u8(jpeg)
            // Caches Sprite in cached_server_icons.
            // -----------------------------------------------------------------
            case 0x1F:
            {
                string serverId = Packet.GetString(incoming);
                byte   hasData  = Packet.GetByte(incoming);

                requesting_server_icons.Remove(serverId);

                Sprite sprite = null;
                if (hasData == 1)
                {
                    short  byteCount = Packet.GetShort(incoming);
                    byte[] jpegBytes = Packet.GetBytes(incoming, byteCount);

                    var tex = new Texture2D(32, 32);
                    ImageConversion.LoadImage(tex, jpegBytes);
                    var rect   = new Rect(0, 0, tex.width, tex.height);
                    var pivot  = new Vector2(0.5f, 0.5f);
                    sprite = Sprite.Create(tex, rect, pivot);
                }

                cached_server_icons[serverId] = sprite;

                if (IsInGameScene() && FriendWindowIsOpen())
                    FriendServerInterface.Instance.GotServerIcon(serverId);
                break;
            }

            // -----------------------------------------------------------------
            // 0x20 — JUMP_TO_GAME (begin relay ping phase)
            // Server initiates the join-world flow by sending ping targets.
            // Client pings them and then connects to the best one.
            // Wire: u8(type: 1=joining, other=leaving) + i16(ping_count)
            //       × (Str(username) + Str(address) + i16(port))
            // Only processes if in-game.
            // -----------------------------------------------------------------
            case 0x20:
            {
                if (!IsInGameScene()) break;

                byte joinType = Packet.GetByte(incoming);
                string connectingMsg = joinType == 1 ? "Joining..." : "Leaving...";

                WindowControl.Instance.CloseAllWindows();
                PopupControl.Instance.ShowConnecting(connectingMsg);

                short pingCount = Packet.GetShort(incoming);
                if (pingCount < 1) break;

                var pingTargets = new List<ToPing>(pingCount);
                for (int i = 0; i < pingCount; i++)
                {
                    string username = Packet.GetString(incoming);
                    string address  = Packet.GetString(incoming);
                    short  port     = Packet.GetShort(incoming);
                    pingTargets.Add(new ToPing(username, address, port));
                }

                // After pinging, callback fires with best latencies → connect.
                PingController.Instance.PingMany(pingTargets, OnJumpToGamePingsComplete, this);
                break;
            }

            // -----------------------------------------------------------------
            // 0x23 — KICKED_OR_BANNED
            // Server kicked or banned this client.
            // Only processes if in-game; shows a kicked/banned message.
            // -----------------------------------------------------------------
            case 0x23:
            {
                if (!IsInGameScene()) break;
                PopupControl.Instance.ShowMessage("You have been removed from the session.", dismissable: true);
                break;
            }

            // -----------------------------------------------------------------
            // 0x25 — CONNECT_TO_GAME_SERVER
            // Server sends game server connection credentials.
            // Wire: Str(host_display_name) + Str(world_name) + Str(ip)
            //       + Str(auth_token) + i16(port) + u8(unused)
            //
            // If in-game: shows "Connecting to [host]..." and calls
            //   GameServerConnector.ConnectToGameServer(ip, auth_token, port, world_name)
            // If not in-game (e.g. on title screen): just EndTimeout.
            // -----------------------------------------------------------------
            case 0x25:
            {
                if (IsInGameScene())
                {
                    string hostDisplay = Packet.GetString(incoming);
                    string worldName   = Packet.GetString(incoming);
                    string ip          = Packet.GetString(incoming);
                    string authToken   = Packet.GetString(incoming);
                    short  port        = Packet.GetShort(incoming);
                    Packet.GetByte(incoming); // unused byte

                    // Strip display name formatting characters for comparison.
                    string hostClean = hostDisplay.Replace("*", "");
                    string myDisplay = PlayerData.Instance.GetGlobalString("display_name");

                    string popup = hostClean == myDisplay
                        ? "Connecting..."
                        : $"Connecting to {hostClean}...";
                    PopupControl.Instance.ShowConnecting(popup);

                    GameServerConnector.Instance.ConnectToGameServer(ip, authToken, port, worldName);
                }
                else
                {
                    FriendServerSender.Instance.EndTimeout();
                }
                break;
            }

            // -----------------------------------------------------------------
            // 0x27 — ERROR_WRONG_SCREEN
            // Server error; shown when client is on wrong screen or timed out.
            // If on screen 1, navigates to screen 6 (chat).
            // -----------------------------------------------------------------
            case 0x27:
            {
                FriendServerSender.Instance.EndTimeout();
                if (!IsInGameScene()) break;

                PopupControl.Instance.ShowMessage("An error occurred.", dismissable: true);

                if (FriendWindowIsOpen() && FriendServerInterface.Instance.curr_screen == 1)
                    FriendServerInterface.Instance.ChangeFriendScreen(6);
                break;
            }

            // -----------------------------------------------------------------
            // 0x28 — FRIEND_SENT_INVITE
            // A friend invited us to join their world.
            // Wire: Str(from_username) + u8(slot)
            //       + if slot 1 or 2: Str(world_info)
            // Creates a chat_log entry with accept/decline action buttons.
            // -----------------------------------------------------------------
            case 0x28:
            {
                string fromUsername = Packet.GetString(incoming);
                byte   slot         = Packet.GetByte(incoming);
                string worldInfo    = (slot == 1 || slot == 2)
                    ? Packet.GetString(incoming)
                    : "";

                Friend friend = GetFriendByUsername(fromUsername);
                if (friend == null) break;

                friend.chat.DisableOldInvites();

                string display = $"{friend.username_punctuated} invited you to their world!";
                var actions = new Dictionary<string, string>
                {
                    { "type",    "invite" },
                    { "action",  "join" },
                    { "slot",    slot.ToString() },
                    { "world",   worldInfo },
                    { "from",    fromUsername },
                };
                var log = new chat_log(display, "invite",
                                       icon: FriendServerInterface.Instance.icon_invite,
                                       is_invite: false, actions: actions);
                friend.chat.AddLog(log);

                if (IsInGameScene() && FriendWindowIsOpen())
                    FriendServerInterface.Instance.FriendChatReceived(friend, log);
                else
                {
                    friend.chat.n_unread++;
                    total_unread++;
                }
                break;
            }

            // -----------------------------------------------------------------
            // 0x29 — AUTO_ACCEPT_JOIN
            // Server tells us to auto-accept a pending join request.
            // Wire: Str(requester_username)
            // If in-game: send YouMayJoinMyWorldNow
            // If not: send AcceptInviteFailed
            // -----------------------------------------------------------------
            case 0x29:
            {
                string requester = Packet.GetString(incoming);
                if (IsInGameScene())
                    FriendServerSender.Instance.SendYouMayJoinMyWorldNow(requester, slot: 0, worldInfo: "");
                else
                    FriendServerSender.Instance.SendAcceptInviteFailed(requester, slot: 0);
                break;
            }

            // -----------------------------------------------------------------
            // 0x2A — INVITE_RESPONSE
            // The player we invited responded to our invite.
            // Wire: Str(player_name) + u8(response: 1=accepted, 0=declined)
            // -----------------------------------------------------------------
            case 0x2A:
            {
                string playerName = Packet.GetString(incoming);
                byte   response   = Packet.GetByte(incoming);

                string msg = response == 1
                    ? $"{playerName} accepted your invite!"
                    : $"{playerName} declined your invite.";
                PopupControl.Instance.ShowMessage(msg, dismissable: true);
                break;
            }

            // -----------------------------------------------------------------
            // 0x2B — HIDE_ALL_POPUPS
            // Server requests all popups be dismissed.
            // Only processes if in-game.
            // -----------------------------------------------------------------
            case 0x2B:
            {
                if (!IsInGameScene()) break;
                PopupControl.Instance.HideAll();
                break;
            }

            // -----------------------------------------------------------------
            // 0x2C — FRIEND_WORLD_UPDATED
            // A friend's world info string changed (e.g. moved zones).
            // Wire: Str(username) + UnpackWorldString data
            // If their chat is open, redraws the chat view.
            // -----------------------------------------------------------------
            case 0x2C:
            {
                string username = Packet.GetString(incoming);
                Friend friend = GetFriendByUsername(username);
                if (friend == null) break;

                UnpackWorldString(friend, incoming);

                if (IsInGameScene() && FriendWindowIsOpen())
                {
                    int screen = FriendServerInterface.Instance.curr_screen;
                    if (screen == 4)
                        FriendServerInterface.Instance.RedrawFriendsList();
                    else if (screen == 6 &&
                             FriendServerInterface.Instance.curr_chat_username == username)
                        FriendServerInterface.Instance.RedrawChat(friend.chat);
                }
                break;
            }

            // -----------------------------------------------------------------
            // 0x2D — JOIN_REQUEST_RECEIVED
            // Another player wants to join our world.
            // Wire: Str(from_username) + u8(slot)
            //       + if slot 1 or 2: Str(world_info)
            // Creates a join-request chat_log with accept/decline action buttons.
            // -----------------------------------------------------------------
            case 0x2D:
            {
                string fromUsername = Packet.GetString(incoming);
                byte   slot         = Packet.GetByte(incoming);
                string worldInfo    = (slot == 1 || slot == 2)
                    ? Packet.GetString(incoming)
                    : "";

                Friend friend = GetFriendByUsername(fromUsername);
                if (friend == null) break;

                friend.chat.DisableOldJoins();

                string display = $"{friend.username_punctuated} wants to join your world!";
                var actions = new Dictionary<string, string>
                {
                    { "type",    "join_request" },
                    { "action",  "join" },
                    { "slot",    slot.ToString() },
                    { "world",   worldInfo },
                    { "from",    fromUsername },
                };
                var log = new chat_log(display, "join_request",
                                       icon: FriendServerInterface.Instance.icon_want_to_join,
                                       is_invite: false, actions: actions);
                friend.chat.AddLog(log);

                if (IsInGameScene() && FriendWindowIsOpen())
                    FriendServerInterface.Instance.FriendChatReceived(friend, log);
                else
                {
                    friend.chat.n_unread++;
                    total_unread++;
                }
                break;
            }

            // -----------------------------------------------------------------
            // 0x2E — GENERIC_ERROR_MESSAGE
            // Server sends a generic error (e.g. session full, banned).
            // -----------------------------------------------------------------
            case 0x2E:
                PopupControl.Instance.ShowMessage("You cannot join this session.", dismissable: true);
                break;

            // -----------------------------------------------------------------
            // 0x2F — WARNING_NOTIFICATION
            // Wire: u8(warning_type) — 0 means no warning
            // -----------------------------------------------------------------
            case 0x2F:
            {
                byte warningType = Packet.GetByte(incoming);
                if (warningType != 0)
                    ShowWarning(warningType);
                break;
            }

            // -----------------------------------------------------------------
            // 0x34 — RECEIVE_GEMS
            // Server is giving the client in-game gems.
            // Wire: i16(amount)
            // -----------------------------------------------------------------
            case 0x34:
            {
                short amount = Packet.GetShort(incoming);
                ShowReceiveGems(amount);
                break;
            }

            // -----------------------------------------------------------------
            // 0x37 — RECEIVE_TROPHY
            // Server is awarding a trophy.
            // Wire: 6×Str(trophy fields)
            // Adds to trophies list; if friend list is open, redraws it.
            // -----------------------------------------------------------------
            case 0x37:
            {
                string a = Packet.GetString(incoming);
                string b = Packet.GetString(incoming);
                string c = Packet.GetString(incoming);
                string d = Packet.GetString(incoming);
                string e = Packet.GetString(incoming);
                string f = Packet.GetString(incoming);
                trophies.Add(new Trophy(a, b, c, d, e, f));

                if (IsInGameScene() && FriendWindowIsOpen())
                {
                    if (FriendServerInterface.Instance.curr_screen == 4)
                        FriendServerInterface.Instance.RedrawFriendsList();
                    FriendServerInterface.Instance.ShowNewGiftsNotif();
                }
                break;
            }

            // -----------------------------------------------------------------
            // 0x3E — FAILED_TO_CONNECT
            // Server notifies that connection to a game server failed.
            // If the friend window is open on the main screen, shows the error.
            // -----------------------------------------------------------------
            case 0x3E:
            {
                bool inGame = IsInGameScene();
                if (inGame && FriendWindowIsOpen() && FriendServerInterface.Instance.curr_screen == 1)
                    FriendServerInterface.Instance.ShowFailedToConnect("", reconnect: false);

                TryReconnectConnector();
                break;
            }

            default:
                Debug.LogWarning($"[FriendServerReceiver] Unknown packet id: 0x{packetId:X2}");
                break;
        }
    }

    // =========================================================================
    // Helper method stubs
    // =========================================================================

    private bool IsInGameScene() => false; // check scene name == "Main" (or equiv)
    private bool FriendWindowIsOpen() => WindowControl.Instance?.curr_miniwindow == 14;
    private void TryReconnectConnector() { }
    private Friend GetFriendByUsername(string username) => null;
    private void UnpackWorldString(Friend friend, Packet p) { }
    private void OnPingManyComplete(Dictionary<string, short> results) { }
    private void OnJumpToGamePingsComplete(Dictionary<string, short> results) { }
    private string BuildJoinErrorMessage(byte errorType, string playerName) => "";
    private void InsertSortedByPlayerCount(List<ServerInfo> list, ServerInfo s) { }
    private void ShowWarning(byte warningType) { }
    private void ShowReceiveGems(short amount) { }
}
