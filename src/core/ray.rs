use defs::{Vector3, Point3, FloatType, Matrix4};
use std::collections::{VecDeque};
use core::{RayIntersection, Material};
use tools::Vector3Extensions;
use na::{Unit};


#[derive(Debug)]
pub enum RayError {
    DepthLimitReached,
    InvalidContinuationDirection,
}


#[derive(Clone, Copy, Debug)]
struct RayState {
    distance_to_origin : FloatType,
    depth_counter : i32,
    depth_limit: Option<u32>,
}

impl RayState {
    pub fn get_continuation(input: &Self, distance_to_intersection: FloatType) -> Result<Self, RayError> {
        match input.depth_limit {
            Some(depth_limit) => {
                if depth_limit >= 1 {
                    Ok (Self {  distance_to_origin: input.distance_to_origin + distance_to_intersection,
                                depth_counter: input.depth_counter + 1,
                                depth_limit: Some(depth_limit - 1),
                                ..*input
                    })
                } else {
                    Err(RayError::DepthLimitReached)
                }
            },
            None => {
                    Ok (Self {  distance_to_origin: input.distance_to_origin + distance_to_intersection,
                                depth_counter: input.depth_counter + 1,
                                ..*input
                    })
            }
        }
    }

    pub fn get_maximum_depth_limited(&self, maximum_depth_limit: u32) -> Self {
        Self {
            depth_limit: Some(maximum_depth_limit.min(if let Some(depth_limit) = self.depth_limit {depth_limit} else {maximum_depth_limit})),
            ..*self
        }
    }

    pub fn get_distance_to_origin(&self) -> FloatType {
        self.distance_to_origin
    }

    pub fn get_depth_counter(&self) -> i32 {
        self.depth_counter
    }
}

#[derive(Debug, Clone)]
pub struct Ray {
    direction : Unit<Vector3>,
    origin : Point3,
    state : RayState,
    mediums : Option<VecDeque<Material>>, 
}

impl Ray {
    pub fn new(origin: Point3, dir: Vector3) -> Self {
        Self    { direction: Unit::new_normalize(dir),
                  origin: origin,
                  mediums: None,
                  state: RayState { distance_to_origin: 0.0,
                                    depth_counter: 0,
                                    depth_limit: None }
        }
    }

    pub fn new_depth_limited(origin: Point3, dir: Vector3, depth_limit: u32) -> Self {
        Self    { direction: Unit::new_normalize(dir),
                  origin: origin,
                  mediums: None,
                  state: RayState { distance_to_origin: 0.0,
                                    depth_counter: 0,
                                    depth_limit: Some(depth_limit) }
        }
    }

    pub fn new_single_shot(origin: Point3, dir: Vector3) -> Self {
        Self::new_depth_limited(origin, dir, 1)
    }

    pub fn set_maximum_depth_limited(&self, maximum_depth_limit: u32) -> Self {
        Self {
            state: self.state.get_maximum_depth_limited(maximum_depth_limit),
            ..self.clone()
        }
    }

    pub fn set_maximum_depth_limited_mut(&mut self, maximum_depth_limit: u32){
        self.state = self.state.get_maximum_depth_limited(maximum_depth_limit);
    }

    fn push_medium(&mut self, material: Material) {
        if let Some(ref mut mediums) = self.mediums {
            mediums.push_back(material);
        } else {
            self.mediums = Some(VecDeque::new());
            self.mediums.as_mut().unwrap().push_back(material);
        }
    }

    fn pop_medium(&mut self) {
        if self.mediums.is_some() {
            if let Some(ref mut mediums) = self.mediums {
                mediums.pop_back();
            }
            if self.mediums.as_ref().unwrap().is_empty() {
                self.mediums = None;
            }
        }
    }

    pub fn continue_ray_from_intersection_into_medium(intersection: &RayIntersection, direction: Vector3) -> Result<Self, RayError> {
        let original_ray = intersection.get_intersector_ray();

        match RayState::get_continuation(original_ray.get_state(), intersection.get_distance_to_intersection()) {
            Ok (continued_state) => {
                let mut result = Self {  direction: Unit::new_normalize(direction),
                                         origin: *intersection.get_intersection_point(),
                                         mediums: original_ray.mediums.clone(),
                                         state: continued_state};
                if intersection.was_inside() {
                    result.pop_medium();
                } else {
                    result.push_medium(*intersection.get_material());
                }

                Ok(result)
            },
            Err(e) => Err(e)
        }
    }

