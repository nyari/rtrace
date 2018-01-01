use core::*;
use core::color::Color;

pub struct SimpleIlluminator {
    lights : LightSourceVec
}

impl Illuminator for SimpleIlluminator {
    fn new(lights: LightSourceVec) -> Self {
        Self {lights : lights}
    }

    fn get_lights(&self) -> &LightSourceVec {
        &self.lights
    }

    fn get_lights_mut(&mut self) -> &mut LightSourceVec{
        &mut self.lights
    }

    fn get_illumination_at(&self, intersection: &RayIntersection, illumination_caster: &IlluminationCaster) -> Option<Color> {
        let result = self.lights.iter().fold(None, |acc, light| {
            match light.get_ray_to_intersection(intersection) {
                None => acc,
                Some(ray) => {
                    match illumination_caster.cast_light_ray(&ray, intersection) {
                        None => acc,
                        Some(illumintaion_shadowing) => {
                            match light.get_illumination_at(intersection) {
                                None => acc,
                                Some(illumination) => {
                                    match acc {
                                        None => Some(illumination * illumintaion_shadowing),
                                        Some(acc_illumination) => Some(acc_illumination + illumination * illumintaion_shadowing)
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        match result {
            Some(value) => Some(value.normalized()),
            None => None,
        }
    }
}