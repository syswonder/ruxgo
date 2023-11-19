//! A library for building and packaging C and C++ projects.
//!
//! This library automatically configures various targets in your project
//! and gives an easy interface to grab packages from github.
//!
//! The library uses config_linux.toml or config_win32.toml file to configure the project.
//!

/// Contains code to build projects
pub mod builder;
/// Contains code that handles various binary flags
pub mod commands;
/// Contains logger and config parser
pub mod utils;
/// Contains hashing related functions
pub mod hasher;
/// Contains features related functions
pub mod features;

pub mod qemu;