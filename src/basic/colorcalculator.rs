use core::{RayCaster, RayIntersection, Color, ColorCalculator, Material,
          IlluminationCaster, LightIntersection, RayPropagator, RayPropagatorError};

use defs::{FloatType};
use std;

pub struct SimpleColorCalculator {
    
}

impl SimpleColorCalculator {
    pub fn new() -> Self {
        Self {}
    }

    fn get_ambient_color(&self, intersection: &RayIntersection) -> Color {
        let material = intersection.get_material();
        *material.get_ambient_color().unwrap_or(&Color::zero())
    }

    fn get_local_color(&self, intersection: &RayIntersection, illuminations: &Vec<LightIntersection>) -> Color { 
        illuminations.iter().fold(Color::zero(), |acc, light_intersection|{
            let diffuse_color = Material::get_diffuse_illumination(intersection, light_intersection);
            let specular_color = Material::get_specular_illumination(intersection, light_intersection);
            
            acc + specular_color.unwrap_or(Color::zero()) + diffuse_color.unwrap_or(Color::zero())
        }).normalized()
    }

    fn get_reflected_color(&self, intersection: &RayIntersection, ray_caster: &RayCaster) -> Color {
        let material = intersection.get_material();
        if material.is_reflective() {
            let propagator = RayPropagator::new(intersection);
            match propagator.get_mirrored_ray() {
                Ok(mirror_ray) => {
                    if let Some(color) = ray_caster.cast_ray(&mirror_ray) {
                        if let Some(fresnel_color) = Material::get_fresnel_reflection(intersection) {
                            fresnel_color * color
                        } else {
                            Color::zero()
                        }
                    } else {
                        Color::zero()
                    }
                },
                Err(RayPropagatorError::RayRelated(_)) => Color::zero(),
                Err(_) => panic!("Unhandled RayPropagator error!")
            }
        } else {
            Color::zero()
        }
    }

    fn get_refracted_color(&self, intersection: &RayIntersection, ray_caster: &RayCaster) -> Color {
        let material = intersection.get_material();
        if material.is_refractive() {
            let propagator = RayPropagator::new(intersection);
            match propagator.get_refracted_ray() {
                Ok(refract_ray) => {
                    if let Some(color) = ray_caster.cast_ray(&refract_ray) {
                        if let Some(fresnel_color) = Material::get_fresnel_refraction(intersection) {
                            fresnel_color * color
                        } else {
                            Color::zero()
                        }
                    } else {
                        Color::zero()
                    }
                }
                Err(RayPropagatorError::RayRelated(_)) => Color::zero(),
                Err(_) => panic!("Unhandled RayPropagator error!")
            }
        } else {
            Color::zero()
        }
    }
}

impl ColorCalculator for SimpleColorCalculator {
    fn get_color(&self, intersection: &RayIntersection, ray_caster: &RayCaster, illumination_caster: &IlluminationCaster) -> Option<Color> {
        let illuminations = illumination_caster.get_illumination_at(intersection);

        let result =    self.get_ambient_color(intersection) +
                        self.get_local_color(intersection, &illuminations) +
                        self.get_reflected_color(intersection, ray_caster) +
                        self.get_refracted_color(intersection, ray_caster);

        Some (result)
    }
}

