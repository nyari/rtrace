//! This module contains type definitions and constants that are used and should be used
//! within this library.  
//!
//! The purpose of these definitions is to shorten [nalgebra](http://nalgebra.org/) types,
//! , limit the scope of literals by defining constants here, and provide consistent
//! integer and floating-point precision in the entire library.

/// General floating-point type that should be used within the library everywhere floating point value is needed  
///
/// Changing it to any other floating point type should still compile and work
/// but because its precision output can change.
pub type FloatType = f64;

/// General signed integer type that should be used within the library everywhere integer value is needed  
/// 
/// Changing it to any other signed integer type that cointain the output image resolution should work
/// and should not change output
pub type IntType = i32;

/// Alias for a 4 column row vector
pub type VectorRow4 = super::na::core::Matrix1x4<FloatType>;
/// Alias for a 4 row column vector
pub type VectorColumn4 = super::na::core::Vector4<FloatType>;

/// Alias for a 4 by 4 square matrix
pub type Matrix4 = super::na::core::Matrix4<FloatType>;
/// Alias for a 3 by 3 square matrix
pub type Matrix3 = super::na::core::Matrix3<FloatType>;

/// Alias for a 3 dimensional point
pub type Point3 = super::na::Point3<FloatType>;
/// Alias for a 3 dimensional vector
pub type Vector3 = super::na::core::Vector3<FloatType>;

/// Alias for a 2 dimensional point
pub type Point2 = super::na::Point2<FloatType>;
/// Alias for a 2 dimensional vector
pub type Vector2 = super::na::Vector2<FloatType>;

/// Alias for a 2 dimensional integer point
pub type Point2Int = super::na::Point2<IntType>;
/// Alias for a 2 dimensional integer vector
pub type Vector2Int = super::na::Vector2<IntType>;


/// Definition of the ULPS tolerance when comparing FloatType-s. See [tools](../tools/index.html) module
pub static FLOAT_ULPS_TOLERANCE: i64 = 65536;