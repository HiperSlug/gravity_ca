# Gravity Cellular Automaton
This project aims at providing a fast 2d simulation of gravity on a 2d grid of cells. 

# Status
Currently there is a small [demo](https://hiperslug.itch.io/gravity-cellular-automaton) for a single chunk simulation. A multi-chunk simulation is in progress.

# What is Cellular Automaton
In Cellular Automaton each cell is compared with its neighbors to determine the next state. \
To effectivly simulate gravity I use a bottom-up approach to allow cells to fall into spots that were previously occupied that frame.

# `Chunk` structure
Each `Chunk` is a 64x64 block of cells. \
Each cells stores: 

1. If it contains something.
2. If it should attempt to fall.

I am using `u64` bit masks to batch tests per row.

# Multi-`Chunk` simulation (WIP)
1. Parrallel simulation of chunks. This includes offsetting every other chunk by 32 cells to allow each chunk to only be able to mutate 4 chunks instead of 5.
2. Dynamic loading, simulation, and generation of chunks on demand.
