use core::{Color, Ray, RayIntersection, Illuminator, Intersector, LightIntersection};
use tools::{Vector3Extensions, CompareWithTolerance};


pub trait RayCaster: Send + Sync {
    fn cast_ray(&self, ray: &Ray) -> Option<Color>;
    fn cast_colored_light_ray(&self, ray: &Ray, intersection: &RayIntersection) -> Option<Color>;
    fn cast_model_ray(&self, ray: &Ray) -> Option<RayIntersection>;
}

pub trait IlluminationCaster: Send + Sync {
    fn get_illumination_at(&self, intersection: &RayIntersection) -> Vec<LightIntersection>;
}

pub trait ColorCalculator: Send + Sync {
    fn get_color(&self, itersection: &RayIntersection, ray_caster: &RayCaster, illumination_caster: &IlluminationCaster) -> Option<Color>;
}

pub struct World<IntersectorType, ColorCalculatorType, IlluminatorType> {
    intersector : IntersectorType,
    color_calculator : ColorCalculatorType,
    illuminator: IlluminatorType,
    depth_limit : i32,
}

impl<IntersectorType: Intersector + Send + Sync,
     ColorCalculatorType: ColorCalculator + Send + Sync,
     IlluminatorType : Illuminator + Send + Sync> 
     World<IntersectorType, ColorCalculatorType, IlluminatorType> {
    
    pub fn new(intersector: IntersectorType, colorcalc: ColorCalculatorType, illuminator: IlluminatorType, ray_depth_limit: i32) -> Self {
        Self {intersector: intersector,
              color_calculator: colorcalc,
              illuminator: illuminator,
              depth_limit: ray_depth_limit}
    }
}

impl<IntersectorType: Intersector + Send + Sync,
     ColorCalculatorType: ColorCalculator + Send + Sync,
     IlluminatorType : Illuminator + Send + Sync> RayCaster for World<IntersectorType, ColorCalculatorType, IlluminatorType> {
    fn cast_ray(&self, ray: &Ray) -> Option<Color> {
        if ray.get_depth_counter() <= self.depth_limit {
            match self.intersector.get_nearest_intersection(ray) {
                Some(nearest_intersection) => self.color_calculator.get_color(&nearest_intersection, self, self),
                None => None,
            }
        } else {
            None
        }
    }

    fn cast_colored_light_ray(&self, ray: &Ray, intersection: &RayIntersection) -> Option<Color> {
        let origin_to_intersection_vector = intersection.get_intersection_point() - ray.get_origin();
        
        let max_length = origin_to_intersection_vector.length();
        let mut resulting_color = Color::one();

        for intersection in self.intersector.get_intersections_reverse_ordered(ray).iter().rev() {
            if intersection.get_distance_to_intersection().greater_eq_eps(&max_length) {
                return Some(resulting_color)
            }
            
            match intersection.get_material().get_transparency_to_light() {
                None => return None,
                Some(transparency) => resulting_color *= transparency
            }
        }

        Some(resulting_color)
    }

    fn cast_model_ray(&self, ray: &Ray) -> Option<RayIntersection> {
        self.intersector.get_nearest_intersection(ray)
    }
}

impl<IntersectorType: Intersector + Send + Sync,
     ColorCalculatorType: ColorCalculator + Send + Sync,
     IlluminatorType : Illuminator + Send + Sync> IlluminationCaster for World<IntersectorType, ColorCalculatorType, IlluminatorType> {
    fn get_illumination_at(&self, intersection: &RayIntersection) -> Vec<LightIntersection> {
        self.illuminator.get_illumination_at(intersection, self)
    }
}