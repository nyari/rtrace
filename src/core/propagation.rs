use defs::{FloatType};

use core::{Ray, RayError, RayIntersection, ColorComponent};
use tools::{CompareWithTolerance};

#[derive(Debug)]
pub enum LightPropagatorError {
    RayRelated(RayError),
    NoRefraction,
    NotRefractiveMaterial
}

pub struct LightPropagator<'intersection> {
    intersection: &'intersection RayIntersection
}

impl<'intersection> LightPropagator<'intersection> {
    pub fn new(intersection: &'intersection RayIntersection) -> Self {
        Self {
            intersection: intersection
        }
    }

    pub fn get_mirrored_ray(&self) -> Result<Ray, LightPropagatorError> {
        let view = self.intersection.get_view_direction();
        let normal = self.intersection.get_normal_vector();

        let mirror_direction = -view + (normal * 2.0);

        Ray::continue_ray_from_intersection(self.intersection, mirror_direction).map_err(|ray_error| {
            match ray_error {
                RayError::DepthLimitReached => LightPropagatorError::RayRelated(ray_error),
                _ => panic!("LightPropagator encountered unhandleable RayError")
            }
        })
    }

    pub fn get_refracted_ray_custom_index(&self, refractive_index: FloatType) -> Result<Ray, LightPropagatorError> {
        let view = self.intersection.get_view_direction();
        let normal = self.intersection.get_normal_vector();

        let cosa = normal.dot(&view);
        let rooted = 1.0-((1.0-cosa.powi(2)) / refractive_index.powi(2));

        if rooted.greater_eq_eps(&0.0) {
            let nf = if self.intersection.was_inside() {
                refractive_index.recip()
            } else {
                refractive_index
            };

            let refraction_direction = view * (-nf.recip()) + normal * (cosa/nf - rooted.sqrt());

            Ray::continue_ray_from_intersection(self.intersection, refraction_direction).map_err(|ray_error| {
                match ray_error {
                    RayError::DepthLimitReached => LightPropagatorError::RayRelated(ray_error),
                    _ => panic!("LightPropagator encountered unhandleable RayError")
                }
            })
        } else {
            Err(LightPropagatorError::NoRefraction)
        }
    }

    pub fn get_refracted_ray(&self) -> Result<Ray, LightPropagatorError> {
        let material = self.intersection.get_material();
        if let Some(refractive_index) = material.get_average_refractive_index() {
            self.get_refracted_ray_custom_index(refractive_index)
        } else {
            Err(LightPropagatorError::NotRefractiveMaterial)
        }
    }

    pub fn get_refracted_component_ray(&self, component: ColorComponent) -> Result<Ray, LightPropagatorError> {
        let material = self.intersection.get_material();
        if let Some(refractive_index) = material.get_refractive_index_for_component(component) {
            self.get_refracted_ray_custom_index(refractive_index)
        } else {
            Err(LightPropagatorError::NotRefractiveMaterial)
        }
    }
}