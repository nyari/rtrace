use defs::{FloatType};

use core::{Ray, RayError, RayIntersection, ColorComponent};
use tools::{CompareWithTolerance};

#[derive(Debug)]
pub enum RayPropagatorError {
    RayRelated(RayError),
    NoRefraction,
    NotRefractiveMaterial
}

pub struct RayPropagator<'intersection> {
    intersection: &'intersection RayIntersection
}

impl<'intersection> RayPropagator<'intersection> {
    pub fn new(intersection: &'intersection RayIntersection) -> Self {
        Self {
            intersection: intersection
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
}