#![allow(irrefutable_let_patterns)] // crazy idiom

pub mod commands;
pub mod utils;

pub type Result<T, E = eyre::Error> = std::result::Result<T, E>;
