use core::{Color, ColorComponent, FresnelIndex, RayIntersection, LightIntersection};

use defs::FloatType;
use tools::CompareWithTolerance;

#[derive(Copy, Clone)]
struct FresnelData {
    pub n: FresnelIndex,
    pub n_inverse: FresnelIndex,
    pub n_avg: FloatType,
    pub n_imaginary: FresnelIndex,
    pub n_imaginary_inverse: FresnelIndex,
    pub n_imaginary_avg: FloatType,
    pub f0: FresnelIndex,
    pub f0_avg: FloatType,
    pub f0_inverse: FresnelIndex,
    pub f0_inverse_avg: FloatType
}

impl FresnelData {
    fn new(real: FresnelIndex, imaginary: FresnelIndex) -> Self {
        let mut f0 = ((real - FresnelIndex::one()) * (real - FresnelIndex::one())) + imaginary * imaginary;
        f0 *=  (((real + FresnelIndex::one()) * (real + FresnelIndex::one())) + imaginary * imaginary).recip();

        let mut f0_inverse = ((imaginary - FresnelIndex::one()) * (imaginary - FresnelIndex::one())) + imaginary * imaginary;
        f0_inverse *=  (((imaginary + FresnelIndex::one()) * (imaginary + FresnelIndex::one())) + imaginary * imaginary).recip(); 

        Self {  n: real,
                n_inverse: real.recip(),
                n_avg: real.intensity_avg(),
                n_imaginary: imaginary,
                n_imaginary_inverse: imaginary.recip(),
                n_imaginary_avg: imaginary.intensity_avg(),
                f0: f0,
                f0_avg: f0.intensity_avg(),
                f0_inverse: f0_inverse,
                f0_inverse_avg: f0_inverse.intensity_avg()
        }
    }

    pub fn get_fresnel_reflect(&self, intersection: &RayIntersection) -> Option<Color> {
        let view_and_normal_angle_cosine = intersection.get_view_direction().dot(intersection.get_normal_vector());

        if view_and_normal_angle_cosine.greater_eq_eps(&0.0) {
            let f = if !intersection.was_inside() {
                self.f0
            } else {
                self.f0_inverse
            };

            let f1 = (Color::one()-f) * Color::one().mul_scalar(&(1.0 - view_and_normal_angle_cosine).powi(5));
            
            Some(f + f1)
        } else {
            None
        }
    }

    pub fn get_fresnel_refract(&self, intersection: &RayIntersection) -> Option<Color> {
        if let Some(color) = self.get_fresnel_reflect(intersection) {
            Some(Color::one() - color)
        } else {
            None
        }
    }
}


#[derive(Copy, Clone)]
pub struct Material {
    ambient: Option<Color>,
    diffuse: Option<Color>,
    specular: Option<(Color, FloatType)>,
    fresnel: Option<FresnelData>,
    reflective: bool,
    refractive: bool,
}

impl Material {
    pub fn new_useless() -> Self {
        Self { ambient: None,
               diffuse: None,
               specular: None,
               fresnel: None,
               reflective: false,
               refractive: false
        }
    }

    pub fn new_diffuse(diffuse: Color, ambient: Option<Color>) -> Self {
        Self { diffuse: Some(diffuse),
               ambient: ambient,
               specular: None,
               fresnel: None,
               reflective: false,
               refractive: false}
    }

    pub fn new_shiny(diffuse: Color, specular: (Color, FloatType), ambient: Option<Color>) -> Self {
        Self { diffuse: Some(diffuse),
               ambient: ambient,
               specular: Some(specular),
               fresnel: None,
               reflective: false,
               refractive: false}
    }

    pub fn new_reflective(fresnel_real: FresnelIndex, fresnel_imagninary: FresnelIndex, diffuse: Option<Color>, specular: Option<(Color, FloatType)>, ambient: Option<Color>) -> Self {
        Self { diffuse: diffuse,
               ambient: ambient,
               specular: specular,
               fresnel: Some(FresnelData::new(fresnel_real, fresnel_imagninary)),
               reflective: true,
               refractive: false}
    }

