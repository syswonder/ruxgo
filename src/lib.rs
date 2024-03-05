//! A library for building and packaging C and C++ projects.
//!
//! This library automatically configures various targets in your project
//! and gives an easy interface to grab packages from github.
//!
//! The library uses config_linux.toml or config_win32.toml file to configure the project.
//!

/// Contains code to build projects
pub mod builder;
/// Contains code that handles various CLI flags
pub mod commands;
/// Handles global config
pub mod global_cfg;
/// Contains hashing related functions
pub mod hasher;
/// Contains packages management related functions
pub mod packages;
/// Contains parse related functions
pub mod parser;
/// Contains logger, config parser and environment config
pub mod utils;

#[macro_use]
extern crate lazy_static;
