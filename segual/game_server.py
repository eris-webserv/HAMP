import socket, struct, sys, threading, binascii, datetime, os, json, re, time, random, math, hashlib
if hasattr(sys.stdout, 'reconfigure'):
    sys.stdout.reconfigure(line_buffering=True)  # flush every print immediately

# ── LOGGING SETUP ─────────────────────────────────────────────────────────────
LOG_FILENAME = datetime.datetime.now().strftime("log-%Y-%m-%d-%H-%M-%S.txt")
LOG_FILE = open(LOG_FILENAME, "a", encoding="utf-8")

def log_data(direction, data, info=""):
    hex_view = binascii.hexlify(data).decode('utf-8')
    txt_view = "".join(chr(b) if 32 <= b <= 126 else "." for b in data)
    timestamp = datetime.datetime.now().strftime('%H:%M:%S')
    LOG_FILE.write(f"[{timestamp}] [{direction}] {info}\n HEX: {hex_view}\n TXT: {txt_view}\n{'-'*60}\n")
    LOG_FILE.flush()
# ─────────────────────────────────────────────────────────────────────────────

# ── args ────────────────────────────────────────────────────────────────────
if len(sys.argv) < 3:
    print("Usage: game_server.py <port> <room_token>")
    sys.exit(1)

PORT       = int(sys.argv[1])
ROOM_TOKEN = sys.argv[2]
IS_PUBLIC  = True # FORCED DEDICATED MODE for development
IS_PVP     = IS_PUBLIC  # PvP ON for dedicated servers, OFF for friend servers

# Explicit host assignment from arguments (e.g. game_server.py <port> <token> host <hostname>)
host_player = None
if len(sys.argv) >= 5 and sys.argv[3].lower() == "host":
    host_player = sys.argv[4]
    print(f"[*] Pre-assigned HOST: {host_player!r}")
elif len(sys.argv) >= 4 and sys.argv[3].lower() == "host": # handle cases with only 4 args
    # This shouldn't normally happen with our current intercept.py, but for safety:
    pass

# ── inactivity auto-kill ───────────────────────────────────────────────────────
INACTIVITY_TIMEOUT = 30  # seconds
last_activity = time.time()
activity_lock = threading.Lock()

def touch_activity():
    global last_activity
    with activity_lock:
        last_activity = time.time()

def inactivity_watchdog():
    while True:
        time.sleep(5)
        if IS_PUBLIC:
            continue
        with activity_lock:
            elapsed = time.time() - last_activity
        if elapsed >= INACTIVITY_TIMEOUT:
            with players_lock:
                count = len(players)
            if count == 0:
                print(f"[*] No activity for {INACTIVITY_TIMEOUT}s and no players connected. Shutting down.")
                os._exit(0)

threading.Thread(target=inactivity_watchdog, daemon=True).start()

# ── chunk persistence (PUBLIC servers only) ──────────────────────────────────
CHUNK_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), f"chunks_{ROOM_TOKEN}")
if IS_PUBLIC:
    os.makedirs(CHUNK_DIR, exist_ok=True)

# ── host relay state (FRIEND/PRIVATE servers) ────────────────────────────────
# host_player is already initialized from sys.argv at the top of the file
host_lock    = threading.Lock()
host_last_seen = time.time() # For auto-shutdown if host leaves

# Maps for pending relay requests
pending_chunk_requests     = {}
pending_container_requests = {}
pending_lock               = threading.Lock()

def host_watchdog():
    """Shuts down the game server if the host player is gone for too long."""
    while True:
        time.sleep(2)
        if IS_PUBLIC: continue
        
        with host_lock:
            h = host_player
            last = host_last_seen
            
        if h:
            with players_lock:
                host_connected = (h in players)
            
            if not host_connected:
                # Host disconnected from socket
                elapsed = time.time() - last
                if elapsed > 10:
                    print(f"[*] Host {h!r} has been disconnected for >10s. Shutting down friend server.")
                    os._exit(0)
        else:
            # Host hasn't even logged in yet? Give them 30s from server start.
            if time.time() - start_time > 30:
                print("[*] Host never joined within 30s. Shutting down.")
                os._exit(0)

start_time = time.time()
threading.Thread(target=host_watchdog, daemon=True).start()

CONTAINER_FILE = os.path.join(CHUNK_DIR, "_containers.json")

def chunk_path(x, z):
    return os.path.join(CHUNK_DIR, f"x{x}-y{z}")

def load_chunk_data(x, z):
    path = chunk_path(x, z)
    if os.path.exists(path):
        with open(path, 'r', encoding='utf-8') as f:
            return json.load(f)
    return {"builds": []}

def save_chunk_data(x, z, data):
    path = chunk_path(x, z)
    with open(path, 'w', encoding='utf-8') as f:
        json.dump(data, f, indent=2)

def load_containers():
    if os.path.exists(CONTAINER_FILE):
        with open(CONTAINER_FILE, 'r', encoding='utf-8') as f:
            return json.load(f)
    return {}

def save_containers(containers):
    with open(CONTAINER_FILE, 'w', encoding='utf-8') as f:
        json.dump(containers, f, indent=2)

def chunk_str_to_coords(chunk_str):
    """Extract (x, z) from a chunk string like '_overworld_-2_0_'."""
    numbers = re.findall(r'-?\d+', chunk_str)
    if len(numbers) >= 2:
        return int(numbers[-2]), int(numbers[-1])
    return None, None

# ── biome definitions & world generation ─────────────────────────────────────
# ChunkData 4 shorts: biome, floor_model_id, floor_texture_index, floor_rotation
# Biome IDs: 0=grass, 1=desert, 2=snow, 3=evergreen, 4=ocean, 5=swamp, 6=woodlands, 7=sakura
BIOME_GRASS     = 0
BIOME_DESERT    = 1
BIOME_SNOW      = 2
BIOME_EVERGREEN = 3
BIOME_OCEAN     = 4
BIOME_SWAMP     = 5
BIOME_WOODLANDS = 6
BIOME_SAKURA    = 7
BIOME_COUNT     = 8

# World seed derived from room token for deterministic generation
WORLD_SEED = int(hashlib.md5(ROOM_TOKEN.encode()).hexdigest()[:8], 16)

# ── Catalogue mode ───────────────────────────────────────────────────────────
# When enabled, dedicated chunks at (x=50..61, z=0) contain every buildable item
CATALOGUE_ENABLED = False

# Cache of generated biomes: (x, z) → biome_id
biome_cache = {}
biome_cache_lock = threading.Lock()

def _chunk_hash(x, z, layer=0):
    """Deterministic hash for a chunk coordinate, returns float 0.0-1.0."""
    h = hashlib.md5(struct.pack('<iiI', x, z, WORLD_SEED + layer)).digest()
    return int.from_bytes(h[:4], 'little') / 0xFFFFFFFF

def _noise2d(x, z, scale=1.0, layer=0):
    """Simple value noise with bilinear interpolation for smooth gradients."""
    sx = x / scale
    sz = z / scale
    ix, iz = int(math.floor(sx)), int(math.floor(sz))
    fx, fz = sx - ix, sz - iz
    # Smoothstep
    fx = fx * fx * (3 - 2 * fx)
    fz = fz * fz * (3 - 2 * fz)
    # Corner values
    v00 = _chunk_hash(ix,     iz,     layer)
    v10 = _chunk_hash(ix + 1, iz,     layer)
    v01 = _chunk_hash(ix,     iz + 1, layer)
    v11 = _chunk_hash(ix + 1, iz + 1, layer)
    # Bilinear interpolation
    top = v00 + (v10 - v00) * fx
    bot = v01 + (v11 - v01) * fx
    return top + (bot - top) * fz

def _multi_octave_noise(x, z, octaves=3, base_scale=8.0, layer=0):
    """Layered noise for more natural-looking terrain."""
    val = 0.0
    amp = 1.0
    total_amp = 0.0
    scale = base_scale
    for i in range(octaves):
        val += _noise2d(x, z, scale, layer + i * 1000) * amp
        total_amp += amp
        amp *= 0.5
        scale *= 0.5
    return val / total_amp

def get_biome(x, z):
    """Get the biome for chunk (x, z), deterministic and context-aware."""
    with biome_cache_lock:
        if (x, z) in biome_cache:
            return biome_cache[(x, z)]

    # Generate multiple noise channels to determine biome
    # Large scales (24-32) ensure biomes form big coherent regions, not tiny patches
    temperature = _multi_octave_noise(x, z, 3, 28.0, layer=0)   # 0=cold, 1=hot
    moisture    = _multi_octave_noise(x, z, 3, 26.0, layer=100)  # 0=dry, 1=wet
    elevation   = _multi_octave_noise(x, z, 3, 32.0, layer=200)  # 0=low, 1=high
    special     = _multi_octave_noise(x, z, 2, 16.0, layer=300)  # for sakura/swamp

    # Decision tree based on noise values
    if elevation < 0.28:
        biome = BIOME_OCEAN
    elif elevation < 0.35 and moisture > 0.45:
        biome = BIOME_SWAMP
    elif temperature < 0.35:
        if elevation > 0.6:
            biome = BIOME_SNOW
        else:
            biome = BIOME_EVERGREEN
    elif temperature > 0.65:
        biome = BIOME_DESERT
    elif special > 0.68 and moisture > 0.35:
        biome = BIOME_SAKURA
    elif moisture > 0.5:
        biome = BIOME_WOODLANDS
    else:
        biome = BIOME_GRASS

    with biome_cache_lock:
        biome_cache[(x, z)] = biome

    return biome

