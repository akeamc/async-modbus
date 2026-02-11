#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

#[cfg(feature = "embedded-io")]
pub mod embedded_io;

mod error;
mod util;

// Include generated message types
mod generated {
    #![allow(missing_docs)]
    include!(concat!(env!("OUT_DIR"), "/generated_messages.rs"));
}

pub mod request;
pub mod response;

pub use error::*;
pub use util::crc;

pub use zerocopy;
