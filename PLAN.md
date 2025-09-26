# Plan
I plan on doing all computation on the cpu because gpu readback is slow and simulations are branch heavy.

Each cell has three possible states:
`None`,
`Some(Dynamic)`
`Some(Stationary)`

Every row is processed before ascending rows.
Each row is processed in two phases:
Phase 1:
If this cell is `Some(Dynamic)` and the cell below is `None` then fall.
Phase 2:
For both directions: If this cell is still `Some(Dynamic)` its adjacent cells is `None`, and its diagonal cell is `None` then fall diagonally. 
This will deterministically prioritize certain directions over others. I don't think it matters but I can later add randomness. Either way it needs to be determinsitic meaning that the rng is seeded either per row or per cell. 

Each cell is stored as 2 bits: A `Some`/`None` bit and a `Dynamic`/`Stationary` bit. These bits are in separate channels and are packed into a u64.

Each chunk of cells is 64x64. No padding. Neighbor access will be accomplished through raw pointers, so I have to ensure safety. Additionally diagonal chunks can be accessed indirectly by following a neighbor chunk. On removal Pointers are recursivly removed.

```rust
struct Chunk {
	some_mask: [u64; 64],
	dynamic_mask: [u64; 64],
	neighbors: [Option<NonNull<Chunk>>; 4],
}
```
The x position of a cell is the bit position. The y position of the cell is the array index.

When simulating gravity if a cell attempts to enter a nonexistent chunk we need to create it, but not simulate it until next frame.

ex:
```rust
for y in 1..64 { // exclude bottom padding, include top padding
	let dynamic_mask = chunk.dynamic_mask[y];
	let below_some_mask = chunk.some_mask[y - 1];

	let fall_mask = dynamic_mask & !below_some_mask;
	
	chunk.dynamic_mask[y] &= !fall_mask;
	chunk.some_mask[y] &= !fall_mask;

	chunk.dynamic_mask[y - 1] |= fall_mask;
	chunk.some_mask[y - 1] |= fall_mask;
}
```

# Parrallelism
We can process a single row of bits at the same time. To achieve this we group chunks by their y position. Then for each y in that layer of chunks we do the following:
1. Collect neighbor data for a row. For corner simulation multiple chunks may need to be referenced.
2. Simulate gravity for that row.
3. Repeat for each row in all chunks.

Step 1 and 2 are separate steps which should allow parrallelism because step 1 requires multiple immutable references to the same chunks while step two requires mutable access to every chunk. In other words both of these steps can be easily parrallelized.



# NVM MVP is just a single chunk simulating