# ── natural object spawn tables per biome ────────────────────────────────────
# Each entry: (item_name, weight, size) where size is 1 or 2 (1x1 or 2x2)
# Higher weight = more likely to be placed
BIOME_OBJECTS = {
    BIOME_GRASS: [
        ("Birch Tree (Variant 1)", 8, 2),   ("Birch Tree (Variant 2)", 6, 2),
        ("Mossy Tree",             4, 2),    ("Willow Tree",            2, 2),
        ("Stone Vein",             3, 2),    ("Metal Vein",             1, 1),
        ("Spawner - Sticks",       5, 1),    ("Spawner - Bones",        2, 1),
        ("Spawner - Nuts",         3, 1),    ("Berry Bush",             4, 1),
        ("Flowers",                6, 1),    ("Cotton Plant",           2, 1),
        ("Creature Nest",          1, 1),    ("Beehive",                1, 1),
        ("Lavender Bush",          2, 1),    ("Tea Tree",               2, 1),
    ],
    BIOME_DESERT: [
        ("Palm Tree",              8, 2),    ("Cactus",                 6, 1),
        ("Stone Vein (Desert)",    4, 2),    ("Gold Vein",              2, 1),
        ("Spawner - Sticks",       3, 1),    ("Spawner - Bones",        3, 1),
        ("Spawner - Coconuts",     4, 1),    ("Spawner - Fossils",      2, 1),
        ("Hot Pepper Plant",       2, 1),    ("Creature Nest",          1, 1),
    ],
    BIOME_SNOW: [
        ("Frozen Tree",            8, 2),    ("Evergreen Tree",         6, 2),
        ("Stone Vein (Snowy)",     4, 2),    ("Silver Vein",            2, 1),
        ("Ice Shard Vein",         2, 1),    ("Spawner - Snowballs",    5, 1),
        ("Spawner - Sticks",       3, 1),    ("Spawner - Bones",        2, 1),
        ("Creature Nest",          1, 1),
    ],
    BIOME_EVERGREEN: [
        ("Evergreen Tree",         10, 2),   ("Large Mossy Tree",       3, 2),
        ("Mossy Tree",             4, 2),    ("Stone Vein",             3, 2),
        ("Metal Vein",             2, 1),    ("Titanium Vein",          1, 1),
        ("Spawner - Sticks",       5, 1),    ("Spawner - Nuts",         3, 1),
        ("Spawner - Bones",        2, 1),    ("Berry Bush",             3, 1),
        ("Giant Brown Mushroom",   2, 1),    ("Giant Red Mushroom",     1, 1),
        ("Beehive",                1, 1),    ("Creature Nest",          1, 1),
        ("Spiderhive",             1, 1),
    ],
    BIOME_OCEAN: [
        ("Stone Vein (Ocean)",     4, 2),
        ("Spawner - Red Shells",   3, 1),    ("Spawner - Blue Shells",  3, 1),
        ("Spawner - Green Shells", 3, 1),    ("Spawner - Purple Shells",2, 1),
        ("Spawner - White Shells", 2, 1),    ("Spawner - Black Shells", 1, 1),
        ("Spawner - Gold Shells",  1, 1),    ("Spawner - Bones",        2, 1),
    ],
    BIOME_SWAMP: [
        ("Willow Tree",            6, 2),    ("Mossy Tree",             5, 2),
        ("Rotting Stump",          4, 1),    ("Stone Vein",             3, 2),
        ("Spawner - Sticks",       4, 1),    ("Spawner - Bones",        3, 1),
        ("Spawner - Spirit Branch",2, 1),    ("Giant Purple Mushroom",  3, 1),
        ("Giant Brown Mushroom",   2, 1),    ("Lavender Bush",          2, 1),
        ("Spiderhive",             2, 1),    ("Creature Nest",          1, 1),
        ("Lava",                   1, 1),
    ],
    BIOME_WOODLANDS: [
        ("Birch Tree (Variant 1)", 7, 2),   ("Birch Tree (Variant 2)", 5, 2),
        ("Mossy Tree",             5, 2),    ("Large Mossy Tree",       2, 2),
        ("Evergreen Tree",         3, 2),    ("Stone Vein",             3, 2),
        ("Metal Vein",             2, 1),    ("Emerald Vein",           1, 1),
        ("Spawner - Sticks",       5, 1),    ("Spawner - Nuts",         4, 1),
        ("Berry Bush",             3, 1),    ("Goldberry Bush",         1, 1),
        ("Flowers",                3, 1),    ("Cotton Plant",           2, 1),
        ("Beehive",                2, 1),    ("Giant Brown Mushroom",   2, 1),
        ("Creature Nest",          1, 1),
    ],
    BIOME_SAKURA: [
        ("Sakura Tree",            10, 2),   ("Birch Tree (Variant 1)", 3, 2),
        ("Stone Vein (Sakura)",    3, 2),    ("Titanium Vein (Sakura)", 1, 1),
        ("Spawner - Sticks",       4, 1),    ("Spawner - Nuts",         3, 1),
        ("Berry Bush",             3, 1),    ("MoonBerry Bush",         2, 1),
        ("Salmonberry Bush",       2, 1),    ("Flowers",                5, 1),
        ("Lavender Bush",          3, 1),    ("Tea Tree",               3, 1),
        ("Beehive",                1, 1),    ("Creature Nest",          1, 1),
    ],
}

# ── Catalogue: every buildable item (for CATALOGUE_ENABLED mode) ─────────────
# (item_name, size) — size: 1=1x1, 2=2x2, 3=3x3, 5=5x5
CATALOGUE_ITEMS = [
    # 1x1 items
    ("3-day Land Claim",1), ("8-day Land Claim",1), ("Admin Land Claim",1),
    ("Amber Vein",1), ("Amethyst Vein",1), ("Anvil",1), ("Armor Display",1),
    ("Basket",1), ("Battle Start Pad",1), ("Beehive",1), ("Berry Bush",1),
    ("Big Stone Head",1), ("Blue String Lights",1), ("Blue Torch",1),
    ("Bonsai Tree",1), ("Bouncy Floor",1), ("Cactus",1), ("Campfire",1),
    ("Cauldron",1), ("Cave Basket",1), ("Cave Chest",1), ("Chair",1),
    ("Chest",1), ("Clouds",1), ("Cobblestone Path",1), ("Coin Hoard",1),
    ("Cotton Plant",1), ("Crafting Table",1), ("Crate",1), ("Creature Nest",1),
    ("Crucible",1), ("Custom Statue",1), ("Dark Shard Vein",1),
    ("Dirt Path",1), ("Double Crate",1),
    ("Dug-up Brown Mushroom",1), ("Dug-up Pumpkin",1),
    ("Dug-up Purple Mushroom",1), ("Dug-up Red Mushroom",1), ("Dug-up Wheat",1),
    ("Egg Fuser",1), ("Emerald Vein",1), ("Fancy Torch",1), ("Flagpole",1),
    ("Flowers",1), ("Giant Brown Mushroom",1), ("Giant Purple Mushroom",1),
    ("Giant Red Mushroom",1), ("Gold Chest",1), ("Gold Vein",1),
    ("Goldberry Bush",1), ("Grandfather Clock",1), ("Gravestone",1),
    ("Green Blob",1), ("Holiday Lights",1), ("Hot Pepper Plant",1),
    ("Huge Coin Hoard",1), ("Ice Shard Vein",1), ("Karaoke",1),
    ("Lamp Post",1), ("Large Black Stalagmite",1), ("Large Ocean Stalagmite",1),
    ("Large Snow Stalagmite",1), ("Large Weapon Display",1), ("Lava",1),
    ("Lavender Bush",1), ("Loom",1), ("Loot Basket",1), ("Loot Chest",1),
    ("Low Stone Wall",1), ("Magic Bean",1), ("Magmite Vein",1),
    ("Merchant Sign",1), ("Metal Chair",1), ("Metal Lamp Post",1),
    ("Metal Vein",1), ("MoonBerry Bush",1), ("Music Box",1), ("Navpost",1),
    ("Old Land Claim",1), ("Old Torch",1), ("Oven",1), ("Paint Mixer",1),
    ("Paint Shaker",1), ("Painting",1), ("Painting Easel",1),
    ("Palisade Wall",1), ("Poison Spikes",1),
    ("Red String Lights",1), ("Red Torch",1), ("Rotting Stump",1),
    ("Ruby Vein",1), ("Salmonberry Bush",1), ("Sapphire Vein",1), ("Sign",1),
    ("Silver Vein",1), ("Sky Chest",1), ("Small Black Stalagmite",1),
    ("Small Ocean Stalagmite",1), ("Small Snow Stalagmite",1),
    ("Small Table",1), ("Snowman",1), ("Sofa Chair",1),
    ("Spawner - Ancient Bones",1), ("Spawner - Black Shells",1),
    ("Spawner - Blue Shells",1), ("Spawner - Bones",1),
    ("Spawner - Coconuts",1), ("Spawner - Fossils",1),
    ("Spawner - Gold Shells",1), ("Spawner - Green Shells",1),
    ("Spawner - Nuts",1), ("Spawner - Purple Shells",1),
    ("Spawner - Red Shells",1), ("Spawner - Snowballs",1),
    ("Spawner - Spirit Branch",1), ("Spawner - Sticks",1),
    ("Spawner - White Shells",1), ("Spiderhive",1), ("Stamp Maker",1),
    ("Stone Bricks",1), ("Stone Lantern",1), ("String Lights",1),
    ("Stump Chair",1), ("Tall Stone Wall",1), ("Tea Tree",1),
    ("Teleporter",1), ("Titanium Chest",1), ("Titanium Vein",1),
    ("Titanium Vein (Sakura)",1), ("Torch",1), ("Trophy",1),
    ("Uranium Vein",1), ("Vending Machine",1), ("Weapon Display",1),
    ("Wisdom Chest",1), ("Wood Rails",1),
    # 2x2 items
    ("Birch Tree (Variant 1)",2), ("Birch Tree (Variant 2)",2),
    ("Blue Blob",2), ("Boss Chest",2), ("Circular Rug",2),
    ("Dug-up Giant Pumpkin",2), ("Evergreen Tree",2), ("Frozen Tree",2),
    ("Large Mossy Tree",2), ("Mossy Tree",2), ("Palm Tree",2),
    ("Red Blob",2), ("Sakura Tree",2), ("Spirit Tree",2),
    ("Spooky Well",2), ("Stone Vein",2), ("Stone Vein (Desert)",2),
    ("Stone Vein (Ocean)",2), ("Stone Vein (Sakura)",2),
    ("Stone Vein (Snowy)",2), ("Stone Vein (White)",2), ("Willow Tree",2),
    # 2x1
    ("Wooden Gate",2),
    # 3x3 items
    ("Ancient Pillars",3), ("Ancient Pillars (dark)",3),
    ("Ancient Pillars (storm)",3), ("Desert Cave Entrance",3),
    ("Evergreen Cave Entrance",3), ("Grass Cave Entrance",3),
    ("Large Amber Vein",3), ("Large Amethyst Vein",3),
    ("Large Dark Shard Vein",3), ("Large Emerald Vein",3),
    ("Large Gold Vein",3), ("Large Ice Shard Vein",3),
    ("Large Metal Vein",3), ("Large Ruby Vein",3),
    ("Large Sapphire Vein",3), ("Large Silver Vein",3),
    ("Large Stone Vein",3), ("Large Titanium Vein",3),
    ("Large Uranium Vein",3), ("Ocean Cave Entrance",3),
    ("Personal Mine",3), ("Snow Cave Entrance",3), ("Swamp Cave Entrance",3),
    # 5x5 items
    ("Boss Spawner - Shindeon",5), ("Boss Spawner - Yandeon",5),
    ("Frozen Pond",5), ("Pond",5), ("Sakura Pond",5), ("Tar Pit",5),
    # x_plus_1 (treat as 2x1)
    ("Bed",2), ("Big Table",2), ("Bookshelf",2), ("Fireplace",2),
    ("Park Bench",2), ("Pool Table",2), ("Pop-up Saw",2), ("Trading Table",2),
]

