use defs::{Point3, Vector3, Point2Int, FloatType, IntType};
use core::{Ray};
use tools::{CompareWithTolerance, Between, Vector3Extensions};
use na::{Unit};

#[derive(Debug)]
pub enum ScreenError {
    PixelOutOfBoundsError
}

pub struct Screen {
    center: Point3,
    up: Unit<Vector3>,
    left: Unit<Vector3>,
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

        Self {  center: center,
                up: up_corrected,
                left: left_normlized,
                width: width,
                height: height,
                horizontal_resolution: h_res,
                vertical_resolution: v_res}
    }

    pub fn new_unit(center: Point3, normal: Vector3, up: Vector3, width_to_height_ratio: FloatType, height: FloatType, v_res: IntType) -> Self {
        let vertical_resolution = v_res as f64;
        Self::new(center, normal, up, width_to_height_ratio * height, height, (vertical_resolution * width_to_height_ratio).round() as IntType, v_res)
    }

    pub fn get_resolutoion(&self) -> (IntType, IntType) {
        (self.horizontal_resolution, self.vertical_resolution)
    }

    pub fn get_pixel_count(&self) -> IntType {
        self.horizontal_resolution * self.vertical_resolution
    }

    fn get_pixel_coord_core(&self, coord: Point2Int) -> Point3 {
        let left = -(((coord.x as FloatType / ((self.horizontal_resolution as FloatType) / 2.0)) - 1.0) * self.width);
        let up = -(((coord.y as FloatType / ((self.vertical_resolution as FloatType) / 2.0)) - 1.0) * self.height);

        self.center + (self.up.as_ref() * up + self.left.as_ref() * left)
    }

    pub fn get_pixel_coord(&self, coord: Point2Int) -> Result<Point3, ScreenError> {
        if coord.x.between(&0, &self.horizontal_resolution) && coord.y.between(&0, &self.vertical_resolution) {
            Ok(self.get_pixel_coord_core(coord))
        } else {
            Err(ScreenError::PixelOutOfBoundsError)
        }
    }

    pub fn get_nth_pixel_coord(&self, pixel_index: IntType) -> Result<Point3, ScreenError> {
        let x = pixel_index % self.horizontal_resolution;
        let y = pixel_index / self.horizontal_resolution;
        self.get_pixel_coord(Point2Int::new(x, y))
    }

    pub fn get_nth_pixel_screen_coord(&self, pixel_index: IntType) -> Result<Point2Int, ScreenError> {
        let x = pixel_index % self.horizontal_resolution;
        let y = pixel_index / self.horizontal_resolution;
        if 0 <= x && x < self.horizontal_resolution && 0 <= y && y < self.vertical_resolution {
            Ok(Point2Int::new(x, y))
        } else {
            Err(ScreenError::PixelOutOfBoundsError)
        }
    }
}

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
        match self.screen.get_pixel_coord(coordinate) {
            Ok(point) => {
                let eye_coord = self.eye.get_position();
                let ray_direction = point - eye_coord;
                Ok(Ray::new(*self.eye.get_position(), ray_direction))
            },
            Err(err) => Err(ViewError::ScreenRelated(err))
        }
    }

    pub fn get_ray_to_screen_pixel_index(&self, index: IntType) -> Result<Ray, ViewError> {
        match self.screen.get_nth_pixel_coord(index) {
            Ok(point) => {
                let eye_coord = self.eye.get_position();
                let ray_direction = point - eye_coord;
                Ok(Ray::new(*self.eye.get_position(), ray_direction))
            },
            Err(err) => Err(ViewError::ScreenRelated(err))
        }
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
        match screen.get_nth_pixel_screen_coord(*index) {
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