use defs::{FloatType, Vector3};

use core::{Ray, RayError, RayIntersection, ColorComponent};
use tools::{CompareWithTolerance};

use na;
use na::{Rotation3, Unit};


#[derive(Debug)]
pub enum RayPropagatorError {
    RayRelated(RayError),
    NoRefraction,
    NotRefractiveMaterial
}

pub struct RayPropagator<'intersection> {
    intersection: &'intersection RayIntersection,
    view_axis: Unit<Vector3>
}

impl<'intersection> RayPropagator<'intersection> {
    pub fn new(intersection: &'intersection RayIntersection) -> Self {
        let normal = intersection.get_normal_vector();
        let view = intersection.get_view_direction();
        let view_axis = normal.cross(&view);
        Self {
            intersection: intersection,
            view_axis: Unit::new_normalize(view_axis)
        }
    }

    pub fn get_mirrored_ray(&self) -> Result<Ray, RayPropagatorError> {
        let view = self.intersection.get_view_direction();
        let normal = self.intersection.get_normal_vector();

        let mirror_direction = -view + (normal * 2.0);

        Ray::continue_ray_from_intersection(self.intersection, mirror_direction).map_err(|ray_error| {
            match ray_error {
                RayError::DepthLimitReached => RayPropagatorError::RayRelated(ray_error),
                _ => panic!("RayPropagator encountered unhandleable RayError")
            }
        })
    }

    pub fn get_refracted_ray_custom_index_ratio(&self, refractive_index_ratio: FloatType) -> Result<Ray, RayPropagatorError> {
        let view = self.intersection.get_view_direction();
        let normal = self.intersection.get_normal_vector();

        let cosa = normal.dot(&view);
        let rooted = 1.0-((1.0-cosa.powi(2)) / refractive_index_ratio.powi(2));

        if rooted.greater_eq_eps(&0.0) {
            let refraction_direction = view * (-refractive_index_ratio.recip()) + normal * (cosa/refractive_index_ratio - rooted.sqrt());

            Ray::continue_ray_from_intersection_into_medium(self.intersection, refraction_direction).map_err(|ray_error| {
                match ray_error {
                    RayError::DepthLimitReached => RayPropagatorError::RayRelated(ray_error),
                    _ => panic!("RayPropagator encountered unhandleable RayError")
                }
            })
        } else {
            Err(RayPropagatorError::NoRefraction)
        }
    }

    pub fn get_transition_refraction_index(&self) -> Option<FloatType> {
        let target_material = self.intersection.get_material();
        if target_material.is_refractive() {
            if let Some(source_material) = self.intersection.get_ray_medium() {
                let source_material_index = if source_material.is_refractive() { source_material.get_average_refractive_index().unwrap() } else { 1.0 };
                let target_material_index = target_material.get_average_refractive_index().unwrap();
                Some(target_material_index / source_material_index)
            } else {
                target_material.get_average_refractive_index()
            }
        } else {
            None
        }
    }

    pub fn get_transition_refraction_index_component(&self, component: ColorComponent) -> Option<FloatType> {
        let target_material = self.intersection.get_material();
        if target_material.is_refractive() {
            if let Some(source_material) = self.intersection.get_ray_medium() {
                let source_material_index = if source_material.is_refractive() { source_material.get_refractive_index_for_component(component).unwrap() } else { 1.0 };
                let target_material_index = target_material.get_refractive_index_for_component(component).unwrap();
                Some(target_material_index / source_material_index)
            } else {
                target_material.get_refractive_index_for_component(component)
            }
        } else {
            None
        }
    }

    pub fn get_refracted_ray(&self) -> Result<Ray, RayPropagatorError> {
        if let Some(refractive_index) = self.get_transition_refraction_index() {
            self.get_refracted_ray_custom_index_ratio(refractive_index)
        } else {
            Err(RayPropagatorError::NotRefractiveMaterial)
        }
    }

    pub fn get_refracted_component_ray(&self, component: ColorComponent) -> Result<Ray, RayPropagatorError> {
        if let Some(refractive_index) = self.get_transition_refraction_index_component(component) {
            self.get_refracted_ray_custom_index_ratio(refractive_index)
        } else {
            Err(RayPropagatorError::NotRefractiveMaterial)
        }
    }

    pub fn get_diffuse_direction_vector(&self, angle_to_normal: FloatType, angle_to_view_direction: FloatType) -> Vector3 {
        let normal = self.intersection.get_normal_vector();
        let view = self.intersection.get_view_direction();
        let angle_to_rotate_view = na::angle(normal, &view) - angle_to_normal;

        let rotate_to_normal = Rotation3::from_axis_angle(&self.view_axis, angle_to_rotate_view);
        let rotate_around_normal = Rotation3::from_axis_angle(&Unit::new_unchecked(*normal), angle_to_view_direction);

        rotate_around_normal * rotate_to_normal * view
    }

    pub fn get_diffuse_direction_ray(&self, pitch: FloatType, yaw: FloatType) -> Result<Ray, RayPropagatorError> {
        Ray::continue_ray_from_intersection(self.intersection, self.get_diffuse_direction_vector(pitch, yaw)).map_err(|ray_error| {
            match ray_error {
                RayError::DepthLimitReached => RayPropagatorError::RayRelated(ray_error),
                _ => panic!("RayPropagator encountered unhandleable RayError")
            }
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use defs::{Point3};
    use core::{Material};
    use na::{Unit};
    use std::f64::consts::{PI};

    #[test]
    fn diffuse_direction_vector() {
        let ray = Ray::new(Point3::new(1.0, 0.0, 1.0), Vector3::new(-1.0, 0.0, -1.0));
        let intersection = RayIntersection::new(Vector3::new(0.0, 0.0, 1.0), Point3::new(0.0, 0.0, 0.0),
                                                &ray, Material::new_useless(), false).unwrap();

        let propagator = RayPropagator::new(&intersection);

        assert_relative_eq!(propagator.get_diffuse_direction_vector(PI/4.0, 0.0),       Unit::new_normalize(Vector3::new(1.0, 0.0, 1.0)).unwrap());
        assert_relative_eq!(propagator.get_diffuse_direction_vector(PI/8.0, 0.0),       Unit::new_normalize(Vector3::new((3.0*PI/8.0).tan(), 0.0, 1.0)).unwrap());
        assert_relative_eq!(propagator.get_diffuse_direction_vector(PI/8.0, PI/2.0),    Unit::new_normalize(Vector3::new(0.0, (3.0*PI/8.0).tan(), 1.0)).unwrap());
        assert_relative_eq!(propagator.get_diffuse_direction_vector(PI/8.0, PI),        Unit::new_normalize(Vector3::new(-((3.0*PI/8.0).tan()), 0.0, 1.0)).unwrap());
    }

}
