#![cfg_attr(not(feature = "std"), no_std)]
// #![warn(missing_docs)]
#![doc = include_str!("../README.md")]

#[cfg(feature = "embedded-io")]
pub mod client;
mod frame;
pub mod pdu;
mod util;

pub use ::zerocopy;
pub use frame::*;
pub use pdu::Pdu;
pub use util::crc;
