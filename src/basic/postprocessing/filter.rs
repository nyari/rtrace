use defs::{IntType, FloatType, Point2Int};

use core::{Screen, Color, ImmutableSceneBuffer, SceneBufferError};
use basic::{Rect, RectIterator};
use tools::{CompareWithTolerance};

pub struct MedianFilter<'obuffer> {
    original_buffer: &'obuffer ImmutableSceneBuffer,
    rect_radius: IntType,
}

impl<'obuffer> MedianFilter<'obuffer> {
    pub fn new(original_buffer: &'obuffer ImmutableSceneBuffer, rect_radius: IntType) -> Self {
        Self {
            original_buffer: original_buffer,
            rect_radius: rect_radius
        }
    }

    fn get_vector_median(vector: &mut Vec<FloatType>) -> Option<FloatType> {
        vector.sort_by(|lhs, rhs| lhs.compare_eps(rhs));
        if !vector.is_empty() {
            let vec_len = vector.len();
            let vec_len_2 = vec_len / 2;
            if vector.len() % 2 == 0 || vec_len == 1 {
                Some(vector[vec_len_2])
            } else {
                let average = (vector[vec_len_2] + vector[vec_len_2 + 1]) / 2.0;
                Some(average)
            }
        } else {
            None
        }
    }
}

impl<'obuffer> ImmutableSceneBuffer for MedianFilter<'obuffer> {
    fn get_pixel_value(&self, pixel: Point2Int) -> Result<Option<Color>, SceneBufferError> {
        if self.original_buffer.get_pixel_value(pixel).is_ok() {
            let median_rect = Rect::new_square_from_middle(pixel, self.rect_radius);
            let mut red_channel_vector: Vec<FloatType> = Vec::with_capacity(median_rect.get_size() as usize);
            let mut green_channel_vector: Vec<FloatType> = Vec::with_capacity(median_rect.get_size() as usize);
            let mut blue_channel_vector: Vec<FloatType> = Vec::with_capacity(median_rect.get_size() as usize);

            let mut has_value = false;

            for coord in RectIterator::new(&median_rect) {
                match self.original_buffer.get_pixel_value(coord) {
                    Ok(Some(color)) => {
                        let (r, g, b) = color.get();
                        red_channel_vector.push(r);
                        green_channel_vector.push(g);
                        blue_channel_vector.push(b);
                        has_value = true;
                    },
                    Ok(None) => {
                        red_channel_vector.push(0.0);
                        green_channel_vector.push(0.0);
                        blue_channel_vector.push(0.0);
                    }
                    Err(SceneBufferError::InvalidInputCoord) => (),
                    Err(error) => return Err(error)
                }
            }
            if has_value {
                Ok(Some(Color::new(Self::get_vector_median(&mut red_channel_vector).unwrap_or(0.0),
                                   Self::get_vector_median(&mut green_channel_vector).unwrap_or(0.0),
                                   Self::get_vector_median(&mut blue_channel_vector).unwrap_or(0.0))))
            } else {
                Ok(None)
            }
        } else {
            Err(SceneBufferError::InvalidInputCoord)
        }
    }

    fn get_screen(&self) -> &Screen {
        self.original_buffer.get_screen()
    }
}