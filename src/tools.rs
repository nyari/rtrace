//!The module defines and implements traits make are used often and therefore
//!simplify other parts of the library code.

use std::cmp::Ordering;
use defs::{Vector3, FloatType, IntType, FLOAT_ULPS_TOLERANCE};
use ::na;
use ::numt::Num;
use ::numt::float::Float;
use ::flcmp::ApproxOrdUlps;

/// Trait to help with comparing floating point values with some tolerance
pub trait CompareWithTolerance<T: Float> {
    /// Compare values and return an Ordering
    fn compare_eps(&self, rhs: &T) -> Ordering;
    /// Compare values and check wether self is less than right-hand side `(rhs)` 
    fn less_eps(&self, rhs: &T) -> bool;
    /// Compare values and check wether self is less or equal to right-hand side `(rhs)`
    fn less_eq_eps(&self, rhs: &T) -> bool;
    /// Compare values and check wether self is equal to right-hand side `(rhs)`
    fn equal_eps(&self, rhs: &T) -> bool;
    /// Compare values and check wether self is qreater than right-hand side `(rhs)`
    fn greater_eps(&self, rhs: &T) -> bool;
    /// Compare values and check wether self is qreater or equal to right-hand side `(rhs)`
    fn greater_eq_eps(&self, rhs: &T) -> bool;
    /// Compare values and check wether self is near to zero (same as `self.equal_eps(&0.0)`)
    fn near_zero_eps(&self) -> bool;
}

/// Implementation of `CompareWithTolerance` trait for `FloatType` with the use of the `float_cmp` crate
/// This also uses the `FLOAT_ULPS_TOLERANCE` defined in the `defs` module
impl CompareWithTolerance<FloatType> for FloatType {
    fn compare_eps(&self, rhs: &FloatType) -> Ordering {
        self.approx_cmp_ulps(&rhs, FLOAT_ULPS_TOLERANCE)
    }

    fn less_eps(&self, rhs: &FloatType) -> bool {
        self.approx_cmp_ulps(&rhs, FLOAT_ULPS_TOLERANCE) == Ordering::Less
    }

    fn less_eq_eps(&self, rhs: &FloatType) -> bool {
        match self.approx_cmp_ulps(&rhs, FLOAT_ULPS_TOLERANCE) {
            Ordering::Less => true,
            Ordering::Equal => true,
            _ => false,
        }
    }

    fn equal_eps(&self, rhs: &FloatType) -> bool {
        self.approx_cmp_ulps(&rhs, FLOAT_ULPS_TOLERANCE) == Ordering::Equal
    }

    fn greater_eps(&self, rhs: &FloatType) -> bool {
        self.approx_cmp_ulps(&rhs, FLOAT_ULPS_TOLERANCE) == Ordering::Greater
    }

    fn greater_eq_eps(&self, rhs: &FloatType) -> bool {
        match self.approx_cmp_ulps(&rhs, FLOAT_ULPS_TOLERANCE) {
            Ordering::Greater => true,
            Ordering::Equal => true,
            _ => false,
        }
    }

    fn near_zero_eps(&self) -> bool {
        self.approx_cmp_ulps(&0.0, FLOAT_ULPS_TOLERANCE) == Ordering::Equal
    }
}

/// This trait provides additional funtionality to `nalgebra::Vector3` that weren't easily accessible
pub trait Vector3Extensions {
    /// Check if `&self` Vector3 points in the same direction as `rhs`
    fn same_direction_as(&self, rhs: &Vector3) -> bool;
    /// Get the length of `&self` Vector3
    fn length(&self) -> FloatType;
}

/// This implements the `Vector3Extensions` for Vector3
impl Vector3Extensions for Vector3 {
    fn same_direction_as(&self, rhs: &Vector3) -> bool {
        let closed_angle = na::angle(self, &rhs);
        closed_angle.near_zero_eps()
    }

    fn length(&self) -> FloatType {
        let mut result: FloatType = 0.0;

        for item in self.iter() {
            result += item.powi(2);
        }

        result.sqrt()
    }
}

/// Trait that is to be implemented by the result of comparation operations
pub trait OrderingQuery {
    /// Check if the result is less
    fn is_less(self) -> bool;
    /// Check if the result is equal 
    fn is_equal(self) -> bool;
    /// Check if the result is greater
    fn is_greater(self) -> bool;
}

/// Implement `OrderingQuery` for `Ordering`
impl OrderingQuery for Ordering {
    fn is_less(self) -> bool {
        match self {
            Ordering::Less => true,
            _ => false
        }
    }

    fn is_equal(self) -> bool {
        match self {
            Ordering::Equal => true,
            _ => false
        }
    }

    fn is_greater(self) -> bool {
        match self {
            Ordering::Greater => true,
            _ => false
        }
    }
}

/// Trait to check if a numeric value is between other two values (including the ends)
pub trait Between<T: Num> {
    /// Check if `self` is between `less` and `more` including both values in the check 
    fn between(&self, less: &T, more: &T) -> bool;
}

/// Implementation of the `Between` trait for `FloatType`. Uses `CompareWithTolerance` trait for comparisons
impl Between<FloatType> for FloatType {
    fn between(&self, less: &FloatType, more: &FloatType) -> bool {
        less.greater_eq_eps(&self) && more.less_eq_eps(&self)
    }
}

/// Implementation of the `Between` trait fot `IntType`.
impl Between<IntType> for IntType {
    fn between(&self, less: &IntType, more: &IntType) -> bool {
        less <= self && self <= more
    }
}