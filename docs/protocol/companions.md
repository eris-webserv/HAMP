# Companion support in managed game servers

Personal companions are simulated and saved by the owning game client. HAMP
tracks their simulation owner and relays the existing client packets; it does
not create companion AI or replace the player's companion inventory files.

| Packet | Server behavior |
| --- | --- |
| `0x59` Created local mob | Registers the sender as owner; repeated announcements cannot steal another player's companion. Existing relay is preserved. |
| `0x3F` Try claim mobs | In managed mode, replies with a byte count and an ID/success byte for each request. A creature can have only one simulation owner. Relay mode still forwards requests to the host. |
| `0x41` Mob positions | Preserves the existing player-prefixed relay and caches a complete, owned movement snapshot for players joining later. |
| `0x42` / `0x43` Mob data | Existing requester/owner routing supplies the companion's CreatureStruct from its owning client. |
| `0x44` Request simulation transfer | In managed mode, claims an available world creature and sends `0x45` so the requesting client can recreate it locally. |
| `0x40` Deload mob | Releases the sender's ownership and sends the combat ID to peers. |
| `0x4E` / `0x4F` / `0x50` Equip, rename, destroy | Existing player-prefixed wire formats remain unchanged. Destroy also releases the sender's recorded ownership. |
| `0x13` Player gone | Includes owned creature IDs so peers remove companions when the owner disconnects or changes zones. |

On a zone change, personal companions remain assigned to their owner while
claimed world creatures become available to other players. Cached movement
is cleared to avoid replaying coordinates from the previous zone. Disconnects
clear all ownership and cached movement for that player.

Login sends `0x02` first. The client clears its previous map, queues its old
companions for Unity's deferred destruction, and replies with `0x03`. HAMP
waits 300 ms after that reply before sending `0x05` and initial zone data.
Sending companion recreation in the same receive pass as map clearing can
leave visible companion objects whose combat IDs were removed by the old
objects' destruction callbacks. Those companions cannot be targeted locally
or supplied in `0x43` responses to other players. The delay is a server-side
accommodation for that client lifecycle, not an explicit client readiness
acknowledgment.

Malformed claim requests are rejected before any assignments occur. Movement
snapshots must contain every declared ID and its two positions plus rotation
(24 bytes); foreign or unregistered creatures are not cached.

Validation includes a TCP lifecycle test with two simulated clients, preserving
the existing companion prefixes, plus ownership, malformed-request, movement,
zone-change, and departure-list tests. Companion visibility and interaction
after joining have also been verified with actual game clients.
