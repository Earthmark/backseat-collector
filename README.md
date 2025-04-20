# backseat-collector

A small game using wasmtime to create robots that make an n-dimensional mine.

The user provides a program that each miner uses as a "brain", from there they try to gather resources.

The n-dimensionality is projected into 3d space based on matrices that collapse n-space to 3-space.
The user can control the projection.

The projection is done via a 3xN matrix.

## Architecture

The "brain" is a central entity, it is sent data from drones and can give each drone a single command to follow.

A drone will follow the last command it is provided if multiple are, if none is provided the drone will do nothing.

There is a central wasmtime engine and linker, there may be multiple brains as a stretch goal.
