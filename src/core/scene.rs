use defs::{Point2Int};
use core::{RayCaster, IlluminationCaster, View, Color, RayIntersection, Screen, ScreenIterator};
use std::sync::{Arc, Mutex};


#[derive(Debug)]
pub enum SceneError {
    NothingIntersected,
    InvalidInputCoord
}

pub trait Scene: Send + Sync {
    fn get_pixel_color(&self, pixel: Point2Int) -> Result<Color, SceneError>;
    fn get_pixel_intersection(&self, pixel: Point2Int) -> Result<RayIntersection, SceneError>;
    fn get_view(&self) -> &View;
    fn get_ray_caster(&self) -> &RayCaster;
    fn get_illumination_caster(&self) -> &IlluminationCaster;
}

#[derive(Debug)]
pub enum SceneBufferError {
    InvalidInputCoord,
    MutexLockError,
    OtherBufferNotSameSize
}

#[derive(Debug)]
pub enum SceneBufferLayering {
    Under,
    Over
}

pub trait ImmutableSceneBuffer: Send + Sync {
    fn get_pixel_value(&self, pixel: Point2Int) -> Result<Option<Color>, SceneBufferError>;
    fn get_screen(&self) -> &Screen;

    fn is_same_size_buffer(&self, rhs: &ImmutableSceneBuffer) -> bool {
        self.get_screen().get_resolution() == rhs.get_screen().get_resolution()
    }
}

pub trait MutableSceneBuffer: ImmutableSceneBuffer +  Send + Sync { //Internally mutable (Mutex), thread safe
    fn set_pixel_value(&self, pixel: Point2Int, color: &Color) -> Result<(), SceneBufferError>;
    fn accumulate_pixel_value(&self, pixel: Point2Int, color: &Color) -> Result<(), SceneBufferError>;
    fn reset_pixel(&self, pixel: Point2Int) -> Result<(), SceneBufferError>;

    fn layer_buffer(&self, place: SceneBufferLayering, rhs: &ImmutableSceneBuffer) -> Result<(), SceneBufferError> {
        let layerer = |self_color_option: Option<Color>, rhs_color_option: Option<Color>| {
            match place {
                SceneBufferLayering::Under => {
                    match self_color_option {
                        Some(_) => None,
                        None => rhs_color_option
                    }
                },
                SceneBufferLayering::Over => {
                    match rhs_color_option {
                        Some(color) => Some(color),
                        None => None
                    }
                }
            }
        };
        if self.is_same_size_buffer(rhs) {
            for (pixel, rhs_color_option) in SceneBufferIterator::new(rhs) {
               match self.get_pixel_value(pixel) {
                   Ok(self_color_option) => {
                       match layerer(self_color_option, rhs_color_option) {
                           Some(color) => {
                               match self.set_pixel_value(pixel, &color) {
                                   Err(error) => return Err(error),
                                   _ => ()
                               }
                           }
                           _ => ()
                       }
                   },
                   Err(error) => {
                       return Err(error);
                   }
               }
            }
            Ok(())
        } else {
            Err(SceneBufferError::OtherBufferNotSameSize)
        }
    }

    fn combine_buffer(&self, rhs: &ImmutableSceneBuffer) -> Result<(), SceneBufferError> {
        if self.is_same_size_buffer(rhs) {
            for (pixel, rhs_color_option) in SceneBufferIterator::new(rhs) {
                match rhs_color_option {
                    Some(color) => {
                        match self.accumulate_pixel_value(pixel, &color) {
                            Err(error) => return Err(error),
                            _ => ()
                        }
                    },
                    _ => ()
                }
            }

            Ok(())
        } else {
            Err(SceneBufferError::OtherBufferNotSameSize)
        }
    }
}


pub trait SceneBuffer: ImmutableSceneBuffer + MutableSceneBuffer + Send + Sync {

}


pub struct SceneBufferIterator<'buffer> {
    buffer: &'buffer ImmutableSceneBuffer,
    screen_iterator: ScreenIterator
}

impl<'buffer> SceneBufferIterator<'buffer> {
    pub fn new(buffer: &'buffer ImmutableSceneBuffer) -> Self {
        Self {
            buffer: buffer,
            screen_iterator: ScreenIterator::new(buffer.get_screen())
        }
    }
}

impl<'buffer> Iterator for SceneBufferIterator<'buffer> {
    type Item = (Point2Int, Option<Color>);
    fn next(&mut self) -> Option<Self::Item> {
        match self.screen_iterator.next() {
            Some(next_coord) => Some((next_coord, self.buffer.get_pixel_value(next_coord).unwrap())),
            None => None
        }
    }
}

