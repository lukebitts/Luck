# Change Log

All notable per-release changes will be documented in this file. This project
adheres to [Semantic Versioning][sv].

[sv]: http://semver.org/

## Unreleased
### Added
* Added support to rust stable.
* Added a dynamic_tree collection to the common sub-module.
* Added a resource module to help with loading and parsing every kind of
    resource. Only .obj files can be loaded for now, though adding more types
    is easy.
* Added materials, meshs and a vertex type.
* Tentative Spatial system/component design.

### Changed
* Unfinished changes to the ecs sub-module.

### Removed
* Removed dependency on Clippy and added allow unknown lints to the Clippy
    allow lints. Clippy should still be run on the project but through Cargo.
* Removed benchmark tests since they are unstable.
* Removed dependency on FnBox since it is still unstable (and there is no
    advantage over FnMut?)

## 0.2.0 (2016-02-24) 🍀
### Added
* Added a language disclaimer to the CONTRIBUTING file.
* Added a rustfmt.toml to the project.
* Added a .travis.yml file to support integration with Travis.
* Added code to the math module.
* Added a "Building" session to the README file.

### Changed
* Changed the README file to look prettier in github.
* Bumped the version of the ECS module from 0.1.0 to 0.1.1
* Created some tests for the Quaternion code and fixed a bug with quaternion
    multiplication.

## 0.1.0 (2016-02-22)
### Added
* Completed the ECS module. Fully documented.

## 0.0.1 (2016-02-20)
* Initial release
