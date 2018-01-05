use core::Intersector;

use core::Model;
use core::RayIntersection;
use core::Ray;

use tools::CompareWithTolerance;


pub type ModelVec = Vec<Box<Model>>;

pub struct SimpleIntersector<> {
    models: ModelVec,
}

impl Intersector for SimpleIntersector {
    fn get_intersections_reverse_ordered(&self, ray: &Ray) -> Vec<RayIntersection> { //Nearest elem is last
        let mut result: Vec<RayIntersection> = self.models.iter().filter_map(|model_box| model_box.get_intersection(ray)).collect();

        result.sort_by(|lhs: &RayIntersection, rhs: &RayIntersection| {
            lhs.get_distance_to_intersection().compare_eps(&rhs.get_distance_to_intersection()).reverse()
        });

        result
    }

    fn get_nearest_intersection(&self, ray: &Ray) -> Option<RayIntersection> {
        let mut all_intersections = self.get_intersections_reverse_ordered(ray);

        match all_intersections.pop() {
            Some(intersection) => Some(intersection),
            None => None
        }
    }
}