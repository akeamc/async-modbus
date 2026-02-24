#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

// #[cfg(feature = "embedded-io")]
// pub mod embedded_io;

// mod error;
mod frame;
mod util;

mod pdu;

pub use frame::Frame;

// pub mod request;
// pub mod response;

// pub use error::*;
pub use util::crc;

// pub use zerocopy;