    pub fn new_refractive(fresnel_real: FresnelIndex, fresnel_imagninary: FresnelIndex, diffuse: Option<Color>, specular: Option<(Color, FloatType)>, ambient: Option<Color>) -> Self {
        Self { diffuse: diffuse,
               ambient: ambient,
               specular: specular,
               fresnel: Some(FresnelData::new(fresnel_real, fresnel_imagninary)),
               reflective: false,
               refractive: true}
    }

    pub fn new_reflective_and_refractive(fresnel_real: FresnelIndex, fresnel_imagninary: FresnelIndex, diffuse: Option<Color>, specular: Option<(Color, FloatType)>, ambient: Option<Color>) -> Self {
        Self { diffuse: diffuse,
               ambient: ambient,
               specular: specular,
               fresnel: Some(FresnelData::new(fresnel_real, fresnel_imagninary)),
               reflective: true,
               refractive: true}
    }

    pub fn get_ambient_color(&self) -> Option<&Color> {
        self.ambient.as_ref()
    }

    pub fn get_diffuse_color(&self) -> Option<&Color> {
        self.diffuse.as_ref()
    }

    pub fn get_specular_color(&self) -> Option<&(Color, FloatType)> {
        self.specular.as_ref()
    }

    pub fn is_opaque(&self) -> bool {
        !self.refractive
    }

    pub fn is_transparent(&self) -> bool {
        !self.is_opaque()
    }

    pub fn is_reflective(&self) -> bool {
        self.reflective
    }

    pub fn is_refractive(&self) -> bool {
        self.refractive
    }

    pub fn get_transparency_to_light(&self) -> Option<Color> {
        if self.is_transparent() {
            match self.diffuse {
                None => Some(Color::one()),
                Some(color) => Some(Color::one() - color)
            }
        } else {
            None
        }
    }

    pub fn get_average_refractive_index(&self) -> Option<FloatType> {
        self.fresnel.and_then(|fresnel_data| {
            Some(fresnel_data.n_avg)
        })
    }

    pub fn get_refractive_index_for_component(&self, component: ColorComponent) -> Option<FloatType> {
        self.fresnel.and_then(|fresnel_data| {
            Some(fresnel_data.n.get_component(component))
        })
    }

    fn get_fresnel_data(&self) -> Option<&FresnelData> {
        self.fresnel.as_ref()
    }

    pub fn get_diffuse_illumination(ray_intersection: &RayIntersection, light_intersection: &LightIntersection) -> Option<Color> {
        let material = ray_intersection.get_material();

        material.get_diffuse_color().and_then(|color| {
            let surface_normal = ray_intersection.get_normal_vector();
            let light_direction = light_intersection.get_light_direction();
            let cosln = light_direction.dot(surface_normal).max(0.0);
            let illumination = light_intersection.get_illumination();
            Some ((*color * *illumination).mul_scalar(&cosln))
        })
    }

    pub fn get_specular_illumination(ray_intersection: &RayIntersection, light_intersection: &LightIntersection) -> Option<Color> {
        let material = ray_intersection.get_material();

        material.get_specular_color().and_then(|color_shiny| {
            let illumination = light_intersection.get_illumination();
            let view_direction = ray_intersection.get_view_direction();
            let surface_normal = ray_intersection.get_normal_vector();
            let (color, shininess) = *color_shiny;
            let light_direction = light_intersection.get_light_direction();
            let half_direction = (view_direction + light_direction).normalize();
            let coshn = half_direction.dot(surface_normal).max(0.0).powf(shininess);
            Some((color * *illumination).mul_scalar(&coshn))
        })
    }

    pub fn get_fresnel_reflection(ray_intersection: &RayIntersection) -> Option<Color> {
        let material = ray_intersection.get_material();
        material.get_fresnel_data().and_then(|fresnel_data| {
            fresnel_data.get_fresnel_reflect(ray_intersection)
        })
    }

    pub fn get_fresnel_refraction(ray_intersection: &RayIntersection) -> Option<Color> {
        let material = ray_intersection.get_material();
        material.get_fresnel_data().and_then(|fresnel_data| {
            fresnel_data.get_fresnel_refract(ray_intersection)
        })
    }
}