// #![feature(assoc_int_consts)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(alloc))]
#![allow(invalid_reference_casting)]
#![feature(diagnostic_namespace)]

extern crate paste;

#[macro_use]
extern crate pi_print_any;

#[macro_use]
extern crate serde;

mod calc;
// mod grow_shrink;
mod geometry;
mod layout_tree;
mod number;
pub mod style;
mod tree;

pub mod prelude {
    pub use crate::calc::*;
    pub use crate::geometry::*;
    pub use crate::layout_tree::*;
    pub use crate::number::*;
    pub use crate::style::*;
    pub use crate::tree::*;
}
