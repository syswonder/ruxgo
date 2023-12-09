# Ruxgo

A Cargo-like build tool for building C and C++ applications

🚧 Working In Progress. 

## Installation

The tool currently only supports local installation
```console
git clone https://github.com/Ybeichen/ruxgo.git && cd ruxgo
cargo build
cargo install --path .
```
For subcommands run with -h flag

## Features & TODOs

* [x] Multithreaded
* [x] Can generate compile_commands.json
* [x] Can generate .vscode/c_cpp_properties.json
* [x] Auto add project libraries to other targets
* [x] Get libraries as packages from github
* [x] Create new project
* [x] Supported run by qemu
* [x] Supported ruxos and different platforms

## Supported Apps

The currently supported applications (c):

* [x] helloworld
* [x] memtest
* [x] redis
* [x] sqlite3

## Usage

Write a config_linux.toml for linux and config_win32.toml for windows in the project directory

You can then build the project with:
```console
ruxgo -b
```

Once built, you can execute the unikernel via:
```console
ruxgo -r
``

For help
```console
ruxgo --help
```

The help command will show you the following
```sh
Usage: builder_cpp.exe [OPTIONS] [COMMAND]

Commands:
  init    Initialize a new project Defaults to C++ if no language is specified
  config  Configuration settings
  help    Print this message or the help of the given subcommand(s)

Options:
  -b, --build                   Build your project
  -c, --clean                   Clean the obj and bin intermediates
  -r, --run                     Run the executable
      --bin-args <BIN_ARGS>...  Arguments to pass to the executable when running
      --gen-cc                  Generate compile_commands.json
      --gen-vsc                 Generate .vscode/c_cpp_properties.json
      --clean-packages          Clean packages
      --update-packages         Update packages
      --restore-packages        Restore packages
  -h, --help                    Print help
  -V, --version                 Print version
```