# Pre-generate catalogue chunk layouts (chunk_offset → list of build dicts)
CATALOGUE_ORIGIN_X = 50  # catalogue chunks start at x=50, z=0
catalogue_cache = {}

def _generate_catalogue_chunks():
    """Pre-layout all catalogue items across chunks at (CATALOGUE_ORIGIN_X+n, 0)."""
    if not CATALOGUE_ENABLED:
        return

    # Sort items by size (large first) so they get placed before space runs out
    # Within each size group, keep original order
    items_1 = [(n, s) for n, s in CATALOGUE_ITEMS if s == 1]
    items_2 = [(n, s) for n, s in CATALOGUE_ITEMS if s == 2]
    items_3 = [(n, s) for n, s in CATALOGUE_ITEMS if s == 3]
    items_5 = [(n, s) for n, s in CATALOGUE_ITEMS if s >= 5]

    # Place large items first in their own chunks, then smaller ones
    all_sorted = items_5 + items_3 + items_2 + items_1
    placed_count = 0

    # Simple grid placement: each chunk is 10x10 tiles
    chunk_offset = 0
    builds = []
    occupied = set()
    cursor_x, cursor_z = 0, 0
    row_height = 1  # track tallest item in current row

    for item_name, size in all_sorted:
        # Try to fit item at cursor position
        fits = False
        while not fits:
            if cursor_x + size > 10:
                # Next row
                cursor_x = 0
                cursor_z += row_height
                row_height = size
            if cursor_z + size > 10:
                # Chunk full, start new chunk
                catalogue_cache[(CATALOGUE_ORIGIN_X + chunk_offset, 0)] = builds
                chunk_offset += 1
                builds = []
                occupied = set()
                cursor_x, cursor_z = 0, 0
                row_height = size

            needed = {(cursor_x + dx, cursor_z + dz) for dx in range(size) for dz in range(size)}
            if not needed & occupied and all(t < 10 for _, t in needed) and all(t < 10 for t, _ in needed):
                occupied |= needed
                cx = CATALOGUE_ORIGIN_X + chunk_offset
                item_bytes = pack_item(item_name)
                builds.append({
                    "item_hex": item_bytes.hex(),
                    "rotation": 0,
                    "tile_x": cursor_x,
                    "tile_z": cursor_z,
                    "pos": [cx, 0, cursor_x, cursor_z],
                    "owner": "",
                    "zone": "overworld"
                })
                placed_count += 1
                cursor_x += size
                if size > row_height:
                    row_height = size
                fits = True
            else:
                cursor_x += 1
                if cursor_x + size > 10:
                    cursor_x = 0
                    cursor_z += row_height
                    row_height = size
                if cursor_z + size > 10:
                    catalogue_cache[(CATALOGUE_ORIGIN_X + chunk_offset, 0)] = builds
                    chunk_offset += 1
                    builds = []
                    occupied = set()
                    cursor_x, cursor_z = 0, 0
                    row_height = size

    # Save last chunk
    if builds:
        catalogue_cache[(CATALOGUE_ORIGIN_X + chunk_offset, 0)] = builds
        chunk_offset += 1

    print(f"[CATALOGUE] Generated {chunk_offset} catalogue chunks with {placed_count} items (x={CATALOGUE_ORIGIN_X}..{CATALOGUE_ORIGIN_X + chunk_offset - 1}, z=0)")

# Cache of generated chunk objects: (x, z) → list of build dicts
worldgen_cache = {}
worldgen_cache_lock = threading.Lock()

def generate_chunk_objects(cx, cz, biome_id):
    """Generate natural objects for a chunk. Returns list of build dicts."""
    with worldgen_cache_lock:
        if (cx, cz) in worldgen_cache:
            return worldgen_cache[(cx, cz)]

    # Seeded RNG for this chunk
    rng = random.Random(WORLD_SEED ^ (cx * 73856093) ^ (cz * 19349669))

    objects = BIOME_OBJECTS.get(biome_id, BIOME_OBJECTS[BIOME_GRASS])

    # Determine how many objects to place (3-8 per chunk)
    num_objects = rng.randint(3, 8)

    # Build weighted selection list
    pool = []
    for item_name, weight, size in objects:
        pool.extend([(item_name, size)] * weight)

    # Track occupied tiles to prevent overlap
    occupied = set()
    builds = []

    for _ in range(num_objects):
        if not pool:
            break
        item_name, size = rng.choice(pool)
        rotation = rng.randint(0, 3)

        # Try to find a free spot (up to 10 attempts)
        placed = False
        for _attempt in range(10):
            if size == 2:
                tx = rng.randint(0, 8)  # 2x2 needs room
                tz = rng.randint(0, 8)
                tiles = {(tx, tz), (tx+1, tz), (tx, tz+1), (tx+1, tz+1)}
            else:
                tx = rng.randint(0, 9)
                tz = rng.randint(0, 9)
                tiles = {(tx, tz)}

            if not tiles & occupied:
                occupied |= tiles
                item_bytes = pack_item(item_name)
                builds.append({
                    "item_hex": item_bytes.hex(),
                    "rotation": rotation,
                    "tile_x": tx,
                    "tile_z": tz,
                    "pos": [cx, cz, tx, tz],
                    "owner": "",
                    "zone": "overworld"
                })
                placed = True
                break

    with worldgen_cache_lock:
        worldgen_cache[(cx, cz)] = builds

    return builds

# ── day/night cycle ──────────────────────────────────────────────────────────
# Time is a float in seconds; client reads Short(ms) then divides by 1000.
# We track a server-wide time and sync it periodically.
server_time_start = time.time()
DAY_LENGTH_SECONDS = 600.0  # 10 real minutes = 1 full day cycle (tune as needed)

def get_daynight_ms():
    """Return the current day/night time as a short (milliseconds)."""
    elapsed = time.time() - server_time_start
    cycle_pos = (elapsed % DAY_LENGTH_SECONDS) / DAY_LENGTH_SECONDS
    return int(cycle_pos * 24000) % 32767  # 0-24000 range, fits in signed short

def daynight_sync_loop():
    """Periodically broadcast day/night time to all players."""
    while True:
        time.sleep(30)  # sync every 30 seconds
        with players_lock:
            if not players:
                continue
        ms = get_daynight_ms()
        pkt = bytes([0x17]) + struct.pack('<h', ms)
        broadcast(pkt, label="DAYNIGHT_SYNC")

threading.Thread(target=daynight_sync_loop, daemon=True).start()

# ── report logging ───────────────────────────────────────────────────────────
REPORTS_FILE = os.path.join(CHUNK_DIR, "_reports.json") if IS_PUBLIC else os.path.join(
    os.path.dirname(os.path.abspath(__file__)), "_reports.json")

def save_report(reporter, report_data):
    """Append a report entry to the reports JSON file."""
    reports = []
    if os.path.exists(REPORTS_FILE):
        try:
            with open(REPORTS_FILE, 'r', encoding='utf-8') as f:
                reports = json.load(f)
        except:
            pass
    reports.append({
        "time": datetime.datetime.now().isoformat(),
        "reporter": reporter,
        "data": report_data
    })
    with open(REPORTS_FILE, 'w', encoding='utf-8') as f:
        json.dump(reports, f, indent=2)
    print(f"  [REPORT] Saved report from {reporter} ({len(reports)} total)")

# ── packet helpers (identical to intercept.py) ───────────────────────────────
def craft_batch(qid, payload):
    return struct.pack('<HBBB I', 9 + len(payload), 1, qid, 3, len(payload)) + payload

def pack_string(s):
    enc = s.encode('utf-16-le')
    return struct.pack('<H', len(enc)) + enc

def unpack_string(data, offset):
    try:
        length = struct.unpack('<H', data[offset:offset+2])[0]
        val    = data[offset+2 : offset+2+length].decode('utf-16-le')
        return val, offset + 2 + length
    except:
        return "", offset

def skip_inventory_item(data, off):
    """Skip past an InventoryItem::UnpackFromWeb in binary data. Returns new offset."""
    # Short properties: Short(count) + [String(key) + Short(value)] × count
    count = struct.unpack_from('<H', data, off)[0]; off += 2
    for _ in range(count):
        _, off = unpack_string(data, off)  # key
        off += 2  # short value
    # String properties: Short(count) + [String(key) + String(value)] × count
    count = struct.unpack_from('<H', data, off)[0]; off += 2
    for _ in range(count):
        _, off = unpack_string(data, off)  # key
        _, off = unpack_string(data, off)  # value
    # Long properties: Short(count) + [String(key) + Long(value)] × count
    count = struct.unpack_from('<H', data, off)[0]; off += 2
    for _ in range(count):
        _, off = unpack_string(data, off)  # key
        off += 4  # long value (4 bytes)
    return off

def read_inventory_item(data, off):
    """Read an InventoryItem and return (item_bytes, new_offset)."""
    start = off
    off = skip_inventory_item(data, off)
    return data[start:off], off

def read_basket_contents(data, off):
    """Read BasketContents from binary data. Returns (slots_list, raw_bytes, new_offset)."""
    start = off
    slot_count = struct.unpack_from('<H', data, off)[0]; off += 2
    slots = []
    for _ in range(slot_count):
        slot_index = struct.unpack_from('<H', data, off)[0]; off += 2
        quantity = struct.unpack_from('<H', data, off)[0]; off += 2
        item_start = off
        off = skip_inventory_item(data, off)
        item_hex = data[item_start:off].hex()
        slots.append({"index": slot_index, "count": quantity, "item_hex": item_hex})
    return slots, data[start:off], off

def pack_basket_contents(slots):
    """Pack a slots list back into binary BasketContents format."""
    p = struct.pack('<H', len(slots))
    for slot in slots:
        p += struct.pack('<H', slot["index"])
        p += struct.pack('<H', slot["count"])
        p += bytes.fromhex(slot["item_hex"])
    return p

def pack_item(item_type):
    """Build a minimal InventoryItem with just the item_id string property."""
    p  = struct.pack('<H', 0)                          # 0 short properties
    p += struct.pack('<H', 1)                          # 1 string property
    p += pack_string("item_id") + pack_string(item_type)
    p += struct.pack('<H', 0)                          # 0 long properties
    return p

def give_item(conn, item_type, count=1):
    """Send a container packet (0x1B) with the given item to a player."""
    item_bytes = pack_item(item_type)
    basket_id = 999999  # virtual container ID
    resp = bytes([0x1b])
    resp += struct.pack('<I', basket_id)
    # BasketContents: 1 slot at index 0 with quantity=count
    resp += struct.pack('<H', 1)       # slot_count = 1
    resp += struct.pack('<H', 0)       # slot_index = 0
    resp += struct.pack('<H', count)   # quantity
    resp += item_bytes
    send_packet(conn, 2, resp, f"GIVE_ITEM_{item_type}")