#[derive(Debug)]
pub enum BasicSceneBufferError {
    BufferNotCorrectSize
}

pub struct BasicSceneBuffer {
    screen: Screen,
    buffer: Arc<Mutex<Vec<Option<Color>>>>,
}

impl BasicSceneBuffer {
    pub fn new(screen: Screen) -> Self {
        let mut buffer: Vec<Option<Color>> = Vec::with_capacity(screen.get_pixel_count() as usize);
        buffer.resize(screen.get_pixel_count() as usize, None);
        Self {
            screen: screen,
            buffer: Arc::new(Mutex::new(buffer))
        }
    }

    fn map_pixel_to_buffer(&self, pixel: Point2Int) -> Option<usize> {
        if let Ok(result) = self.screen.get_pixel_index_by_screen_coord(&pixel) {
            if result < 0 {
                panic!("Negative value as screen pixel index");
            }
            Some(result as usize)
        } else {
            None
        } 
    }

    pub fn with_buffer(screen: Screen, input_buffer: Vec<Option<Color>>) -> Result<Self, BasicSceneBufferError> {
        if input_buffer.len() == screen.get_pixel_count() as usize {
            Ok(Self {
                screen: screen,
                buffer: Arc::new(Mutex::new(input_buffer))
            })
        } else {
            Err(BasicSceneBufferError::BufferNotCorrectSize)
        }
    }
}


impl ImmutableSceneBuffer for BasicSceneBuffer {
    fn get_pixel_value(&self, pixel: Point2Int) -> Result<Option<Color>, SceneBufferError> {
        if let Some(index) = self.map_pixel_to_buffer(pixel) {
            if let Ok(ref mut result_buffer_acessor) = self.buffer.lock() {
                let buffer_item = result_buffer_acessor.get_mut(index).unwrap();
                Ok(*buffer_item)
            } else {
                Err(SceneBufferError::MutexLockError)
            }
        } else {
            Err(SceneBufferError::InvalidInputCoord)
        }
    }

    fn get_screen(&self) -> &Screen {
        &self.screen        
    }
}


impl MutableSceneBuffer for BasicSceneBuffer {
    fn set_pixel_value(&self, pixel: Point2Int, color: &Color) -> Result<(), SceneBufferError> {
        if let Some(index) = self.map_pixel_to_buffer(pixel) {
            if let Ok(ref mut result_buffer_acessor) = self.buffer.lock() {
                let buffer_item = result_buffer_acessor.get_mut(index).unwrap();
                *buffer_item = Some(*color);
                Ok(())
            } else {
                Err(SceneBufferError::MutexLockError)
            }
        } else {
            Err(SceneBufferError::InvalidInputCoord)
        }
    }

    fn accumulate_pixel_value(&self, pixel: Point2Int, color: &Color) -> Result<(), SceneBufferError> {
        if let Some(index) = self.map_pixel_to_buffer(pixel) {
            if let Ok(ref mut result_buffer_acessor) = self.buffer.lock() {
                let buffer_item = result_buffer_acessor.get_mut(index).unwrap();
                if let Some(ref mut contained_color) = *buffer_item {
                    *contained_color += *color;
                } else {
                    *buffer_item = Some(*color);
                }
                Ok(())
            } else {
                Err(SceneBufferError::MutexLockError)
            }
        } else {
            Err(SceneBufferError::InvalidInputCoord)
        }
    }

    fn reset_pixel(&self, pixel: Point2Int) -> Result<(), SceneBufferError> {
        if let Some(index) = self.map_pixel_to_buffer(pixel) {
            if let Ok(ref mut result_buffer_acessor) = self.buffer.lock() {
                let buffer_item = result_buffer_acessor.get_mut(index).unwrap();
                *buffer_item = None;
                Ok(())
            } else {
                Err(SceneBufferError::MutexLockError)
            }
        } else {
            Err(SceneBufferError::InvalidInputCoord)
        }
    }
}

impl SceneBuffer for BasicSceneBuffer {

}

pub struct ImmutableSceneBufferWrapper<'buffer> {
    buffer: &'buffer SceneBuffer
}

impl<'buffer> ImmutableSceneBufferWrapper<'buffer> {
    pub fn new(buffer: &'buffer SceneBuffer) -> Self {
        Self {
            buffer: buffer
        }
    }
}

impl<'buffer> ImmutableSceneBuffer for ImmutableSceneBufferWrapper<'buffer> {
    fn get_pixel_value(&self, pixel: Point2Int) -> Result<Option<Color>, SceneBufferError> {
        self.buffer.get_pixel_value(pixel)
    }

    fn get_screen(&self) -> &Screen {
        self.buffer.get_screen()
    }
}