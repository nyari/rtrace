#[cfg(test)]

#[macro_use]
extern crate approx; // For the macro relative_eq!
extern crate nalgebra as na;

extern crate num;

pub mod defs;
pub mod tools;
pub mod core;
pub mod basic;


mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
