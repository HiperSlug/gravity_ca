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
This will deterministically prioritize certain directions over others. I don't think it matters but I can later add randomness.

Each cell is stored as 2 bits: A `Some`/`None` bit and a Dynamic/Stationary bit. These bits are in separate channels and are packed into a u64.

Edited! - Each chunk of cells is 64x64 with one bit of padding. The single bit of padding should mean that chunks should avoid attempting to write the same cell at the same time as another chunk (this requires all cells to have deterministic falling priority or simulaneous computations on the same bit(b/c padding) may result in different outcomes). Additionally padding requires identical shift priorities, lest cells compete for a direction. Finally: Padding requires syncronization after EVERY operation. When doing left/right diagonal fall checking we need to check left, sync, check right, sync. 
New! - Each cell needs two bits of padding. In order to simulate a full pass where each cell can move exactly one cell per tick, then exactly two bits (the first one for simulating the movement of boundary cells, the second for simulating the movement of padding cells.) All simulation failing b/c of no more padding will be corrected during a syncronization step. This also means that if I allow a cell to move, for example, two cells away i'll need to increase the padding.


```rust
struct Chunk {
	some_mask: [u64; 64],
	dynamic_mask: [u64; 64],
}
```
The x position of a cell is the bit position. The y position of the cell is the array index.

We process all rows in a chunck except the first row. 1..63. 
For each row we use `&` to locate all cells which are dynamic and also have certain neighbor conditions.
Then once we have located all cells that satisfy the neighbor conditions we move cells in bulk.

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

After every right/left change we need to syncronize with the adjacnet chunks. And after we have finished processing a row we need to syncronize with the chunks below and above. Then we process the next row of chunks.
