# RukosKit

A Cargo-like build tool for building C and C++ applications

🚧 Working In Progress. 

## Usage

Write a config_linux.toml for linux and a config_win32.toml for windows in the project directory

## Features & TODOs

* [x] Multithreaded
* [x] Can generate compile_commands.json
* [x] Can generate .vscode/c_cpp_properties.json
* [x] Auto add project libraries to other targets
* [x] Get libraries as packages from github
* [x] Create new project
* [x] Supported static libraries rust_lib
* [x] Supported ar and ld commands as optional
* [x] Supported the feature selection
* [ ] Qemu cmd
* [ ] Env cmd
* [ ] Support different Arch

## Installation

The tool currently only supports local installation
```console
cargo install --path .
```
For subcommands run with -h flag