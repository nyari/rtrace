#![warn(missing_docs)]
//! rtrace is a simple recursive ray tracer library in Rust that was mainly written to practice the language.  
//!
//! Even so it is developed with extensibility and multi-threaded operation in mind.
//! It also contains a GlobalIllumination pixel shader and a Median Filter for clearing up graininess in 
//! resulting images to illustrate this.

#[cfg(test)]

#[macro_use]
extern crate approx; // For the macro relative_eq!
extern crate nalgebra as na;
extern crate float_cmp as flcmp;
extern crate num;
extern crate num_traits as numt;
extern crate uuid;
extern crate rand;

pub mod defs;
pub mod tools;
pub mod core;
pub mod basic;
