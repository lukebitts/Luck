<img src="https://travis-ci.org/lukebitts/Luck.svg" />

<img align="left" width="148px" src="documentation/images/logo.png" />

# Luck

Luck is a personal game engine project I'm working on to help me learn Rust. I
do not actively support it besides working on whatever interests me, but feel
free to use this code if it is in any way useful to you. It is not by any means
completed and/or stable.


## Sub-projects

This project is divided into several sub-projects:

### ECS

Luck ECS is an Entity-Component system, it has no dependecy on the rest of the
engine and can be used by itself.

### Math

Luck Math depends on glm-rs and reexports the library adding a few functions and
types that glm-rs did not port from glm. It provides a Quaternion and AABB
types.

### Core

 Luck Core is the implementation of the engine as a framework, there you can
 find rendering code, physics, cameras, etc. It has a number of dependecies
 which can be found in the cargo file.

# Building

To build Luck, simply run `cargo build` in the root folder. It requires
`rustc 1.8.0-nightly (57c357d89 2016-02-16)`, if you remove [Clippy][clippy]
dependecy you can probably compile it on stable rust (but I haven't tested that
yet).

# Travis

The project doesn't build on Travis because [Clippy][clippy] requires a very
specific `rustc` version. I'll fix it eventually.

[clippy]: https://github.com/Manishearth/rust-clippy
