#[macro_use]
extern crate teletel_derive;

#[macro_use]
mod macros;

mod error;
mod wrapper;

pub mod minitel;
pub mod protocol; //confusing with wrapper::proto, todo fix
pub mod terminal;

pub use error::Error;

pub use macros::*;
pub use wrapper::*;
