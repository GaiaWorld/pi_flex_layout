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
mod grow_shrink;
mod layout;
mod layout_context;
mod layout_tree;
mod node_state;
mod number;
pub mod style;
mod traits;

pub mod prelude {
    pub use crate::calc::{CharNode, INode};
    pub use crate::geometry::*;
    pub use crate::layout::*;
    pub use crate::layout_context::*;
    pub use crate::layout_tree::*;
    pub use crate::node_state::NodeState;
    pub use crate::number::*;
    pub use crate::style::*;
    pub use crate::traits::*;
}
