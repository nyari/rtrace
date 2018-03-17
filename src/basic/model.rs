use core::{Model, Material, RayIntersection, Ray, RayIntersectionError};
use defs::{Point3, Vector3, FloatType};
use tools::{CompareWithTolerance};
use na;
use na::{Unit};
use std;
use uuid::{Uuid};

pub struct SolidSphere {
    material: Material,
    origo: Point3,
    radius: FloatType,
    identifier: Uuid
}

impl SolidSphere {
    pub fn new(material: Material) -> Self {
        Self {  material: material,
                origo: Point3::new(0.0, 0.0, 0.0),
                radius: 1.0,
                identifier: Uuid::new_v4()}
    }

    pub fn new_positioned(material: Material, origo: Point3, radius: FloatType) -> Self {
        Self {  material: material,
                origo: origo,
                radius: radius,
                identifier: Uuid::new_v4()}
    }

    pub fn set_custom_identifier(&mut self, identifier: Uuid) {
        self.identifier = identifier;
    }
}

impl Model for SolidSphere {
    fn get_intersection(&self, ray: &Ray) -> Option<RayIntersection> {
        let dir = ray.get_direction();
        let origin = ray.get_origin();
        let ray_origo = origin - self.origo;

        let a = dir.x.powi(2) + dir.y.powi(2) + dir.z.powi(2);
        let b = 2.0 * (ray_origo.x * dir.x + ray_origo.y * dir.y + ray_origo.z * dir.z);
        let c = ray_origo.x.powi(2) + ray_origo.y.powi(2) + ray_origo.z.powi(2) - self.radius.powi(2);

        let determinant = b.powi(2) - 4.0 * a * c;
        if determinant.less_eps(&0.0) {
            None
        } else {
            let t1 = (-b + determinant.sqrt()) / (2.0 * a);
            let t2 = (-b - determinant.sqrt()) / (2.0 * a);
            let result_calc = |t, inside: bool| {
                let intersection_point = origin + dir * t;
                let normal = if !inside { intersection_point - self.origo } else { self.origo - intersection_point };

                match RayIntersection::new_model_identifier(normal, intersection_point, ray, self.material, inside, self.identifier) {
                    Ok(intersection) => Some(intersection),
                    Err(RayIntersectionError::NoRayTravelDistance) => None,
                    _ => panic!("Unrecoverable RayIntersectin:new_model_identifier error")
                }
            };

            if t1.is_sign_negative() && t2.is_sign_negative() {
                None
            } else if t1.is_sign_positive() && t2.is_sign_negative() {
                result_calc(t1, true)
            } else {
                match result_calc(t2, false) {
                    Some(result) => Some(result),
                    None => result_calc(t1, true),
                }
            }
        }
    }
}


pub struct SolidPlane {
    material: Material,
    base: Point3,
    normal: Vector3,
    identifier: Uuid,
}

impl SolidPlane {
    pub fn new(material: Material) -> Self {
        Self {  material: material,
                base: Point3::origin(),
                normal: Vector3::new(0.0, 0.0, 1.0),
                identifier: Uuid::new_v4()
        }
    }

    pub fn new_positioned(material: Material, base: Point3, normal: Unit<Vector3>) -> Self {
        Self {  material: material,
                base: base,
                normal: normal.unwrap(),
                identifier: Uuid::new_v4()
        }
    }

    pub fn set_custom_identifier(&mut self, identifier: Uuid) {
        self.identifier = identifier;
    }
}

impl Model for SolidPlane {
    fn get_intersection(&self, ray: &Ray) -> Option<RayIntersection> {
        let origin = ray.get_origin();
        let dir = ray.get_direction();

        let u = self.normal.dot(dir);
        if !u.near_zero_eps() {
            let k = self.normal.x * self.base.x + self.normal.y * self.base.y + self.normal.z * self.base.z;
            let s = self.normal.x * origin.x + self.normal.y * origin.y + self.normal.z * origin.z;
            let t = (k-s) / u;
            if t.greater_eq_eps(&0.0) {
                let is_inside = na::angle(&self.normal, dir).less_eq_eps(&std::f64::consts::FRAC_PI_2);
                let point = origin + dir * t;
                if !is_inside {
                    match RayIntersection::new_model_identifier(self.normal, point, ray, self.material, is_inside, self.identifier) {
                        Ok(intersection) => Some(intersection),
                        Err(RayIntersectionError::NoRayTravelDistance) => None,
                        _ => panic!("Unrecoverable RayIntersectin:new_model_identifier error")
                    }
                } else {
                    match RayIntersection::new_model_identifier(-self.normal, point, ray, self.material, is_inside, self.identifier) {
                        Ok(intersection) => Some(intersection),
                        Err(RayIntersectionError::NoRayTravelDistance) => None,
                        _ => panic!("Unrecoverable RayIntersectin:new_model_identifier error")
                    }
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use core::Color;

    fn test_solid_unit_sphere(test_ray: &Ray, expected_result: Option<&Point3>) {
        let test_material = Material::new_shiny(Color::new(1.0, 1.0, 1.0), (Color::new(1.0, 1.0, 1.0), 1.5), None);
        let test_sphere = SolidSphere::new(test_material);

        let intersection_result = test_sphere.get_intersection(test_ray);

        match expected_result {
            Some(expected_point) => {
                let intersection = intersection_result.expect("Was expected intersection but none intersected");

                assert_relative_eq!(expected_point, intersection.get_intersection_point());
            },
            None => {
                assert!(intersection_result.is_none());
            }
        }
    }

    #[test]
    fn test_solid_unit_sphere_head_on() {
        let test_ray = Ray::new_single_shot(Point3::new(2.0, 0.0, 0.0), Vector3::new(-1.0, 0.0, 0.0));
        test_solid_unit_sphere(&test_ray, Some(&Point3::new(1.0, 0.0, 0.0)));
    }

    #[test]
    fn test_solid_unit_sphere_tangential_hit() {
        let test_ray = Ray::new_single_shot(Point3::new(2.0, 0.0, 1.0), Vector3::new(-1.0, 0.0, 0.0));
        test_solid_unit_sphere(&test_ray, Some(&Point3::new(0.0, 0.0, 1.0)));
    }

    #[test]
    fn test_solid_unit_sphere_tangential_miss() {
        let test_ray = Ray::new_single_shot(Point3::new(2.0, 0.0, 1.01), Vector3::new(-1.0, 0.0, 0.0));
        test_solid_unit_sphere(&test_ray, None);
    }
}