use defs::{Point3, Vector3, Matrix3, Point2Int, FloatType, IntType};
use core::{Ray};
use tools::{CompareWithTolerance, Vector3Extensions};
use na::{Unit};

#[derive(Debug)]
pub enum ScreenError {
    PixelOutOfBoundsError
}


#[derive(Copy, Clone)]
pub struct Screen {
    center: Point3,
    up: Unit<Vector3>,
    left: Unit<Vector3>,
    normal: Unit<Vector3>,
    to_plane_trasform_matrix: Matrix3,
    width: FloatType,
    height: FloatType,
    horizontal_resolution: IntType,
    vertical_resolution: IntType,
}

impl Screen {
    pub fn new(center: Point3, normal: Vector3, up: Vector3, width: FloatType, height: FloatType, h_res: IntType, v_res: IntType) -> Self {
        if h_res <= 0 || v_res <= 0 || width.less_eq_eps(&0.0) || height.less_eq_eps(&0.0) {
            panic!("Invalid screen input values");
        }
        if normal.same_direction_as(&up) {
            panic!("Screen normal and up vectors are pointing in the same direction");
        }

        let normal_normalized = Unit::new_normalize(normal);
        let up_normalized = Unit::new_normalize(up);
        let left_normlized = Unit::new_unchecked(up_normalized.cross(&normal_normalized));
        let up_corrected = Unit::new_unchecked(normal_normalized.cross(&left_normlized));
        let transform_matrix = {
            let mut result = Matrix3::from_columns(&[left_normlized.unwrap().clone(), up_normalized.unwrap().clone(), normal_normalized.unwrap().clone()]);
            result.try_inverse().expect("Uninvertible screen transformation matrix")
        };

        Self {  center: center,
                up: up_corrected,
                left: left_normlized,
                normal: normal_normalized,
                to_plane_trasform_matrix: transform_matrix,
                width: width,
                height: height,
                horizontal_resolution: h_res,
                vertical_resolution: v_res}
    }

    pub fn new_unit(center: Point3, normal: Vector3, up: Vector3, width_to_height_ratio: FloatType, height: FloatType, v_res: IntType) -> Self {
        let vertical_resolution = v_res as f64;
        Self::new(center, normal, up, width_to_height_ratio * height, height, (vertical_resolution * width_to_height_ratio).round() as IntType, v_res)
    }

    pub fn get_resolution(&self) -> (IntType, IntType) {
        (self.horizontal_resolution, self.vertical_resolution)
    }

    pub fn get_pixel_count(&self) -> IntType {
        self.horizontal_resolution * self.vertical_resolution
    }

    fn check_pixel_bounds(&self, coord: &Point2Int) -> bool {
        (0 <= coord.x && coord.x < self.horizontal_resolution) && (0 <= coord.y && coord.y < self.vertical_resolution)
    }

    fn get_pixel_coord_core(&self, coord: &Point2Int) -> Point3 {
        let left = -(((coord.x as FloatType / ((self.horizontal_resolution as FloatType) / 2.0)) - 1.0) * self.width / 2.0);
        let up = -(((coord.y as FloatType / ((self.vertical_resolution as FloatType) / 2.0)) - 1.0) * self.height / 2.0);

        self.center + (self.up.as_ref() * up + self.left.as_ref() * left)
    }

    pub fn get_pixel_coord(&self, coord: &Point2Int) -> Result<Point3, ScreenError> {
        if self.check_pixel_bounds(coord) {
            Ok(self.get_pixel_coord_core(&coord))
        } else {
            Err(ScreenError::PixelOutOfBoundsError)
        }
    }

    pub fn get_pixel_coord_by_index(&self, pixel_index: IntType) -> Result<Point3, ScreenError> {
        let x = pixel_index % self.horizontal_resolution;
        let y = pixel_index / self.horizontal_resolution;
        self.get_pixel_coord(&Point2Int::new(x, y))
    }

    pub fn get_pixel_screen_coord_by_index(&self, pixel_index: IntType) -> Result<Point2Int, ScreenError> {
        let x = pixel_index % self.horizontal_resolution;
        let y = pixel_index / self.horizontal_resolution;
        let potential_result = Point2Int::new(x, y);
        if self.check_pixel_bounds(&potential_result) {
            Ok(potential_result)
        } else {
            Err(ScreenError::PixelOutOfBoundsError)
        }
    }

