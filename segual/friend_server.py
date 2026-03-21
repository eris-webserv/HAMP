import socket, struct, random, string, json, os, binascii, threading, time
import urllib.request
import subprocess # Added to spin up your game_server.py

# --- CONFIGURATION ---
HOST, PORT = '0.0.0.0', 7002
DB_FILE = 'players.json'

# --- IP CONFIGURATION ---
# Set FORCE_LOCAL_IP to an empty string to use your public IP (requires port forwarding).
# Set it to '127.0.0.1' for same-machine testing, or your LAN IP (e.g. '192.168.1.x')
# for clients on the same local network.
FORCE_LOCAL_IP = 'put ur ip here'

if FORCE_LOCAL_IP:
    RELAY_IP = FORCE_LOCAL_IP
    print(f"[*] Using forced IP for Relay routing: {RELAY_IP}")
else:
    print("[*] Fetching Public IP for Relay routing...")
    try:
        RELAY_IP = urllib.request.urlopen('https://api.ipify.org').read().decode('utf8')
        print(f"[*] Dynamic Public IP set to: {RELAY_IP}")
    except Exception as e:
        RELAY_IP = '127.0.0.1'
        print(f"[!] Failed to fetch Public IP: {e}. Defaulting to localhost.")

ID_MAP = {
    0x0A: "REGISTER_REQ", 0x0B: "LOGIN", 0x10: "ADD_FRIEND", 0x11: "PUSH_REQ", 
    0x12: "ACCEPT_FRIEND", 0x13: "PUSH_ACCEPTED", 0x14: "DECLINE_FRIEND", 
    0x15: "PUSH_REMOVED", 0x18: "REMOVE_FRIEND", 0x0F: "HEARTBEAT", 
    0x2C: "WORLD_UPDATE", 0x16: "FR_ONLINE", 0x17: "FR_OFFLINE", 
    0x1A: "PRIVATE_MSG", 0x2D: "JOIN_REQ", 0x2B: "JOIN_GRANT",
    0x25: "JUMP_SIGNAL"
}

def load_db():
    if os.path.exists(DB_FILE):
        try:
            with open(DB_FILE, 'r', encoding='utf-8') as f: return json.load(f)
        except: pass
    return {"__config__": {"admin_console_enabled": True}}

def save_db(db):
    with open(DB_FILE, 'w', encoding='utf-8') as f: json.dump(db, f, indent=4)

def pack_string(s):
    encoded = s.encode('utf-16-le')
    return struct.pack('<H', len(encoded)) + encoded

def unpack_string(data, start_offset):
    try:
        length = struct.unpack('<H', data[start_offset:start_offset+2])[0]
        val = data[start_offset+2 : start_offset+2+length].decode('utf-16-le')
        return val, start_offset + 2 + length
    except: return "", start_offset

def decode_payload(data):
    results = []
    i = 0
    while i < len(data) - 2:
        try:
            length = struct.unpack('<H', data[i:i+2])[0]
            if 0 < length < 256 and i + 2 + length <= len(data):
                val = data[i+2 : i+2+length].decode('utf-16-le')
                results.append(f'"{val}"')
                i += 2 + length; continue
        except: pass
        i += 1
    return " | ".join(results)

def craft_batch(qid, payload):
    return struct.pack('<HBBB I', 9 + len(payload), 1, qid, 3, len(payload)) + payload

active_sessions = {} # user_low -> (conn, addr)
world_states = {}
DEFAULT_WORLD = b'\x01\x00\x00\x00\x00\x00\x00'
active_game_servers = {} # user_low -> dyn_port (tracks which game server a user is in) 

def send_packet(conn, qid, payload, label="UNKNOWN"):
    try:
        batch = craft_batch(qid, payload)
        print(f"[SERVER -> CLIENT] [{label}]")
        decoded = decode_payload(payload)
        if decoded: print(f"    DECODED: {decoded}")
        conn.sendall(batch)
    except: pass

