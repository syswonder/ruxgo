# Ruxgo

Ruxgo is a Cargo-like build tool for building C and C++ applications that relies solely on a Toml file. 

**To start using Ruxgo**, learn more at [The Ruxgo Book](https://ruxgo.syswonder.org/).

🚧 Working In Progress. 

## Installation

To build the `ruxgo` executable from source, you will first need to install Rust and Cargo. Follow the instructions on the [Rust installation page](https://www.rust-lang.org/tools/install). Ruxgo currently requires at least Rust version 1.74.

Once you have installed Rust, the following command can be used to build and install Ruxgo:

```sh
cargo install --git https://github.com/syswonder/ruxgo.git ruxgo
```

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

You can also configure the log level with the environment variable `"RUXGO_LOG_LEVEL"`, the default log level is "Info".

## Ruxgo-apps

The `ruxgo/apps/` directory places all the Toml files that have been tested, you can switch to either directory and follow the instructions to build the application. Currently, there are two ways to build an app:

- If building locally, you'll need to download the apps source code and then use ruxgo to build and run it.

- If you want to build on ruxos, you need to copy `config_<platform>.toml` from `ruxgo/apps/<name>/ruxos` into `ruxos/apps/c/<name>`, then download the apps source code and use ruxgo to build and run it.

**Note:** Refer to the README.md in each app directory for details. The following applications are already supported:

* [x] [redis](apps/redis)
* [x] [sqlite3](apps/sqlite3)
* [x] [iperf](apps/iperf)
* [x] helloworld
* [x] memtest
* [x] httpclient
* [x] httpserver
* [x] nginx
* [x] python3