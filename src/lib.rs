extern crate teletel_derive;

#[macro_use]
mod macros;

mod error;
mod wrapper;

pub mod minitel;
pub mod protocol;
pub mod receiver;

pub use error::Error;
pub use minitel::{BaudRate, Minitel};

pub use macros::*;
pub use wrapper::*;