def send_packet(conn, qid, payload, label="?"):
    try:
        batch = craft_batch(qid, payload)
        pid   = payload[0] if payload else 0
        hex_preview = binascii.hexlify(payload[:64]).decode()
        print(f"  [SRV→CLT] [{label}] qid={qid} type=0x{pid:02X} len={len(payload)}")
        print(f"    HEX: {hex_preview}{'...' if len(payload)>64 else ''}")

        log_data("SEND", batch, label)

        conn.sendall(batch)
    except Exception as e:
        print(f"  [!] send_packet error ({label}): {e}")

# ── session state ────────────────────────────────────────────────────────────
# players[username] = { 'conn', 'addr', 'initial_data': bytes|None }
players      = {}
players_lock = threading.Lock()

def get_host_conn():
    """Return the host player's connection, or None."""
    with host_lock:
        hname = host_player
    if hname is None:
        return None, None
    with players_lock:
        p = players.get(hname)
    if p is None:
        return None, None
    return p['conn'], hname

def broadcast(payload, exclude=None, label="BCAST"):
    """Send a fully-built payload to all players except the excluded one."""
    with players_lock:
        for uname, p in list(players.items()):
            if uname == exclude:
                continue
            send_packet(p['conn'], 2, payload, label)

def send_private(target_id, payload, label="PRIVATE"):
    """Send a payload to a specific player by their ID."""
    with players_lock:
        target_id = target_id.lower()
        if target_id in players:
            send_packet(players[target_id]['conn'], 2, payload, label)
        else:
            print(f"  [!] Private target '{target_id}' not found for {label}")

# ── packet name map ──────────────────────────────────────────────────────────
C2S = {
    0x01:"PING",         0x03:"INITIAL_PLAYER_DATA", 0x06:"GAME_CHAT",
    0x09:"GUARD_DIE",    0x0a:"REQ_ZONE_DATA",       0x0c:"REQ_CHUNK",
    0x11:"PLAYER_POS",   0x14:"CHANGE_ZONE",          0x15:"START_TELEPORT",
    0x16:"END_TELEPORT", 0x18:"CHANGE_EQUIP",         0x19:"UPDATE_PARENTS",
    0x1a:"REQ_CONTAINER",0x1e:"CLOSE_BASKET",         0x20:"BUILD_FURNITURE",
    0x21:"REMOVE_OBJECT",0x22:"REPLACE_BUILDABLE",    0x23:"CHANGE_LANDCLAIM",
    0x26:"LOGIN",        0x27:"CLAIM_OBJECT",         0x28:"RELEASE_OBJECT",
    0x29:"REQ_UNIQUE_IDS",0x2b:"YOU_MAY_JOIN",       0x2d:"ASK_JOIN",
    0x2e:"REQ_TELE_PAGE",0x30:"SEND_TELE_SCREENSHOT", 0x31:"REQ_TELE_SCREENSHOT",
    0x33:"FINISH_TELE_EDIT",0x34:"NEW_TELE_SEARCH",   0x35:"CHALLENGE_MINIGAME",
    0x36:"MINIGAME_RESP",0x37:"BEGIN_MINIGAME",       0x38:"EXIT_MINIGAME",
    0x39:"POOL_CUE_POS", 0x3a:"POOL_SHOOT",           0x3b:"POOL_SYNC_READY",
    0x3c:"POOL_PLACE_BALL",0x3d:"POOL_PLAY_AGAIN",    0x3e:"SIT_CHAIR",
    0x3f:"TRY_CLAIM_MOBS",0x40:"DELOAD_MOB",          0x41:"MOB_POSITIONS",
    0x46:"ATTACK_ANIM",  0x47:"HIT_MOB",              0x48:"MOB_DIE",
    0x4a:"CREATURE_STATS",0x4b:"INCREASE_HP",         0x4e:"COMPANION_EQUIP",
    0x4f:"RENAME_COMPANION",0x50:"DESTROY_COMPANION",  0x51:"APPLY_PERK",
    0x52:"LAUNCH_PROJECTILE",0x53:"QUICK_TAG",         0x54:"ALL_PRE_PERKS",
    0x55:"CREATE_PERK_DROP",0x56:"RESPAWN",           0x57:"RETURN_TO_BREEDER",
    0x58:"SYNC_TARGET_IDS",0x59:"CREATED_LOCAL_MOB",  0x5a:"BANDIT_FLAG",
}

# ── server→client packet IDs ─────────────────────────────────────────────────
SRV_LOGIN_SUCCESS   = 0x0b
SRV_PLAYER_NEARBY   = 0x13  # S2C case 0x13 flag=1: NewPlayerNearby (adds to onlinePlayers + spawns)
SRV_PLAYER_GONE     = 0x13  # S2C case 0x13 flag=0: NearbyPlayerWentAway (removes + destroys mobs)
SRV_PONG            = 0x01
SRV_CHAT            = 0x06
SRV_LOGIN_OUT_NOTIF = 0x1e
SRV_TELEPORT_START  = 0x0c
SRV_TELEPORT_END    = 0x00
SRV_JOIN_CONFIRMED  = 0x02
SRV_EQUIP_CHANGE    = 0x23
SRV_POSITION        = 0x11
SRV_PERK            = 0x24
SRV_CHUNK           = 0x0d

# ── zone data packet builder ─────────────────────────────────────────────────

def make_zone_data_blob(zone_name="overworld", zone_type=0):
    p = struct.pack('<HHH', 0, 0, 0)  # InventoryItem properties dummy
    p += bytes([zone_type])
    p += struct.pack('<HHHH', 0, 0, 0, 0)
    p += pack_string(zone_name)
    p += struct.pack('<H', 0)  # LandClaim Timer Count 0
    return p

def make_success_zone_packet(zone_name="overworld", zone_type=0):
    p = bytes([0x0B])
    p += bytes([1])                              # flag = 1 (zone data follows)
    p += bytes([0])                              # sub_flag = 0
    p += pack_string(zone_name)                  # zone name string
    p += make_zone_data_blob(zone_name, zone_type)
    p += bytes([zone_type])                       # trailing zone type byte
    return p


def make_dummy_chunk(x, z, zone_name="overworld", dim=0, sub_zone="", builds=None):
    """
    S2C 0x0D — builds are embedded directly into chunk tile data.

    ChunkData::UnpackFromWeb tile format:
      Byte(tile_count) + [
        Byte(tx) + Byte(tz) + Short(elem_count) + [
          Byte(rotation) + InventoryItem
        ] × elem_count
      ] × tile_count

    BUILD shorts[2]=tileX and shorts[3]=tileZ map to tx/tz bytes in tiles.
    Confirmed from ConstructionControl$PlayerBuildAt → ChunkData$AddElement(param_6, param_7).
    """
    p  = bytes([0x0D])
    p += pack_string(zone_name)              # outer zone_name
    p += struct.pack('<hh', x, z)            # header chunk coords
    p += bytes([0])                          # flag = 0 (new chunk data)
    p += pack_string("")                     # checkpoint = empty
    # -- ChunkData::UnpackFromWeb body --
    p += struct.pack('<hh', x, z)            # chunk coords (inside ChunkData)
    p += pack_string(zone_name)              # zone_name (inner)
    # ChunkData shorts: biome, floor_model_id, floor_texture_index, floor_rotation
    biome_id = get_biome(x, z)
    p += struct.pack('<hhhh', biome_id, 0, 0, 0)
    p += pack_string(sub_zone)               # string at ChunkData+0x24
    p += pack_string("")                     # string at ChunkData+0x28

    # -- tile data: embed saved builds --
    if builds:
        # Group builds by (tile_x, tile_z)
        tiles = {}
        for b in builds:
            tx = b["tile_x"] & 0xFF
            tz = b["tile_z"] & 0xFF
            key = (tx, tz)
            if key not in tiles:
                tiles[key] = []
            tiles[key].append(b)

        p += bytes([len(tiles)])  # tile_count
        for (tx, tz), elems in tiles.items():
            p += bytes([tx, tz])
            p += struct.pack('<H', len(elems))
            for elem in elems:
                p += bytes([elem["rotation"]])
                p += bytes.fromhex(elem["item_hex"])
    else:
        p += bytes([0])  # tile_count = 0 (empty)

    p += struct.pack('<H', 0)                # land_claim_count = 0
    p += bytes([0])                          # bandit_camp_count = 0
    return p


def make_join_confirmed(host_name, username, player_id_short=0, is_host=0):
    p  = bytes([0x02])
    p += pack_string(host_name)
    p += bytes([is_host])
    p += bytes([0])
    p += pack_string(username)
    p += struct.pack('<h', player_id_short)
    p += struct.pack('<h', 0)
    return p


def make_login_response(world_name, token, zone_type=0):
    p  = bytes([0x26])
    p += struct.pack('<h', 0)           # zone_trail_count = 0
    p += pack_string(world_name)
    p += pack_string(token)
    p += bytes([zone_type])
    return p

# ── client handler ───────────────────────────────────────────────────────────
def handle_client(conn, addr):
    player_id  = None
    conn_state = {'login_done': False}
    buf        = b""
    try:
        print(f"[+] Connection from {addr}")
        while True:
            chunk = conn.recv(65536)
            if not chunk:
                break

            touch_activity()
            log_data("RECV", chunk, str(addr))

            buf += chunk

            # Consume complete framed packets from buffer
            while len(buf) >= 2:
                # Handshake: single 0x66 byte (no length prefix)
                if buf[0] == 0x66:
                    print(f"\n[{addr}] [HANDSHAKE]")
                    send_packet(conn, 0, b'\x09\x01', "HANDSHAKE")
                    buf = buf[1:]
                    continue

                if len(buf) < 2:
                    break
                total_len = struct.unpack('<H', buf[0:2])[0]
                if len(buf) < total_len:
                    break

                packet = buf[:total_len]
                buf    = buf[total_len:]

                if total_len < 10:
                    continue

                pid  = packet[9]
                name = C2S.get(pid, f"ID_0x{pid:02X}")
                print(f"\n[{addr}] [{name}]")

                dispatch(conn, addr, packet, pid, player_id, name, conn_state)

                if player_id is None:
                    with players_lock:
                        for uid, p in players.items():
                            if p['conn'] is conn:
                                player_id = uid
                                break

    except Exception as e:
        import traceback
        print(f"[!] handle_client error ({addr}): {e}")
        traceback.print_exc()
    finally:
        if player_id:
            with players_lock:
                players.pop(player_id, None)
            broadcast(bytes([SRV_PLAYER_GONE, 0]) + pack_string(player_id) + bytes([0]),
                      label="PLAYER_DISCONNECT")
            # S2C 0x07: join/leave notification — String(unused) + String(name) + Byte(0=left)
            leave_pkt = bytes([0x07]) + pack_string("") + pack_string(player_id) + bytes([0])
            broadcast(leave_pkt, label="PLAYER_LEFT_NOTIF")
            print(f"[-] {player_id} disconnected")
        conn.close()


