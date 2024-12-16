// #![feature(assoc_int_consts)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(alloc))]
#![allow(invalid_reference_casting)]

extern crate paste;

#[macro_use]
extern crate pi_print_any;

#[macro_use]
extern crate serde;

mod calc;
mod geometry;
mod layout_tree;
mod number;
pub mod style;
mod layout;
mod traits;
mod layout_context;
mod node_state;

pub mod prelude {
    pub use crate::traits::*;
    pub use crate::layout_context::*;
    pub use crate::geometry::*;
    pub use crate::layout_tree::*;
    pub use crate::number::*;
    pub use crate::style::*;
    pub use crate::layout::*;
}
