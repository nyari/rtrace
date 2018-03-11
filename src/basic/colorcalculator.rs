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

pub struct GlobalIlluminationColorCalculator {
    simple_calculator: SimpleColorCalculator,
    normalwise_samples: u32,
    normalwise_max_angle: FloatType,
    rotational_samples: u32,
    depth_limit: u32
}

impl GlobalIlluminationColorCalculator {
    pub fn new(calculator: SimpleColorCalculator, normalwise_samples: u32, normalwise_max_angle: FloatType, rotational_samples: u32, depth_limit: u32) -> Self {
        Self {
            simple_calculator: calculator,
            normalwise_samples: normalwise_samples,
            normalwise_max_angle: normalwise_max_angle,
            rotational_samples: rotational_samples,
            depth_limit: depth_limit,
        }
    }

    pub fn calculate_gi(&self, intersection: &RayIntersection, ray_caster: &RayCaster) -> Option<Color> {
        let mut accumulator = Color::zero();
        let mut accumulation_count = 0;
        let propagator = RayPropagator::new(intersection);

        if let Ok(ray) = propagator.get_diffuse_direction_ray(0.0, 0.0) {
            if let Some(upward_color) = ray_caster.cast_ray(&ray.get_maximum_depth_limited(self.depth_limit)) {
                accumulator += upward_color;
                accumulation_count += 1
            }
        } else {
            return None
        }

        let mut rng = rand::thread_rng();

        for _normalwise_counter in 1..self.normalwise_samples {
            for _rotational_counter in 0..self.rotational_samples {
                let normalwise_angle = (rng.gen::<FloatType>()) * self.normalwise_max_angle;
                let rotational_angle = (rng.gen::<FloatType>()) * std::f64::consts::PI * 2.0;
                if let Some(color) = ray_caster.cast_ray(&propagator.get_diffuse_direction_ray(normalwise_angle, rotational_angle).unwrap().get_maximum_depth_limited(self.depth_limit)) {
                    accumulator += color;
                    accumulation_count += 1;
                }
            }
        }

        accumulator = accumulator.mul_scalar(&(accumulation_count as FloatType).recip());
        Some(accumulator)
    }
}

impl ColorCalculator for GlobalIlluminationColorCalculator {
    fn get_color(&self, intersection: &RayIntersection, ray_caster: &RayCaster, illumination_caster: &IlluminationCaster) -> Option<Color> {
        if intersection.get_material().get_diffuse_color().is_some() {
            if let Some(color) = self.simple_calculator.get_color(intersection, ray_caster, illumination_caster) {
                if let Some(gi_color) = self.calculate_gi(intersection, ray_caster) {
                    Some(color + gi_color)
                } else {
                    Some(color)
                }
            } else {
                if let Some(gi_color) = self.calculate_gi(intersection, ray_caster) {
                    Some(gi_color)
                } else {
                    None
                }
            }
        } else {
            self.simple_calculator.get_color(intersection, ray_caster, illumination_caster)
        }
    }
}