    pub fn continue_ray_from_intersection(intersection: &RayIntersection, direction: Vector3) -> Result<Self, RayError> {
        match RayState::get_continuation(intersection.get_intersector_ray().get_state(), intersection.get_distance_to_intersection()) {
            Ok (continued_state) => {
                Ok (Self {  direction: Unit::new_normalize(direction),
                            origin: *intersection.get_intersection_point(),
                            state: continued_state,
                            ..intersection.get_intersector_ray().clone()})
            },
            Err(e) => Err(e)
        }
    }

    pub fn continue_ray_from_previous(previous_ray: &Ray, origin: Point3, direction: Vector3) -> Result<Self, RayError> {
        let calculated_direction = origin - previous_ray.get_origin();

        match RayState::get_continuation(&previous_ray.state, calculated_direction.length()) {
            Ok (continued_state) => {
                Ok (Self {  direction: Unit::new_normalize(direction),
                            origin: origin,
                            state: continued_state,
                            ..previous_ray.clone()})
            },
            Err(e) => Err(e)
        }
    }

    fn get_state(&self) -> &RayState {
        &self.state
    }

    pub fn new_reversed_ray(ray: &Ray) -> Self {
        Self    { direction: (-ray.direction),
                  ..ray.clone() }
    }

    pub fn get_transformed(&self, transformation_matrix: &Matrix4) -> Self {
        let origin = self.origin.to_homogeneous();
        let direction = self.direction.to_homogeneous();

        Self    { origin: Point3::from_homogeneous(transformation_matrix * origin).expect("Unhomogeneous transformed point"),
                  direction: Unit::new_normalize(Vector3::from_homogeneous(transformation_matrix * direction).expect("Unhomogeneous transformed vector")),
                  ..self.clone()
        }
    }

    pub fn get_origin(&self) -> &Point3 {
        &self.origin
    }

    pub fn get_direction(&self) -> &Vector3 {
        &self.direction.as_ref()
    }

    pub fn get_distance_to_origin(&self) -> FloatType {
        self.state.get_distance_to_origin()
    }

    pub fn get_depth_counter(&self) -> i32 {
        self.state.get_depth_counter()
    }

    pub fn get_medium(&self) -> Option<Material> {
        if let Some(ref mediums) = self.mediums {
            if let Some(last_item) = mediums.back() {
                Some(last_item.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use core::{RayIntersection, Material};

    #[test]
    fn test_ray_new1_assignment() {
        let test_ray = Ray::new(Point3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 0.0));

        assert_eq!(test_ray.get_distance_to_origin(), 0.0);
        assert_eq!(test_ray.get_depth_counter(), 0);
    }

    #[test]
    fn test_continue_ray_from_intersection() {
        let initial_ray = Ray::new(Point3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 0.0));
        let intersection = RayIntersection::new(Vector3::new(0.0, 0.0, 1.0),
                                                Point3::new(1.0, 0.0, 0.0),
                                                &initial_ray,
                                                Material::new_useless(),
                                                false).unwrap();
        let continued_ray = Ray::continue_ray_from_intersection(&intersection, Vector3::new(0.0, 1.0, 0.0)).unwrap();

        assert_eq!(continued_ray.get_depth_counter(), 1);
        assert_eq!(continued_ray.get_distance_to_origin(), 1.0);
    }

    #[test]
    fn test_depth_limits() {
        let initial_ray = Ray::new_depth_limited(Point3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 0.0), 1);
        let continue_1 = Ray::continue_ray_from_previous(&initial_ray, Point3::new(1.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 0.0)).expect("Ray should still continue");
        let continue_2 = Ray::continue_ray_from_previous(&continue_1, Point3::new(2.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 0.0));
        if !continue_2.is_err() {
            panic!("Ray should not continue further");
        }
    }
}
