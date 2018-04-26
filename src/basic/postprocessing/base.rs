use std::cmp;

use defs::{Point2Int, Vector2Int, IntType};

#[derive(Debug)]
pub enum RectError {
    InvalidRect
}

pub struct Rect {
    left_up: Point2Int,
    right_down: Point2Int,
}

impl Rect {
    pub fn new(left_up: Point2Int, right_down: Point2Int) -> Result<Self, RectError> {
        let result = Self {
            left_up: left_up,
            right_down: right_down
        };
        if result.check_rect() {
            Ok(result)
        } else {
            Err(RectError::InvalidRect)
        }
    }

    pub fn new_from_coords((first, second): (Point2Int, Point2Int)) -> Self {
        let left_up = Point2Int::new(cmp::min(first.x, second.x), cmp::min(first.y, second.y));
        let right_down = Point2Int::new(cmp::max(first.x, second.x), cmp::max(first.y, second.y));
        
        Self::new(left_up, right_down).unwrap()
    }

    pub fn new_from_left_up(leftup: Point2Int, width: IntType, height: IntType) -> Self {
        let other = Point2Int::new(leftup.x + width, leftup.y + height);

        Self::new_from_coords((leftup, other))
    }

    pub fn new_from_middle(middle: Point2Int, width_radius: IntType, height_radius: IntType) -> Self {
        let left_up = Point2Int::new(middle.x - width_radius, middle.y - height_radius);
        let right_down = Point2Int::new(middle.x + width_radius, middle.y + height_radius);

        Self::new_from_coords((left_up, right_down))
    }

    pub fn new_square_from_middle(middle: Point2Int, radius: IntType) -> Self {
        Self::new_from_middle(middle, radius, radius)
    }

    pub fn get_left_up_corner(&self) -> &Point2Int {
        &self.left_up
    }

    pub fn get_right_down_corner(&self) -> &Point2Int {
        &self.right_down
    }

    pub fn get_primitive(&self) -> (Point2Int, Point2Int) {
        (self.left_up, self.right_down)
    }

    pub fn get_width(&self) -> IntType {
        self.right_down.x - self.left_up.x + 1
    }

    pub fn get_height(&self) -> IntType {
        self.right_down.y - self.left_up.y + 1
    }

    pub fn get_dimensions(&self) -> (IntType, IntType) {
        (self.get_width(), self.get_height())
    }

    pub fn get_size(&self) -> IntType {
        self.get_width() * self.get_height()
    }

    pub fn is_singularity(&self) -> bool {
        self.get_size() == 1
    }

    pub fn coord_in_rect(&self, coord: &Point2Int) -> bool {
        self.left_up.x <= coord.x && coord.x <= self.right_down.x &&
        self.left_up.y <= coord.y && coord.y <= self.right_down.y
    }

    pub fn offset_mut(&mut self, offset: &Vector2Int) {
        self.left_up += offset;
        self.right_down += offset;
    }

    pub fn offset(&self, offset: &Vector2Int) -> Self {
        Self {
            left_up: self.left_up + offset,
            right_down: self.right_down + offset
        }
    }

    fn check_rect(&self) -> bool {
        self.left_up.x <= self.right_down.x && self.left_up.y <= self.right_down.y
    }
}

pub struct RectIterator<'rect> {
    rect: &'rect Rect,
    state: IntType
}

impl<'rect> RectIterator<'rect> {
    pub fn new(rect: &'rect Rect) -> Self {
        Self {
            rect: rect,
            state: 0
        }
    }

    fn to_offset(&self, offset: IntType) -> Vector2Int {
        let horizantal_offset = offset % (self.rect.get_width());
        let height_offset = offset / (self.rect.get_height());

        Vector2Int::new(horizantal_offset, height_offset)
    }
}

impl<'rect> Iterator for RectIterator<'rect> {
    type Item = Point2Int;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rect.get_size() > self.state {
            let offset = self.to_offset(self.state);
            self.state += 1;
            Some(self.rect.get_left_up_corner() + offset)
        } else {
            None
        }
    }
}