use defs::{FloatType, Point3, Vector3, Matrix4};
use core::{Ray, Material};
use tools::{CompareWithTolerance};
use na::{Unit};
use na;
use uuid::{Uuid};

static MIMIMUM_INTERSECTION_DISTANCE: FloatType = 0.000000001;

#[derive(Debug)]
pub enum RayIntersectionError {
    NoRayTravelDistance,
    NoModelIdentifierPresent
}

#[derive(Clone)]
pub struct RayIntersection {
    normal : Unit<Vector3>,
    point : Point3,
    material_at_intersection : Material,
    distance_to_intersection : FloatType,
    was_inside : bool,
    ray : Ray,
    model_identifier : Option<Uuid>,
}

impl RayIntersection {
    pub fn new(normal: Vector3, point: Point3, ray: &Ray, material: Material, was_inside: bool) -> Result<Self, RayIntersectionError> {
        let distance_to_intersection = na::distance(ray.get_origin(), &point);
        if distance_to_intersection.greater_eq_eps(&MIMIMUM_INTERSECTION_DISTANCE) {
            Ok (Self {  normal:  Unit::new_normalize(normal), 
                        point: point, 
                        ray: ray.clone(),
                        material_at_intersection: material,
                        distance_to_intersection: distance_to_intersection,
                        was_inside: was_inside,
                        model_identifier: None
                    })
        } else {
            Err(RayIntersectionError::NoRayTravelDistance)
        }
    }

    pub fn new_model_identifier(normal: Vector3, point: Point3, ray: &Ray, material: Material, was_inside: bool, model_identier: Uuid) -> Result<Self, RayIntersectionError> {
        match Self::new(normal, point, ray, material, was_inside) {
            Ok(mut intersection) => {
                intersection.set_model_identifier_mut(Some(model_identier));
                Ok(intersection)
            },
            Err(err) => Err(err)
        }
    }

    pub fn get_intersection_point(&self) -> &Point3 {
        &self.point
    }

    pub fn get_normal_vector(&self) -> &Vector3 {
        &self.normal.as_ref()
    }

    pub fn get_distance_to_intersection(&self) -> FloatType {
        self.distance_to_intersection
    }

    pub fn get_intersector_ray(&self) -> &Ray {
        &self.ray
    }

    pub fn get_view_direction(&self) -> Vector3 {
        -self.ray.get_direction()
    }

    pub fn get_material(&self) -> &Material {
        &self.material_at_intersection
    }

    pub fn get_ray_medium(&self) -> Option<Material> {
        self.ray.get_medium()
    }

    pub fn was_inside(&self) -> bool {
        self.was_inside
    }

    pub fn get_transformed(self, transformation_matrix: &Matrix4) -> Result<Self, RayIntersectionError> {
        let point = Point3::from_homogeneous(transformation_matrix * self.point.to_homogeneous()).expect("Unhomogeneous transformed point");
        let normal = Vector3::from_homogeneous(transformation_matrix * self.normal.to_homogeneous()).expect("Unhomogeneous transformed vector");
        let ray = self.ray.get_transformed(transformation_matrix);

        Self::new(normal, point, &ray, self.material_at_intersection, self.was_inside)
    }

    pub fn get_model_identifier(&self) -> Option<&Uuid> {
        self.model_identifier.as_ref()
    }

    pub fn set_model_identifier_mut(&mut self, identifier: Option<Uuid>) {
        self.model_identifier = identifier;
    }

    pub fn set_model_identifier(&self, identifier: Option<Uuid>) -> Self {
        Self {  model_identifier: identifier,
                ..self.clone() }
    }

    pub fn is_same_model_intersection(&self, rhs: &RayIntersection) -> Result<bool, RayIntersectionError> {
        if self.model_identifier.is_some() && rhs.model_identifier.is_some() {
            Ok(self.model_identifier.unwrap() == rhs.model_identifier.unwrap())
        } else {
            Err(RayIntersectionError::NoModelIdentifierPresent)
        }
    }
}


#[cfg(test)]
mod tests {

}