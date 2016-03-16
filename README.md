[![Build Status][s1]][tc]
[s1]: https://api.travis-ci.org/lukebitts/Luck.svg?branch=master
[tc]: https://travis-ci.org/lukebitts/Luck

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

To build Luck, simply run `cargo build` in the root folder or the child folders
if building one of the sub-projects.