    pub fn get_pixel_index_by_screen_coord(&self, coord: &Point2Int) -> Result<IntType, ScreenError> {
        if self.check_pixel_bounds(coord) {
            Ok(coord.x + self.horizontal_resolution * coord.y)
        } else {
            Err(ScreenError::PixelOutOfBoundsError)
        }
    }

    fn get_screen_plane_intersection_point(&self, ray: &Ray) -> Option<Point3> {
        let origin = ray.get_origin();
        let dir = ray.get_direction();

        let u = self.normal.dot(dir);
        if !u.near_zero_eps() {
            let k = self.normal.x * self.center.x + self.normal.y * self.center.y + self.normal.z * self.center.z;
            let s = self.normal.x * origin.x + self.normal.y * origin.y + self.normal.z * origin.z;
            let t = (k-s) / u;
            if t.greater_eq_eps(&0.0) {
                Some(origin + dir * t)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_intersected_pixel(&self, ray: &Ray) -> Option<Point2Int> {
        if let Some(plane_intersection_point) = self.get_screen_plane_intersection_point(ray) {
            let path_from_origin_to_intersection = plane_intersection_point - self.center;
            let transformed_vector = self.to_plane_trasform_matrix * path_from_origin_to_intersection;
            let left_scaler = transformed_vector.x;
            let up_scaler = transformed_vector.y;

            let left_multiplier = left_scaler / (self.width / 2.0);
            let up_multiplier = up_scaler / (self.height / 2.0);

            if left_multiplier.abs().less_eq_eps(&1.0) && up_multiplier.abs().less_eq_eps(&1.0) {
                let x = (self.horizontal_resolution as FloatType / 2.0) * (-left_multiplier + 1.0);
                let y = (self.vertical_resolution as FloatType / 2.0) * (-up_multiplier + 1.0);
                Some(Point2Int::new(x.round() as IntType,y.round() as IntType))
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub struct ScreenIterator<'screen> {
    state: IntType,
    screen: &'screen Screen,
}

impl<'screen> ScreenIterator<'screen> {
    pub fn new(screen: &'screen Screen) -> Self {
        Self {  screen: screen,
                state: 0}
    }

    pub fn get_screen_coord(&self, index: &IntType) -> Option<Point2Int> {
        match self.screen.get_pixel_screen_coord_by_index(*index) {
            Ok(coord) => Some(coord),
            _ => None
        }
    }
}

impl<'screen> Iterator for ScreenIterator<'screen> {
    type Item = Point2Int;
    fn next(&mut self) -> Option<Point2Int> {
        let result = self.get_screen_coord(&self.state);
        self.state += 1;
        result
    }
}

#[derive(Copy, Clone)]
pub struct Eye {
    position: Point3,
    direction: Unit<Vector3>,
}

impl Eye {
    pub fn new(position: Point3, direction: Vector3) -> Self {
        Self {  position: position,
                direction: Unit::new_normalize(direction)}
    }

    pub fn get_position(&self) -> &Point3 {
        &self.position
    }

    pub fn get_direction(&self) -> &Vector3 {
        &self.direction.as_ref()
    }
}

#[derive(Debug)]
pub enum ViewError {
    ScreenRelated(ScreenError)
}

#[derive(Copy, Clone)]
pub struct View {
    screen: Screen,
    eye: Eye
}

impl View {
    pub fn new(screen: Screen, eye: Eye) -> Self {
        Self {  screen: screen,
                eye: eye}
    }

    pub fn new_unit(eye_position: Point3, eye_direction: Vector3, screen_up: Vector3, screen_width_to_height_ratio: FloatType, screen_height: FloatType, screen_v_res: IntType) -> Self{
        let eye_unit_direction = Unit::new_normalize(eye_direction);
        Self {  screen: Screen::new_unit(eye_position + eye_unit_direction.as_ref(),
                                         *eye_unit_direction.as_ref(), 
                                         screen_up,
                                         screen_width_to_height_ratio,
                                         screen_height,
                                         screen_v_res),
                eye: Eye::new(eye_position, 
                              *eye_unit_direction.as_ref())
        }
    }

    pub fn get_ray_to_screen_coordinate(&self, coordinate: Point2Int) -> Result<Ray, ViewError> {
        match self.screen.get_pixel_coord(&coordinate) {
            Ok(point) => {
                let eye_coord = self.eye.get_position();
                let ray_direction = point - eye_coord;
                Ok(Ray::new(*self.eye.get_position(), ray_direction))
            },
            Err(err) => Err(ViewError::ScreenRelated(err))
        }
    }

    pub fn get_ray_to_screen_pixel_index(&self, index: IntType) -> Result<Ray, ViewError> {
        match self.screen.get_pixel_coord_by_index(index) {
            Ok(point) => {
                let eye_coord = self.eye.get_position();
                let ray_direction = point - eye_coord;
                Ok(Ray::new(*self.eye.get_position(), ray_direction))
            },
            Err(err) => Err(ViewError::ScreenRelated(err))
        }
    }

    pub fn get_screen_coord_to_world_point(&self, point: &Point3) -> Option<Point2Int> {
        let test_ray = Ray::new(*point, (self.eye.position - point));
        self.screen.get_intersected_pixel(&test_ray)
    }

    pub fn get_screen(&self) -> &Screen {
        &self.screen
    }

    pub fn get_eye(&self) -> &Eye {
        &self.eye
    }

    pub fn get_screen_pixel_count(&self) -> IntType {
        self.screen.get_pixel_count()
    }
}


pub struct ViewIterator<'view> {
    state: IntType,
    view: &'view View,
}

impl<'view> ViewIterator<'view> {
    pub fn new(view: &'view View) -> Self {
        Self {  view: view,
                state: 0}
    }

    pub fn get_screen_coord(&self, index: &IntType) -> Option<Point2Int> {
        let screen = self.view.get_screen();
        match screen.get_pixel_screen_coord_by_index(*index) {
            Ok(coord) => Some(coord),
            _ => None
        }
    }
}

impl<'view> Iterator for ViewIterator<'view> {
    type Item = (Ray, Point2Int);
    fn next(&mut self) -> Option<(Ray, Point2Int)> {
        let result = match self.get_screen_coord(&self.state) {
            Some(coordinate) => Some((self.view.get_ray_to_screen_coordinate(coordinate).unwrap(), coordinate)),
            None => None
        };
        self.state += 1;
        
        result
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screen_get_intersected_pixel_hit_origin() {
        let test_screen = Screen::new_unit(Point3::origin(), Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.0, 1.0, 0.0), 1.0, 1.0, 1000);
        let test_ray = Ray::new(Point3::new(0.0, 0.0, 1.0), Vector3::new(0.0, 0.0, -1.0));

        let result = test_screen.get_intersected_pixel(&test_ray).expect("Should have hit screen");

        assert_eq!(result, Point2Int::new(500, 500));
    }

    #[test]
    fn screen_get_intersected_pixel_hit_to_not_origin() {
        let test_screen = Screen::new_unit(Point3::origin(), Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.0, 1.0, 0.0), 1.0, 1.0, 1000);
        let test_ray = Ray::new(Point3::new(0.25, 0.25, 1.0), Vector3::new(0.0, 0.0, -1.0));

        let result = test_screen.get_intersected_pixel(&test_ray).expect("Should have hit screen");

        assert_eq!(result, Point2Int::new(250, 250));
    }

    #[test]
    fn screen_get_intersected_pixel_hit_not_to_origin_not_perpendicular() {
        let target_on_screen_point = Point3::new(0.25, 0.25, 0.0);
        let eye_point = Point3::new(0.0, 0.0, -1.0);
        let eye_ray_direction = (target_on_screen_point - eye_point).normalize();

        let test_screen = Screen::new_unit(Point3::origin(), Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.0, 1.0, 0.0), 1.0, 1.0, 1000);
        let test_ray = Ray::new(eye_point + (eye_ray_direction * 5.0), -eye_ray_direction);

        let result = test_screen.get_intersected_pixel(&test_ray).expect("Should have hit screen");

        assert_eq!(result, Point2Int::new(250, 250));
    }

    #[test]
    fn screen_get_intersected_pixel_miss() {
        let test_screen = Screen::new_unit(Point3::origin(), Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.0, 1.0, 0.0), 1.0, 1.0, 1000);
        let test_ray = Ray::new(Point3::new(0.6, 0.6, 1.0), Vector3::new(0.0, 0.0, -1.0));

        assert!(test_screen.get_intersected_pixel(&test_ray).is_none());
    }
}