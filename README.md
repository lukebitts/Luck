<img align="left" width="64px" src="documentation/images/logo.png" />

# Luck

Luck is a personal game engine project I'm working on to help me learn Rust. I
do not actively support it besides working on whatever interests me, but feel
free to use this code if it is in any way useful to you.

This project is divided into several sub-projects:

 - luck_ecs

Luck ECS is an Entity-Component system, it has no dependecy on the rest of the
engine and can be used by itself.

 - luck_math

Luck Math depends on glm-rs and reexports the library adding a few functions and
types that glm-rs did not port from glm. It provides a Quaternion and AABB
types.

 - luck_core

 Luck Core is the implementation of the engine as a framework, there you can
 find rendering code, physics, cameras, etc. It has a number of dependecies
 which can be found in the cargo file.
