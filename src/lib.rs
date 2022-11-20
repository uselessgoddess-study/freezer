#![feature(decl_macro)]
#![feature(poll_ready)]
#![feature(try_blocks)]
#![deny(clippy::all, clippy::perf)]

pub mod auth;
pub mod errors;
pub mod handlers;
pub mod model;
pub mod service;
pub mod utils;

pub use errors::Result;