def handle_client(conn, addr):
    current_user = None
    try:
        while True:
            data = conn.recv(8192)
            if not data: break
            if data.startswith(b'\x66'): send_packet(conn, 0, b'\x09\x01', "HANDSHAKE"); continue
            if len(data) < 10: continue
            
            db, packet_id = load_db(), data[9]
            p_name = ID_MAP.get(packet_id, f"ID_{packet_id:02X}")
            print(f"\n[CLIENT -> SERVER] [{p_name}]")

            if packet_id == 0x0B: # LOGIN
                raw_u, off = unpack_string(data, 10); user = raw_u.lower(); token, _ = unpack_string(data, off)
                if user in db:
                    current_user = user; active_sessions[user] = (conn, addr)
                    world_states.setdefault(user, DEFAULT_WORLD)
                    u_data = db[user]; resp = b'\x0B\x01' + struct.pack('<H', len(u_data.get("friends", [])))
                    
                    friends_list = u_data.get("friends", [])
                    for f in friends_list:
                        f_low = f.lower(); f_disp = db.get(f_low, {}).get("display", f)
                        is_on = 1 if f_low in active_sessions else 0
                        resp += pack_string(f_low) + pack_string(f_disp) + struct.pack('<B', is_on)
                        if is_on: resp += world_states.get(f_low, DEFAULT_WORLD)
                        else: resp += b'\x00' 
                    
                    for loop in ["pending_inbound", "pending_outbound"]:
                        p_list = u_data.get(loop, [])
                        resp += struct.pack('<H', len(p_list))
                        for p in p_list: resp += pack_string(p.lower()) + pack_string(db.get(p.lower(), {}).get("display", p))
                    
                    send_packet(conn, 2, resp + struct.pack('<H H B HH', 0, 10, 0, 0, 0), "LOGIN_SUCCESS")

                    my_world = world_states.get(current_user, DEFAULT_WORLD)
                    for f in friends_list:
                        f_low = f.lower()
                        if f_low in active_sessions:
                            f_conn = active_sessions[f_low][0]
                            send_packet(conn, 2, b'\x16' + pack_string(f_low) + world_states.get(f_low, DEFAULT_WORLD), "SYNC_ONLINE_FRIEND")
                            send_packet(f_conn, 2, b'\x16' + pack_string(current_user) + my_world, "NOTIFY_FRIEND_ONLINE")
                        else:
                            send_packet(conn, 2, b'\x17' + pack_string(f_low), "SYNC_OFFLINE_FRIEND")

            elif packet_id == 0x10: # ADD FRIEND
                raw_t, _ = unpack_string(data, 10); t = raw_t.lower()
                if t in db and t != current_user:
                    t_disp = db[t]['display']
                    send_packet(conn, 2, b'\x10\x00' + pack_string(t_disp) + pack_string(raw_t), "ADD_OK")
                    if t in active_sessions:
                        send_packet(active_sessions[t][0], 2, b'\x11' + pack_string(current_user) + pack_string(db[current_user]['display']), "PUSH_REQ")

            elif packet_id == 0x1A: # PM RELAY
                raw_t, off = unpack_string(data, 10); t = raw_t.lower(); msg, _ = unpack_string(data, off)
                if t in active_sessions: send_packet(active_sessions[t][0], 2, b'\x1A' + pack_string(current_user) + pack_string(msg), "RELAY_PM")

            elif packet_id == 0x2D: # JOIN_REQ Relay
                target_raw, off = unpack_string(data, 10); target = target_raw.lower()
                if target in active_sessions:
                    req_payload = b'\x2D' + pack_string(current_user) + b'\x00'
                    send_packet(active_sessions[target][0], 2, req_payload, "RELAY_JOIN_REQ_UI")

            elif packet_id == 0x2B: # JOIN_GRANT (Host accepts)
                target_raw, off = unpack_string(data, 10); target = target_raw.lower()

                # Unfreeze Host UI immediately
                send_packet(conn, 2, b'\x2B', "UNFREEZE_HOST")

                if target in active_sessions:
                    room_token = current_user
                    host_disp = db.get(current_user, {}).get("display", current_user)

                    # Check if the host is already in a game server
                    dyn_port = active_game_servers.get(current_user)
                    server_ready = False
                    if dyn_port:
                        print(f"[*] Checking if existing game server on port {dyn_port} is still alive...")
                        try:
                            # Try to connect to the existing port with a short timeout
                            with socket.create_connection(('127.0.0.1', dyn_port), timeout=2):
                                server_ready = True
                                print(f"[*] Server on port {dyn_port} is alive. Reusing.")
                        except (OSError, ConnectionRefusedError):
                            print(f"[!] Server on port {dyn_port} is dead. Removing and spawning new one.")
                            active_game_servers.pop(current_user, None)
                            dyn_port = None

                    if not server_ready:
                        # Spin up a new game_server.py in its own visible CMD window
                        dyn_port = random.randint(7100, 7900)
                        print(f"[*] Launching private session on Port {dyn_port} for room {room_token}...")
                        subprocess.Popen(
                            ['python', '-u', 'game_server.py', str(dyn_port), room_token, "host", room_token],
                            creationflags=subprocess.CREATE_NEW_CONSOLE
                        )

                        # Wait until game_server.py is actually listening (max 5s)
                        ready = False
                        for _ in range(50):
                            try:
                                with socket.create_connection(('127.0.0.1', dyn_port), timeout=1):
                                    ready = True
                                    break
                            except OSError:
                                time.sleep(0.1)
                        if not ready:
                            print(f"[!] Game server on port {dyn_port} did not start in time!")

                        # Track this server for the host
                        active_game_servers[current_user] = dyn_port

                    # Track for the joining guest too
                    active_game_servers[target] = dyn_port

                    # Unfreeze Guest UI
                    send_packet(active_sessions[target][0], 2, b'\x2B', "UNFREEZE_GUEST")

                    # Construct JUMP_SIGNAL (0x25) pointing to the game server port
                    jump = b'\x25'
                    jump += pack_string(host_disp)         # World Name
                    jump += pack_string(room_token)        # Token
                    jump += pack_string(RELAY_IP)          # Primary IP
                    jump += pack_string(RELAY_IP)          # Fallback IP
                    jump += struct.pack('<H', dyn_port)    # DYNAMIC PORT
                    jump += b'\x00'

                    send_packet(active_sessions[target][0], 2, jump, "JUMP_GUEST")
                    send_packet(conn, 2, jump, "JUMP_HOST")

            elif packet_id == 0x2C: # WORLD_UPDATE
                total_len = struct.unpack('<H', data[0:2])[0]
                world_states[current_user] = data[10:total_len]
            
            elif packet_id == 0x0F: send_packet(conn, 2, b'\x0F', "HB")
    except Exception as e: print(f"[!] handle_client error: {e}")
    finally:
        if current_user:
            db = load_db()
            for f in db.get(current_user, {}).get("friends", []):
                f_low = f.lower()
                if f_low in active_sessions:
                    send_packet(active_sessions[f_low][0], 2, b'\x17' + pack_string(current_user), "NOTIFY_OFFLINE")
            if current_user in active_sessions: del active_sessions[current_user]
            active_game_servers.pop(current_user, None)
        conn.close()

def run_server():
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        s.bind((HOST, PORT)); s.listen(); print(f"[*] Friend Server Orchestrator listening on {PORT}...")
        while True:
            c, a = s.accept(); threading.Thread(target=handle_client, args=(c, a), daemon=True).start()

if __name__ == "__main__":
    run_server()