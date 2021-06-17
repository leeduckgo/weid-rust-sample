//! # WeBase
//!
//! A library to interact with webase.
#![warn(unused_extern_crates)]

pub mod weid_rest_service;
pub use self::weid_rest_service::*;

pub mod weid_generator;
pub use self::weid_generator::*;