def dispatch(conn, addr, data, pid, username, name, conn_state):
    """Route an incoming client packet."""
    total_len = len(data)
    pname = C2S.get(pid, f"ID_0x{pid:02X}")
    print(f"[{addr}] [{pname}]")

    if pid == 0x01: # PING
        send_packet(conn, 2, bytes([SRV_PONG]), "PONG")

    elif pid == 0x26: # LOGIN
        if conn_state.get('login_done'):
            print(f"  [LOGIN] IGNORED (already logged in as {username!r})")
            return
        conn_state['login_done'] = True

        raw_world, off = unpack_string(data, 10)
        raw_token, _   = unpack_string(data, off)

        world_name = raw_world.replace('\x00', '').strip()
        token      = raw_token.replace('\x00', '').strip()

        player_id = token if token else f"player_{addr[1]}"
        world_name = ROOM_TOKEN

        print(f"  [LOGIN] raw_world={raw_world!r} raw_token={raw_token!r}")
        print(f"  [LOGIN] player_id={player_id!r} world={world_name!r}")

        # ── Track the host (pre-assigned, or first login) ───────────────
        is_host_player = False
        if not IS_PUBLIC:
            global host_player
            with host_lock:
                if host_player is None:
                    # Fallback for manual starts: first one in is host
                    host_player = player_id
                    is_host_player = True
                    print(f"  [HOST] No pre-assigned host, using first login: {player_id!r}")
                else:
                    # Explicit check against assigned name
                    if player_id.lower() == host_player.lower():
                        is_host_player = True
                        print(f"  [HOST] Designated host {player_id!r} has logged in")

        with players_lock:
            players[player_id] = {'conn': conn, 'addr': addr, 'initial_data': None,
                                  'world_name': world_name, 'is_host': is_host_player}

        send_packet(conn, 2, make_login_response(world_name, player_id),
                    "LOGIN_RESPONSE")

        uid_count = 16
        uid_block = struct.pack('<H', uid_count)
        for i in range(uid_count):
            uid_block += struct.pack('<q', i + 1)
        send_packet(conn, 2, bytes([0x29]) + uid_block, "LOGIN_UNIQUE_IDS")

        # Tell the host client that it IS the host (is_host=1) so that
        # ShouldSaveLocally returns true and the client loads from disk.
        send_packet(conn, 2, make_join_confirmed(world_name, player_id,
                    is_host=(1 if is_host_player else 0)),
                    "JOIN_CONFIRMED")

        send_packet(conn, 2, make_success_zone_packet("overworld", 0), "LOGIN_SUCCESS_ZONE")

        # Send initial day/night sync
        ms = get_daynight_ms()
        send_packet(conn, 2, bytes([0x17]) + struct.pack('<h', ms), "DAYNIGHT_INIT")

        # S2C 0x07: join notification — String(unused) + String(name) + Byte(1=joined)
        join_pkt = bytes([0x07]) + pack_string("") + pack_string(player_id) + bytes([1])
        broadcast(join_pkt, exclude=player_id, label="PLAYER_JOIN_NOTIF")

    elif pid == 0x2a: # SYNC_COMPLETE
        send_packet(conn, 2, data[9:total_len], "ACK_SYNC_COMPLETE")
        if username:
            broadcast(data[9:total_len], exclude=username, label="RELAY_SYNC_COMPLETE")

    elif pid == 0x03: # INITIAL_PLAYER_DATA
        if username:
            off = 10
            pos_bytes = data[off:off+8]
            off += 8
            zone_name, off = unpack_string(data, off)
            appearance = data[off] if off < len(data) else 0
            off += 1
            rest_of_body = data[off:]

            opd  = pos_bytes
            opd += pos_bytes
            opd += struct.pack('<hhhh', 0, 0, 0, 100)
            opd += bytes([appearance])
            opd += pack_string(zone_name)
            opd += pack_string("")
            opd += rest_of_body

            with players_lock:
                if username in players:
                    players[username]['initial_data'] = opd

            def delayed_sync():
                time.sleep(2.0)
                # Tell everyone else about the new player
                spawn_pkt = bytes([SRV_PLAYER_NEARBY, 1]) + pack_string(username) + pack_string(username) + opd
                broadcast(spawn_pkt, exclude=username, label="SPAWN_PLAYER")

                # Reciprocal sync: Tell current player about everyone else
                with players_lock:
                    for uname, p in players.items():
                        if uname != username and p['initial_data']:
                            pkt = bytes([SRV_PLAYER_NEARBY, 1]) + pack_string(uname) + pack_string(uname) + p['initial_data']
                            send_packet(conn, 2, pkt, "EXISTING_PLAYER_SYNC")

                # FRIEND MODE: Specifically tell host to create a character for guest
                if not IS_PUBLIC:
                    host_conn, hname = get_host_conn()
                    if host_conn and username != hname:
                        host_spawn_pkt = bytes([SRV_PLAYER_NEARBY, 1]) + pack_string(username) + pack_string(username) + opd
                        send_packet(host_conn, 2, host_spawn_pkt, f"SYNC_GUEST_{username}_TO_HOST")
                        print(f"  [RELAY] Notified host {hname!r} to spawn guest {username!r} (after delay)")

            threading.Thread(target=delayed_sync, daemon=True).start()

    elif pid == 0x11: # POSITION
        if username:
            broadcast(bytes([SRV_POSITION]) + pack_string(username) + data[10:],
                      exclude=username, label="RELAY_POS")

    elif pid == 0x41: # MOB_POSITIONS
        if username:
            broadcast(bytes([pid]) + pack_string(username) + data[10:],
                      exclude=username, label="RELAY_MOB_POS")

    elif pid == 0x14: # ZONE_CHANGE
        zone_name, off = unpack_string(data, 10)
        print(f"  zone={zone_name!r}")

    elif pid == 0x0a: # REQ_ZONE_DATA
        zone_name, off = unpack_string(data, 10)
        zone_type = data[off] if len(data) > off else 0
        print(f"  zone_req={zone_name!r} type={zone_type}")

        if IS_PUBLIC:
            send_packet(conn, 2, make_success_zone_packet(zone_name, zone_type), "ZONE_DATA_RESP")
        else:
            # ── FRIEND mode: relay zone req to host ───────────────────────
            host_conn, hname = get_host_conn()
            if host_conn is None:
                send_packet(conn, 2, make_success_zone_packet(zone_name, zone_type), "ZONE_DATA_RESP_FALLBACK")
            elif username == hname:
                # Host requesting its own zone data
                send_packet(conn, 2, make_success_zone_packet(zone_name, zone_type), "ZONE_DATA_RESP_HOST_SELF")
            else:
                # Send S2C 0x0A to host: Byte(0x0A) + String(requester) + String(zone) + Byte(type)
                # The host responds with 0x0B (which we already catch and relay if it matches the pattern)
                # Note: For simplicity, we could also just send the fallback success packet,
                # as biome/zone metadata is usually static, but relaying is more correct.
                with pending_lock:
                    zone_key = f"zone_{zone_name}"
                    if zone_key not in pending_chunk_requests: # Reuse chunk pending map for simplicity or add specific
                        pending_chunk_requests[zone_key] = []
                    pending_chunk_requests[zone_key].append(username)
                
                relay = bytes([0x0A])
                relay += pack_string(username)
                relay += pack_string(zone_name)
                relay += bytes([zone_type])
                # If there's more position data in the original packet, relay it
                if len(data) > off + 1:
                    relay += data[off+1:]
                send_packet(host_conn, 2, relay, "RELAY_ZONE_REQ_TO_HOST")

    elif pid == 0x0c: # REQ_CHUNK
        zone_name, off = unpack_string(data, 10)
        if len(data) >= off + 4:
            x, z = struct.unpack_from('<hh', data, off)
            off += 4
            dimension_type = data[off] if len(data) > off else 0
            off += 1
            sub_zone, _ = unpack_string(data, off)
            print(f"  chunk_req x={x} z={z} zone={zone_name!r}")

            if IS_PUBLIC:
                # ── PUBLIC mode: serve from local chunk folder ────────────
                cd = load_chunk_data(x, z)
                if not os.path.exists(chunk_path(x, z)):
                    save_chunk_data(x, z, cd)
                player_builds = cd.get("builds", [])

                # Generate natural world objects and merge with player builds
                biome_id = get_biome(x, z)
                natural = generate_chunk_objects(x, z, biome_id)

                # Catalogue chunks (if enabled)
                cat_builds = catalogue_cache.get((x, z), []) if CATALOGUE_ENABLED else []

                all_builds = natural + cat_builds + player_builds

                send_packet(conn, 2, make_dummy_chunk(x, z, zone_name, dimension_type, sub_zone, all_builds), "CHUNK_RESP")
                if all_builds:
                    cat_str = f" catalogue={len(cat_builds)}" if cat_builds else ""
                    print(f"  [CHUNK] Served chunk x={x} z={z} biome={biome_id} natural={len(natural)} player={len(player_builds)}{cat_str}")
            else:
                # ── FRIEND mode: relay chunk request to the HOST ─────────
                # The host's OnReceive case 0x0C expects:
                #   String(requester) + String(zone_name) + Short(x) + Short(z)
                # It then loads the real chunk from disk, packs it via
                # ChunkData$$PackForWeb, and sends back a 0x0D with:
                #   String(requester) + packed_chunk_data + bandit_data
                host_conn, hname = get_host_conn()
                if host_conn is None:
                    print(f"  [!] No host connected, sending empty chunk")
                    send_packet(conn, 2, make_dummy_chunk(x, z, zone_name, dimension_type, sub_zone), "CHUNK_RESP_FALLBACK")
                elif username == hname:
                    # Host is requesting its own chunk — it reads locally
                    # via ShouldSaveLocally, so this shouldn't normally happen.
                    # If it does, send an empty chunk.
                    print(f"  [HOST] Host requesting own chunk, sending empty")
                    send_packet(conn, 2, make_dummy_chunk(x, z, zone_name, dimension_type, sub_zone), "CHUNK_RESP_HOST_SELF")
                else:
                    # Register this guest as waiting for the chunk response
                    chunk_key = f"{zone_name}_{x}_{z}"
                    with pending_lock:
                        if chunk_key not in pending_chunk_requests:
                            pending_chunk_requests[chunk_key] = []
                        pending_chunk_requests[chunk_key].append(username)
                    print(f"  [RELAY] Forwarding chunk req to host for {chunk_key}")

                    # Send S2C 0x0C to host:
                    # Byte(0x0C) + String(requester) + String(zone) + Short(x) + Short(z)
                    relay = bytes([0x0C])
                    relay += pack_string(username)      # requester name
                    relay += pack_string(zone_name)
                    relay += struct.pack('<hh', x, z)
                    send_packet(host_conn, 2, relay, "RELAY_CHUNK_REQ_TO_HOST")

    elif pid == 0x06: # CHAT
        msg, _ = unpack_string(data, 10)
        if username:
            # ── Chat commands ──
            msg_lower = msg.strip().lower()
            if msg_lower.startswith("report "):
                reason = msg[7:].strip()
                save_report(username, {"type": "chat_report", "reason": reason})
                confirm = (bytes([SRV_CHAT])
                          + pack_string("Server")
                          + pack_string("Server")
                          + pack_string("Report submitted. Thank you.")
                          + bytes([0]))
                send_packet(conn, 2, confirm, "REPORT_CONFIRM")
                return

            chat_payload = (bytes([SRV_CHAT])
                          + pack_string(username)
                          + pack_string(username)
                          + pack_string(msg)
                          + bytes([0]))
            broadcast(chat_payload, label="RELAY_CHAT")

    elif pid == 0x15: # TELE_START
        tele_name, _ = unpack_string(data, 10)
        if username:
            broadcast(bytes([SRV_TELEPORT_START]) + pack_string(username) + pack_string(tele_name),
                      exclude=username, label="BCAST_TELE_START")

    elif pid == 0x20: # BUILD_FURNITURE
        # C2S: Byte(0x20) + String(validator) + Item + Byte(rot) + String(zone) + Short×4 + String(extra)
        # S2C: Byte(0x20) + Item + Byte(rot) + String(zone) + Short×4 + String(owner) + String(extra)
        # The string is the ZONE NAME (e.g. "overworld"), NOT a chunk key.
        # The 4 shorts are: [chunkX, chunkZ, tileX, tileZ]
        #   shorts[0:2] = chunk coordinates (used in GetChunkString to build key)
        #   shorts[2:4] = tile position within chunk (0-9, used in ChunkData.AddElement)
        # DO NOT echo back to builder — client already placed it locally
        if username:
            off = 10
            _, off = unpack_string(data, off)              # skip validator string
            item_bytes, off = read_inventory_item(data, off)
            rotation = data[off]; off += 1
            zone_str, off = unpack_string(data, off)       # zone name, NOT chunk key
            shorts_bytes = data[off:off+8]; off += 8
            extra_bytes = data[off:]

            s2c = (bytes([0x20]) + item_bytes + bytes([rotation])
                   + pack_string(zone_str) + shorts_bytes
                   + pack_string(username) + extra_bytes)
            broadcast(s2c, exclude=username, label="RELAY_BUILD")

            # Persist to chunk file (PUBLIC only — host saves locally in FRIEND mode)
            if IS_PUBLIC:
                pos = list(struct.unpack_from('<hhhh', shorts_bytes))
                cx, cz = pos[0], pos[1]   # chunk coords from shorts[0:2]
                tx, tz = pos[2], pos[3]   # tile coords from shorts[2:4]
                cd = load_chunk_data(cx, cz)
                cd["builds"].append({
                    "item_hex": item_bytes.hex(),
                    "rotation": rotation,
                    "tile_x": tx,
                    "tile_z": tz,
                    "pos": pos,
                    "owner": username,
                    "zone": zone_str
                })
                save_chunk_data(cx, cz, cd)
                print(f"  [BUILD] Saved to chunk x{cx} z{cz} tile({tx},{tz}), total builds: {len(cd['builds'])}")

    elif pid == 0x21: # REMOVE_OBJECT
        # C2S: Byte(0x21) + String(validator) + String(zone) + Short×4 + Byte(rot) + Item + String(extra)
        # S2C: Byte(0x21) + String(zone) + Short×4 + Byte(rot) + Item + String(owner)
        # Shorts: [chunkX, chunkZ, tileX, tileZ] — same as BUILD
        # DO NOT echo back to builder
        if username:
            off = 10
            _, off = unpack_string(data, off)              # skip validator
            zone_str, off = unpack_string(data, off)       # zone name
            shorts_bytes = data[off:off+8]; off += 8
            rotation = data[off]; off += 1
            item_bytes, off = read_inventory_item(data, off)
            extra_bytes = data[off:]

            s2c = (bytes([0x21]) + pack_string(zone_str) + shorts_bytes
                   + bytes([rotation]) + item_bytes + pack_string(username))
            broadcast(s2c, exclude=username, label="RELAY_REMOVE")

            # Remove from chunk file (PUBLIC only)
            if IS_PUBLIC:
                pos = list(struct.unpack_from('<hhhh', shorts_bytes))
                cx, cz = pos[0], pos[1]
                cd = load_chunk_data(cx, cz)
                cd["builds"] = [b for b in cd["builds"] if b.get("pos") != pos]
                save_chunk_data(cx, cz, cd)
                print(f"  [REMOVE] Updated chunk x{cx} z{cz}, remaining builds: {len(cd['builds'])}")

    elif pid == 0x22: # REPLACE_BUILDABLE
        # C2S: Byte(0x22) + String(validator) + Item(old) + Item(new) + Byte(rot) + String(zone) + Short×4 + String(extra)
        # S2C: Byte(0x22) + Item(old) + Item(new) + Byte(rot) + String(zone) + Short×4 + String(owner)
        # Shorts: [chunkX, chunkZ, tileX, tileZ] — same as BUILD
        # DO NOT echo back to builder
        if username:
            off = 10
            _, off = unpack_string(data, off)              # skip validator
            old_bytes, off = read_inventory_item(data, off)
            new_bytes, off = read_inventory_item(data, off)
            rotation = data[off]; off += 1
            zone_str, off = unpack_string(data, off)       # zone name
            shorts_bytes = data[off:off+8]; off += 8
            extra_bytes = data[off:]

            s2c = (bytes([0x22]) + old_bytes + new_bytes + bytes([rotation])
                   + pack_string(zone_str) + shorts_bytes + pack_string(username))
            broadcast(s2c, exclude=username, label="RELAY_REPLACE")

            # Update chunk file (PUBLIC only)
            if IS_PUBLIC:
                pos = list(struct.unpack_from('<hhhh', shorts_bytes))
                cx, cz = pos[0], pos[1]
                tx, tz = pos[2], pos[3]
                cd = load_chunk_data(cx, cz)
                cd["builds"] = [b for b in cd["builds"] if b.get("pos") != pos]
                cd["builds"].append({
                    "item_hex": new_bytes.hex(),
                    "rotation": rotation,
                    "tile_x": tx,
                    "tile_z": tz,
                    "pos": pos,
                    "owner": username,
                    "zone": zone_str
                })
                save_chunk_data(cx, cz, cd)
                print(f"  [REPLACE] Updated chunk x{cx} z{cz} tile({tx},{tz})")

    elif pid == 0x1a: # REQ_CONTAINER
        # C2S: Byte(0x1a) + String(validator) + Long(basket_id) + Byte(type) + String(chunk) + Short×4
        # S2C 0x1b: Byte(0x1b) + Long(basket_id) + BasketContents
        if username:
            off = 10
            _, off = unpack_string(data, off)              # skip validator
            basket_id = struct.unpack_from('<I', data, off)[0]; off += 4
            container_type = data[off]; off += 1
            # remaining: String(chunk) + Short×4 (not needed for response)

            if IS_PUBLIC:
                # ── PUBLIC mode: serve from local container file ──────────
                basket_key = str(basket_id)
                containers = load_containers()
                slots = containers.get(basket_key, {}).get("slots", [])

                resp = bytes([0x1b])
                resp += struct.pack('<I', basket_id)
                resp += pack_basket_contents(slots)
                send_packet(conn, 2, resp, "CONTAINER_RESPONSE")
                print(f"  [CONTAINER] Opened basket_id={basket_id}, slots={len(slots)}")
            else:
                # ── FRIEND mode: relay to host ────────────────────────────
                # Host's OnReceive case 0x1C expects:
                #   String(requester_name) + Long(basket_id)
                # Host loads from disk and sends back 0x1B:
                #   String(requester_name) + Long(basket_id) + BasketContents
                host_conn, hname = get_host_conn()
                if host_conn is None:
                    # No host — send empty container
                    resp = bytes([0x1b])
                    resp += struct.pack('<I', basket_id)
                    resp += struct.pack('<H', 0)  # 0 slots
                    send_packet(conn, 2, resp, "CONTAINER_RESP_FALLBACK")
                elif username == hname:
                    # Host opens its own container — it reads locally.
                    # This can happen, so send a request back to itself.
                    # Actually the host handles containers via ShouldSaveLocally
                    # but the game still sends 0x1A to the server. We need to
                    # relay 0x1C to the host so it loads from disk and responds.
                    relay = bytes([0x1C])
                    relay += pack_string(username)
                    relay += struct.pack('<I', basket_id)
                    send_packet(host_conn, 2, relay, "RELAY_CONTAINER_REQ_HOST_SELF")
                    with pending_lock:
                        bk = str(basket_id)
                        if bk not in pending_container_requests:
                            pending_container_requests[bk] = []
                        pending_container_requests[bk].append(username)
                    print(f"  [RELAY] Container req (host self) basket_id={basket_id}")
                else:
                    # Guest requests container → relay to host
                    with pending_lock:
                        bk = str(basket_id)
                        if bk not in pending_container_requests:
                            pending_container_requests[bk] = []
                        pending_container_requests[bk].append(username)
                    print(f"  [RELAY] Forwarding container req to host, basket_id={basket_id}")

                    relay = bytes([0x1C])
                    relay += pack_string(username)
                    relay += struct.pack('<I', basket_id)
                    send_packet(host_conn, 2, relay, "RELAY_CONTAINER_REQ_TO_HOST")

    elif pid == 0x1e: # CLOSE_BASKET
        # C2S: Byte(0x1e) + String(validator) + Long(basket_id) + BasketContents + String(item_name)
        #      + String(chunk) + Short×4
        # S2C 0x1e: Byte(0x1e) + Long(basket_id) + BasketContents + String(item_name)
        if username:
            off = 10
            _, off = unpack_string(data, off)              # skip validator
            basket_id = struct.unpack_from('<I', data, off)[0]; off += 4
            slots, basket_raw, off = read_basket_contents(data, off)
            item_name, off = unpack_string(data, off)
            # remaining: String(chunk) + Short×4 (not needed)

            if IS_PUBLIC:
                # Save container contents (PUBLIC only)
                basket_key = str(basket_id)
                containers = load_containers()
                containers[basket_key] = {"slots": slots}
                save_containers(containers)
                print(f"  [CONTAINER] Saved basket_id={basket_id}, slots={len(slots)}")
            else:
                # FRIEND MODE: Relay save/close to host
                host_conn, hname = get_host_conn()
                if host_conn and username != hname:
                    # Send Byte(0x1E) + String(requester) + Long(id) + Contents + ItemName
                    relay = bytes([0x1E]) + pack_string(username) + struct.pack('<I', basket_id) + basket_raw + pack_string(item_name)
                    send_packet(host_conn, 2, relay, "RELAY_CLOSE_BASKET_TO_HOST")
                    print(f"  [RELAY] Forwarded container save for {basket_id} to host")

            # Broadcast S2C 0x1e to other clients so they sync
            s2c = bytes([0x1e])
            s2c += struct.pack('<I', basket_id)
            s2c += basket_raw
            s2c += pack_string(item_name)
            broadcast(s2c, exclude=username, label="RELAY_CLOSE_BASKET")

    elif pid == 0x0b and not IS_PUBLIC: # HOST's ZONE DATA RESPONSE (friend mode only)
        # Host's case 0x10 handler (it was mapped to 0x0A) built this packet:
        # Byte(0x0B) + String(requester) + String(zone) + Byte(type) + PackPos + ZoneData + Trail
        # Guest's case 0x0B handler expects:
        # Byte(success_flag) + Byte(sub_flag) + [ProcessIncomingZoneData reads rest...]
        # where ProcessIncomingZoneData reads: String(zone), ZoneData, etc.
        if username:
            requester_name, off = unpack_string(data, 10)
            
            # The host response has extra info (requester, type_flag, position).
            # We need to transform it to the guest format: 0x0B | flag(1) | sub(0) | zone | rest...
            # The host's 0x0B packet after requester has:
            # String(zone_name) + Byte(type_flag) + [Optional Position] + ZoneDataBody...
            
            zone_name, zoff = unpack_string(data, off)
            type_flag = data[zoff]
            
            # Find the actual start of ZoneDataBody. if type_flag & 0xFE == 2, there is position
            pos_skip = 0
            if (type_flag & 0xFE) == 2:
                pos_skip = 12 # 3 floats (Vector3) = 12 bytes
            
            zone_data_body_start = zoff + 1 + pos_skip
            
            # Reconstruct for guest: Byte(0x0B) + Byte(1) + Byte(0) + String(zone) + rest
            out  = bytes([0x0B, 1, 0])
            out += pack_string(zone_name)
            out += data[zone_data_body_start:total_len]
            
            # Route to requester
            zone_key = f"zone_{zone_name}"
            targets = []
            with pending_lock:
                if zone_key in pending_chunk_requests:
                    targets = pending_chunk_requests.pop(zone_key)
            
            if targets:
                for target_uname in targets:
                    send_private(target_uname, out, f"RELAY_ZONE_RESP→{target_uname}")
                print(f"  [RELAY] Forwarded host zone {zone_name} to {targets}")
            else:
                print(f"  [RELAY] Got host zone response for {zone_name} but no pending requests")

    elif pid == 0x0d and not IS_PUBLIC: # HOST's CHUNK RESPONSE (friend mode only)
        # The host's OnReceive case 0x0C handler built this packet:
        #   Byte(0x0D) + String(requester_name) + [ChunkData packed] + [bandit data]
        # We strip the requester name, look up who asked, and send the rest.
        if username:
            requester_name, off = unpack_string(data, 10)
            # The rest of the payload (from 'off' onward) is the chunk data
            # that the guest's OnReceive case 0x0D handler expects.
            # But wait — the guest 0x0D handler reads:
            #   String(zone_string_?) + Short(x) + Short(z) + Byte(flag) + String(checkpoint)
            #   then ChunkData$$UnpackFromWeb ...
            # The host's PutByte(0x0D) + PutString(requester) are at the front.
            # What the host sends AFTER requester is:
            #   ChunkData$$PackForWeb output = Short(cx)+Short(cz)+String(zone)+4shorts+String+String+tiles+landclaims
            #   Then bandit data.
            # The guest's 0x0D handler reads:
            #   String(zone_display) + Short(cx) + Short(cz)  → from the outer wrapper
            #   then Byte(flag) + String(checkpoint) → before ChunkData body
            #   then ChunkData$$UnpackFromWeb → the body
            #   then Short(bandit_count) + bandit data
            #
            # So the host's PackForWeb only outputs the ChunkData BODY (cx,cz,zone,shorts,strings,tiles,landclaims),
            # but the S2C 0x0D wrapper adds: String(zone_display) BEFORE the body,
            # and the body itself starts with short(cx)+short(cz).
            #
            # Actually looking at case 0x0D decomp again (lines 1078+):
            # Guest reads: String(zone_display), Short(x), Short(z), then gets chunk_string,
            # then Byte(flag), String(checkpoint), then if flag==0 → UnpackFromWeb.
            #
            # The host case 0x0C builds:
            #   PutByte(0x0D), PutString(requester), then:
            #   PutString(uVar6) → this is the first string the host read = requester name
            #                      Wait no — let me re-check...
            #
            # Actually in the host case 0x0C:
            #   uVar6 = GetString  → requester_name
            #   uVar24 = GetString → zone_name
            #   uVar12 = GetShort → x
            #   uVar22 = GetShort → z
            #   ...
            #   iVar30 (new packet) PutByte(0x0D)
            #   PutString(iVar30, uVar6)  → puts REQUESTER name
            #   ChunkData$$PackForWeb(iVar23, iVar30) → packs chunk body
            #   then bandit_count + bandit_camps
            #
            # So the host response is:
            #   Byte(0x0D) + String(requester) + [PackForWeb body] + Byte(bandit_count) + [bandits]
            #
            # And the guest 0x0D handler reads:
            #   String(zone_display) + Short(cx) + Short(cz) → but PackForWeb starts with Short(cx)+Short(cz)+String(zone)...
            #
            # Wait, looking at the guest handler more carefully:
            #   Line 1083: uVar6 = GetString      → this reads the first string AFTER 0x0D
            #   Line 1084: uVar24 = GetShort       → first short
            #   Line 1085: uVar12 = GetShort       → second short
            # Then it uses these to build a chunk_string via GetChunkString(zone, x, z)
            # Then line 1099: GetByte → flag
            # Then line 1100: GetString → checkpoint
            #
            # So the outer 0x0D wrapper that the guest expects:
            #   String(zone_name) + Short(cx) + Short(cz) + Byte(flag) + String(checkpoint)
            #   THEN: ChunkData$$UnpackFromWeb body (which also starts with short,short,string...)
            #
            # But the HOST's response puts String(requester) first, then PackForWeb.
            # PackForWeb outputs: Short(cx)+Short(cz)+String(zone)+4shorts+String+String+tiles+land_claims
            #
            # So the HOST's response layout is:
            #   0x0D | String(requester) | Short(cx) | Short(cz) | String(zone) | ...
            # But the GUEST wants:
            #   0x0D | String(zone_name) | Short(cx) | Short(cz) | Byte(flag) | String(checkpoint) | UnpackFromWeb...
            #
            # These don't match! This means we need to TRANSFORM the host's
            # response into the format the guest expects.
            #
            # The simplest approach: we already know the zone/x/z from the
            # pending request. We take the PackForWeb body from the host
            # and wrap it in our own make_dummy_chunk-style outer wrapper.
            #
            # From the host's response, after String(requester), the rest is
            # the PackForWeb output = the chunk body. We need to wrap it as:
            #   Byte(0x0D) + String(zone) + Short(cx) + Short(cz) ...
            #
            # Actually, let's look at this differently. The PackForWeb output
            # starts: Short(cx), Short(cz), String(zone_name), 4 shorts, String, String,
            # tiles, land_claims. And THEN the host code appends bandit data.
            # The guest handler reads:
            #   1. String(zone_display)  → outer zone
            #   2. Short(cx), Short(cz)  → from outer wrapper
            #   3. GetChunkString(zone, cx, cz)
            #   4. Byte(flag)            → 0 = new chunk
            #   5. String(checkpoint)
            #   6. ChunkData$$UnpackFromWeb → which reads:
            #       Short(cx), Short(cz), String(zone), 4shorts, String, String, tiles, land_claims
            #   7. Short(bandit_count) + bandits
            #
            # So the full guest-expected format of JUST the data part is:
            #   String(zone) + Short(cx) + Short(cz) + Byte(0) + String("") + [UnpackFromWeb body] + Short(bandit_count) + ...
            #
            # And the host's Pack output is:
            #   [PackForWeb body] + Byte(bandit_count) + [bandits]
            #
            # So: host_response_data[off:] = PackForWeb_body + bandit_data
            # We wrap it as:
            #   Byte(0x0D) + String(zone) + Short(cx) + Short(cz) + Byte(0) + String("") + PackForWeb_body + bandit_data
            #
            # But we need to extract cx, cz, zone from PackForWeb to build the
            # outer wrapper. PackForWeb starts: Short(cx)+Short(cz)+String(zone)
            # so we can peek into the host's data to get these.

            chunk_data = data[9+1+len(pack_string(requester_name)):total_len]  # strip pid byte + requester string
            # Wait, data[9] is the pid byte, data[10:] starts after pid.
            # off already points past the requester string.
            # chunk_data = everything after requester = PackForWeb output + bandit data
            chunk_data = data[off:total_len]

            if len(chunk_data) >= 4:
                # Peek at PackForWeb: first 4 bytes = Short(cx) + Short(cz)
                cx, cz = struct.unpack_from('<hh', chunk_data, 0)
                # Next is String(zone_name)
                zone_name_from_host, zoff = unpack_string(chunk_data, 4)

                # Build the outer wrapper the guest expects
                out  = bytes([0x0D])
                out += pack_string(zone_name_from_host)  # outer zone_name
                out += struct.pack('<hh', cx, cz)        # outer chunk coords
                out += bytes([0])                        # flag = 0 (new chunk data)
                out += pack_string("")                   # checkpoint = empty
                out += chunk_data                        # PackForWeb body + bandit data

                # Find who requested this and send to them
                # Build chunk_key from the zone + coords
                chunk_key = f"{zone_name_from_host}_{cx}_{cz}"
                targets = []
                with pending_lock:
                    if chunk_key in pending_chunk_requests:
                        targets = pending_chunk_requests.pop(chunk_key)

                if targets:
                    for target_uname in targets:
                        send_private(target_uname, out, f"RELAY_CHUNK_RESP→{target_uname}")
                    print(f"  [RELAY] Forwarded host chunk {chunk_key} to {targets}")
                else:
                    print(f"  [RELAY] Got host chunk response for {chunk_key} but no pending requests")
            else:
                print(f"  [!] Host chunk response too short ({len(chunk_data)} bytes)")

    elif pid == 0x1b and not IS_PUBLIC: # HOST's CONTAINER RESPONSE (friend mode only)
        # Host's case 0x1C handler built:
        #   Byte(0x1B) + String(requester) + Long(basket_id) + BasketContents$$Pack
        # Guest's case 0x1B handler expects:
        #   Long(basket_id) + BasketContents
        if username:
            requester_name, off = unpack_string(data, 10)
            # The rest (from off) is: Long(basket_id) + BasketContents
            container_data = data[off:total_len]

            if len(container_data) >= 4:
                basket_id = struct.unpack_from('<I', container_data, 0)[0]

                # Build guest-format response: Byte(0x1B) + Long(basket_id) + BasketContents
                out = bytes([0x1B]) + container_data

                # Find who requested this container
                bk = str(basket_id)
                targets = []
                with pending_lock:
                    if bk in pending_container_requests:
                        targets = pending_container_requests.pop(bk)

                if targets:
                    for target_uname in targets:
                        send_private(target_uname, out, f"RELAY_CONTAINER_RESP→{target_uname}")
                    print(f"  [RELAY] Forwarded host container {basket_id} to {targets}")
                else:
                    print(f"  [RELAY] Got host container response for {basket_id} but no pending requests")
            else:
                print(f"  [!] Host container response too short")

    elif pid == 0x42: # MOB_DATA_REQUEST — route to specific mob owner
        target_owner, off = unpack_string(data, 10)
        if username and target_owner:
            relay = bytes([0x42]) + pack_string(username) + data[off:]
            send_private(target_owner, relay, "RELAY_MOB_REQ")

    elif pid == 0x43: # MOB_DATA_RESPONSE — route back to requester
        target_requester, off = unpack_string(data, 10)
        if username and target_requester:
            relay = bytes([0x43]) + pack_string(username) + data[off:]
            send_private(target_requester, relay, "RELAY_MOB_RESP")

    # ── Combat packets: strip validator, relay correctly ────────────────────
    elif pid == 0x46: # ATTACK_ANIM (cosmetic — mob plays attack animation)
        # C2S: Byte(0x46) + String(validator) + String(mob_id)
        # S2C: String(mob_id)
        if username:
            off = 10
            _, off = unpack_string(data, off)  # skip validator
            broadcast(bytes([pid]) + data[off:], exclude=username, label="RELAY_ATTACK_ANIM")

    elif pid == 0x47: # HIT_MOB
        # C2S: String(validator) + String(target) + Long(damage) + Long(secondary) + Byte(hit_type) + Byte(perk) + Byte(bonus) + String(attacker)
        # S2C: same minus validator
        if username:
            off = 10
            _, off = unpack_string(data, off)  # skip validator
            after_validator = data[off:]

            if not IS_PVP:
                # Check if target is a player — if so, zero damage
                target, toff = unpack_string(data, off)
                with players_lock:
                    target_is_player = target.lower() in players
                if target_is_player:
                    # Zero out damage (Long=4 bytes) and secondary (Long=4 bytes)
                    target_bytes = data[off:toff]  # String(target) including length prefix
                    rest_after_damage = data[toff + 8:]  # skip 4+4 bytes of damage+secondary
                    zeroed = target_bytes + struct.pack('<II', 0, 0) + rest_after_damage
                    broadcast(bytes([pid]) + zeroed, exclude=username, label="RELAY_HIT_PVP_OFF")
                    print(f"  [PVP] Zeroed damage from {username} → {target} (PvP disabled)")
                    return

            broadcast(bytes([pid]) + after_validator, exclude=username, label="RELAY_HIT")

    elif pid == 0x48: # MOB_DIE
        # C2S: String(validator) + String(mob_id) + Short×2 + String(mob_type) + Short×5 + String(killer) + Byte×2 + InventoryItem
        # S2C: same minus validator
        if username:
            off = 10
            _, off = unpack_string(data, off)  # skip validator
            broadcast(bytes([pid]) + data[off:], exclude=username, label="RELAY_MOB_DIE")

    # ── Perk packets: strip validator, relay as-is ────────────────────────
    elif pid == 0x51: # APPLY_PERK
        # C2S: String(validator) + String(caster) + Long(timing) + String(target) + PerkData + Short(slot) + String(effectKey) + Byte(isInitial)
        # S2C: same minus validator
        if username:
            off = 10
            _, off = unpack_string(data, off)  # skip validator
            broadcast(bytes([pid]) + data[off:], exclude=username, label="RELAY_PERK")

    elif pid == 0x52: # LAUNCH_PROJECTILE
        # C2S: String(validator) + PerkData + Short(slot) + String(caster) + String(target) + Long(timing) + Position×2
        # S2C: same minus validator
        if username:
            off = 10
            _, off = unpack_string(data, off)  # skip validator
            broadcast(bytes([pid]) + data[off:], exclude=username, label="RELAY_PROJECTILE")

    elif pid == 0x54: # ALL_PRE_PERKS (batch perks)
        # C2S: String(validator) + String(caster_id) + Short(count) + [per-perk data]×count
        # S2C: same minus validator
        if username:
            off = 10
            _, off = unpack_string(data, off)  # skip validator
            broadcast(bytes([pid]) + data[off:], exclude=username, label="RELAY_BATCH_PERKS")

    elif pid == 0x55: # CREATE_PERK_DROP
        # C2S: String(validator) + Position(4 shorts) + String + PerkData + Short + String + Long
        # S2C: same minus validator
        if username:
            off = 10
            _, off = unpack_string(data, off)  # skip validator
            broadcast(bytes([pid]) + data[off:], exclude=username, label="RELAY_PERK_DROP")

    # ── Pool/Minigame packets: relay between players ───────────────────
    elif pid == 0x35: # CHALLENGE_MINIGAME
        # C2S: String(target_player) + Byte(minigame_type)
        # S2C: same — route to the target player
        if username:
            target, off = unpack_string(data, 10)
            minigame_type = data[off] if len(data) > off else 0
            # Send to target: Byte(0x35) + String(challenger) + Byte(type)
            relay = bytes([0x35]) + pack_string(username) + bytes([minigame_type])
            send_private(target, relay, "RELAY_CHALLENGE")
            print(f"  [POOL] {username} challenges {target} to minigame type={minigame_type}")

    elif pid == 0x36: # MINIGAME_RESP
        # C2S: Byte(response) + String(challenger) + Byte(difficulty) + rest
        # S2C: same — route back to the challenger
        if username:
            resp_type = data[10] if len(data) > 10 else 0
            challenger, off = unpack_string(data, 11)
            rest = data[off:]
            relay = bytes([0x36, resp_type]) + pack_string(username) + rest
            send_private(challenger, relay, "RELAY_MINIGAME_RESP")

    elif pid == 0x37: # BEGIN_MINIGAME
        # C2S: String(opponent) + Byte(difficulty) + Byte(game_type)
        # S2C: same — relay to opponent
        if username:
            opponent, off = unpack_string(data, 10)
            rest = data[off:]
            relay = bytes([0x37]) + pack_string(username) + rest
            send_private(opponent, relay, "RELAY_BEGIN_MINIGAME")

    elif pid == 0x38: # EXIT_MINIGAME
        # Broadcast to everyone (opponent needs to know)
        if username:
            broadcast(bytes([0x38]) + pack_string(username), exclude=username, label="RELAY_EXIT_MINIGAME")

    elif pid in (0x39, 0x3a, 0x3b, 0x3c, 0x3d): # POOL_CUE_POS, POOL_SHOOT, POOL_SYNC_READY, POOL_PLACE_BALL, POOL_PLAY_AGAIN
        # All pool gameplay packets: broadcast with username prepended
        if username:
            broadcast(bytes([pid]) + pack_string(username) + data[10:], exclude=username, label=f"RELAY_{pname}")

    # ── Non-validator packets: prepend username as sender ─────────────────
    elif pid in (0x16, 0x18, 0x19, 0x23, 0x4a, 0x4b, 0x53, 0x09, 0x56):
        if username:
            broadcast(bytes([pid]) + pack_string(username) + data[10:], exclude=username, label=f"BCAST_{pname}")

    elif pid == 0x2d: # ASK_JOIN
        target_friend, off = unpack_string(data, 10)
        if username and target_friend:
            relay_payload = bytes([0x2d]) + pack_string(username) + data[off:]
            send_private(target_friend, relay_payload, "RELAY_ASK_JOIN")

    elif pid == 0x2b: # YOU_MAY_JOIN / USED_ID
        if total_len - 10 <= 10: # Likely USED_ID
            if username:
                broadcast(data[9:total_len], exclude=username, label="RELAY_USED_ID")
        else:
            target_friend, off = unpack_string(data, 10)
            if username and target_friend:
                relay_payload = bytes([0x2b]) + pack_string(username) + data[off:]
                send_private(target_friend, relay_payload, "RELAY_YOU_MAY_JOIN")

    elif pid == 0x0F: # HEARTBEAT
        if not IS_PUBLIC and username:
            with host_lock:
                if username == host_player:
                    global host_last_seen
                    host_last_seen = time.time()
        send_packet(conn, 2, b'\x0F', "HB")

    else:
        if username:
            broadcast(bytes([pid]) + pack_string(username) + data[10:], exclude=username, label=f"RELAY_{name}")
        else:
            hd = binascii.hexlify(data[9:min(len(data), 32)]).decode()
            print(f"  [WARN] Unhandled pid 0x{pid:02X}. payload starts at index 9. Hex: {hd}")


# ── entry point ──────────────────────────────────────────────────────────────
def run():
    global start_time
    start_time = time.time()
    
    if not IS_PUBLIC:
        threading.Thread(target=host_watchdog, daemon=True).start()

    # Generate catalogue chunks if enabled
    _generate_catalogue_chunks()

    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        s.bind(('0.0.0.0', PORT))
        s.listen()
        print(f"[*] Game server up  port={PORT}  room_token={ROOM_TOKEN!r}  public={IS_PUBLIC}  catalogue={CATALOGUE_ENABLED}")
        print(f"[*] Packet logging active to file: {LOG_FILENAME}")
        if IS_PUBLIC:
            print(f"[*] Chunk data dir: {CHUNK_DIR}")
        else:
            print(f"[*] FRIEND MODE: chunks/containers will be relayed from the host client")
        if not IS_PUBLIC:
            print(f"[*] Auto-kill after {INACTIVITY_TIMEOUT}s of inactivity (no players)")
        while True:
            c, a = s.accept()
            threading.Thread(target=handle_client, args=(c, a), daemon=True).start()

if __name__ == "__main__":
    run()
