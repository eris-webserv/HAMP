# Overworld generation

The overworld generator follows the client's `ChunkControl`,
`ChunkGeneratorOverworld`, `ConstructionControl`, and `MobControl` rules.
`overworld_data.rs` contains the serialized biome definitions, biome-map pixels,
item footprints, and flower paint list.

Each 36×36 sector assigns biomes independently to color blobs. The default
selection pool is `[0, 0, 1, 2, 3, 4, 4, 6, 8, 9]`, filtered by the minimum
distance of every chunk center in the blob from the world origin. Sakura's
minimum distance is 100. Each blob receives two independent creature choices.
Ocean chunks become islands with probability 0.13; swamp chunks become marsh
with probability 0.5. These variants share their original blob's creature pair.

Chunks use separate scenic and item budgets, the client's nested rarity draws,
depth restrictions, additional clumps, and rotated occupied footprints. Clump
replacement items, flower paints, chest IDs, entrance metadata, mob size
probabilities, and green blob clustering follow the client rules. Forward scans
retain the client's behavior when removing occupied locations from the list.
Object depth measures from the breeding platform's gameplay position
`(-7.6617, 0.762, 6.4955)` using 32-bit floating point distance.

The server retains its SplitMix seed derivation and separate sector, floor, and
object random streams. Saved chunks and configured start-area overrides retain
precedence. Generation changes apply to newly generated chunks.
