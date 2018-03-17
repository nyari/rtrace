use core::*;

pub mod intersector;
pub mod illuminator;
pub mod colorcalculator;
pub mod lightsource;
pub mod model;
pub mod postprocessing;

pub use self::intersector::*;
pub use self::illuminator::*;
pub use self::colorcalculator::*;
pub use self::postprocessing::*;


pub type SimpleWorld = World<SimpleIntersector, SimpleColorCalculator, SimpleIlluminator>;