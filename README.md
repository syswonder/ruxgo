# Ruxgo

Ruxgo is a Cargo-like build tool for building C and C++ applications that relies solely on a toml file. It abandons the complex syntax and rule-dependent construction in the original MAKE tool, exposing the most original compilation process. If you hate using Makefile, might as well try Ruxgo to build applications, just take a few minutes!

For a project you want to build, you just need to figure out the source file path, header file path, cflags and ldflags, then fill them into the appropriate places in the toml file. Ruxgo does the rest, so easy!

🚧 Working In Progress. 

## Installation

To build the `ruxgo` executable from source, you will first need to install Rust and Cargo. Follow the instructions on the [Rust installation page](https://www.rust-lang.org/tools/install). Ruxgo currently requires at least Rust version 1.70.

Once you have installed Rust, the following command can be used to build and install Ruxgo:

```sh
cargo install --git https://github.com/Ybeichen/ruxgo.git ruxgo
```

This will automatically download `ruxgo`, build it, and install it in Cargo's global binary directory (`~/.cargo/bin/` by default).

To uninstall, run the command `cargo uninstall ruxgo`.

## Features & TODOs

* [x] Multithreaded
* [x] Can generate compile_commands.json
* [x] Can generate .vscode/c_cpp_properties.json
* [x] Auto add project libraries to other targets
* [x] Get libraries as packages from github
* [x] Supported ruxos and different platforms
* [x] Supported run by qemu
* [x] Supported ruxlibc and ruxmusl
* [ ] Create new project

## Usage

Write a `config_linux.toml` for linux and `config_win32.toml` for windows in the project directory.

You can then build the project with:
```console
ruxgo -b
```

Once built, you can execute the project via:
```console
ruxgo -r
```

For help:
```console
ruxgo --help
```

The help command will show you the following:
```sh
Usage: ruxgo [OPTIONS] [CHOICES]... [COMMAND]

Commands:
  init    Initialize a new project Defaults to C++ if no language is specified
  config  Configuration settings
  help    Print this message or the help of the given subcommand(s)

Arguments:
  [CHOICES]...  Choose which parts to delete

Options:
  -b, --build                   Build your project
  -c, --clean                   Clean the obj and bin intermediates
  -r, --run                     Run the executable
      --bin-args <BIN_ARGS>...  Arguments to pass to the executable when running
      --gen-cc                  Generate compile_commands.json
      --gen-vsc                 Generate .vscode/c_cpp_properties.json
      --update-packages         Update packages
      --restore-packages        Restore packages
  -h, --help                    Print help
  -V, --version                 Print version
```

You can also configure the log level with the environment variable `"RUXGO_LOG_LEVEL"`, the default log level is "Info".

## Ruxgo-apps

The `apps/` directory places all the toml files that have been tested. Currently, there are two ways to build an app: locally and on ruxos

- If building locally, you'll need to download the apps source code and then use ruxgo to build and run it.

- If you want to build on ruxos, you need to copy `config_linux.toml` from `ruxgo/apps/<name>/ruxos` into `ruxos/apps/c/<name>`, then download the apps source code and use ruxgo to build and run it.

**Note:** Refer to the README.md in each app directory for details. The following applications are already supported:

* [x] [redis](apps/redis)
* [x] [sqlite3](apps/sqlite3)
* [x] [iperf](apps/iperf)
* [x] helloworld
* [x] memtest
* [x] httpclient
* [x] httpserver
* [x] nginx
* [ ] python3

## TOML Module Description

Toml file consists of one **[build]** module and multiple **[targets]** modules. If you want to run on ruxos, you can add the **[os]** module. Here is a description of each module:

The **[build]** module describes the compiler type and remote library packages. It contains two parts: `compiler` and `packages`.

- `compiler`: Specifies the compiler type, for example: "gcc".
- `packages`: Optional, mainly used to get the app source code from Github, and then by parsing the `config_linux.toml` file in it to get the required libraries. When using packages, you need to specify the remote repository and branch.

The **[targets]** module is the core part of the Toml and is used to describe the source build process and dependencies between libraries, as described below:

- `name`: Specifies the target name, if it is of the "dll" type, must begin with "lib_".
- `src`：Specifies the path to the target source code.
- `src_excluded`：Optional. if you want to exclude some source files or directories, you can specify here.
- `include_dir`：Specifies the path to the header file in the target source code, which allows string and vector types.
- `type`：Specifies the type of the target, which can be of type "static", "dll", "object", or "exe". It should be noted that there can be only one "exe" target in a toml file, but there can be multiple targets of other types.
- `cflags`：Specifies the compilation options of the target.
- `archive`：Optional. If the targets type is static, you can specify an archive tool to create the static library.
- `linker`: Optional. Specifies the linking tool is used to link object files, static libraries, and dynamic libraries. If the value is missing, it is specified according to the `compiler`.
- `ldflags`：Specifies the link options of the target.
- `deps`：Specifies other targets to depend on.

The **[os]** module is optional. If you want to run locally, **[config]** and **[targets]** are completely satisfied, if you want to run on ruxos, you can add the **[os]** module. After adding the **[os]** module, the original content of the corresponding **[targets]** modules will be changed. Ruxgo runs smoothly on ruxos by changing compiler, cflags, and ldflags in the backend, such as:

When the platform of the **[os]** module is "x86_64-qemu-q35", the compiler is no longer "gcc", it becomes "x86_64-linux-musl-gcc". Also, all **[targets]** cflags are added with "-nostdinc -fno-builtin -ffreestanding -Wall" by default, you do not need to add them manually. Accordingly, when the type of **[targets]** is "exe", ldflags adds "-nostdlib -static -no-pie --gc-sections" by default. Of course, there are other default additions depending on architecture and platform. Just like, you just need to splice the **[os]** module onto a module that can run locally! The details are as follows:

- `name`: Specifies the name of the os.
- `services`: Specifies the services that the os can provide, similar to the features in ruxos.
- `ulib`: The user library you want to use, the options are: "ruxlibc", "ruxmusl".
- `platform`：If needed, configure it in **[os.platform]**.

If you want to configure the platform further, you can do so in **[os.platform]** , if empty, take the default value. The details are as follows:

- `name`: Specifies what platform you want the os to run on, including: "x86_64-qemu-q35", "aarch64-qemu-virt", "riscv64-qemu-virt". The default value is "x86_64-qemu-q35".
- `smp`: Specifies the number of CPUs. The default value is "1".
- `mode`: Specifies the build mode, including: "release","debug". The default value is "release".
- `log`: Specifies the log level, including: "warn", "error", "info", "debug", and "trace". The default value is "warn".
- `v`: Specifies the Verbose level, including: "", "1", "2". The default value is "".
- `qemu`: If needed, configure it in **[os.platform.qemu]**.

If your platform depends on qemu, you'll need to configure it further in **[os.platform.qemu]**, if empty, take the default value. The details are as follows:

- `blk`: Specifies whether to enable storage devices (virtio-blk). The default value is "n".
- `net`: Specifies whether to enable network devices (virtio-net). The default value is "n".
- `graphic`: Specifies whether to enable display devices and graphic output (virtio-gpu). The default value is "n".
- `disk_img`: Specifies the path to the virtual disk image. The default value is "./disk_img".
- `v9p`: Specifies whether to enable virtio-9p devices. The default value is "n".
- `v9p_path`: Specifies the host path for backend of virtio-9p. The default value is "./".
- `qemu_log`: Specifies whether to enable QEMU logging (log file is "qemu.log"). The default value is "n".
- `net_dump`: Specifies whether to enable network packet dump (log file is "netdump.pcap"). The default value is "n".
- `net_dev`: Specifies QEMU netdev backend types: "user" or "tap". The default value is "user".
- `ip`: Specifies IPv4 address of os. The default value is "10.0.2.15" for QEMU user netdev.
- `gw`: Specifies gateway of IPv4 address. The default value is "10.0.2.2" for QEMU user netdev.
- `args`: Specifies the command-line arguments, separated by comma. It is used to pass specific variables, like `argc`, `argv`. The default value is "".
- `envs`: Specifies the environment variables, separated by comma between key value pairs. The default value is "".

Sample file with a library and an executable (run locally):

```toml
[build]
compiler = "gcc"

[[targets]]
name = "libsqlite3"
src = "./sqlite-amalgamation-3410100"
src_excluded = ["shell.c"]
include_dir = "./sqlite-amalgamation-3410100"
type = "static"
cflags = "-w -DSQLITE_THREADSAFE=0 -DSQLITE_OMIT_FLOATING_POINT -DSQLITE_OMIT_LOAD_EXTENSION -DSQLITE_DEBUG"
archive = "ar"
ldflags = "rcs"

[[targets]]
name = "local_sqlite3"
src = "./"
src_excluded = ["sqlite-amalgamation-3410100"]
include_dir = "./"
type = "exe"
cflags = ""
ldflags = ""
deps = ["libsqlite3"]
```

Sample file with a library and an executable (run on ruxos):

```toml
[build]
compiler = "gcc"

[os]
name = "ruxos"
services = ["fp_simd","alloc","paging","fs","blkfs"]
ulib = "ruxlibc"

[os.platform]
name = "x86_64-qemu-q35"
smp = "4"
mode = "release"
log = "error"

[os.platform.qemu]
blk = "y"
graphic = "n"

[[targets]]
name = "libsqlite3"
src = "./sqlite-amalgamation-3410100"
src_excluded = ["shell.c"]
include_dir = "./sqlite-amalgamation-3410100"
type = "static"
cflags = "-w -DSQLITE_THREADSAFE=0 -DSQLITE_OMIT_FLOATING_POINT -DSQLITE_OMIT_LOAD_EXTENSION -DSQLITE_DEBUG"
archive = "ar"
ldflags = "rcs"

[[targets]]
name = "ruxos_sqlite3"
src = "./"
src_excluded = ["sqlite-amalgamation-3410100"]
include_dir = "./"
type = "exe"
cflags = ""
linker = "rust-lld -flavor gnu"
ldflags = ""
deps = ["libsqlite3"]
```
