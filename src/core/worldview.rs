use defs::{Point2Int};
use core::{RayCaster, IlluminationCaster, View, Color, RayIntersection, Ray, LightIntersection};
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
    MutexLockError
}

pub trait SceneBuffer: Scene + Send + Sync { //Internally mutable (Mutex), thread safe
    fn set_pixel_value(&self, pixel: Point2Int, color: &Color) -> Result<(), SceneBufferError>;
    fn accumulate_pixel_value(&self, pixel: Point2Int, color: &Color) -> Result<(), SceneBufferError>;
    fn reset_pixel(&self, pixel: Point2Int) -> Result<(), SceneBufferError>;
    fn get_pixel_value(&self, pixel: Point2Int) -> Result<Option<Color>, SceneBufferError>;
}


pub trait WorldViewTrait: Scene + SceneBuffer {

}

pub struct WorldView<WorldT> {
    world: Arc<WorldT>,
    view: View,
    result_buffer: Arc<Mutex<Vec<Option<Color>>>>,
}

impl<WorldT> WorldView<WorldT>
    where WorldT: RayCaster + IlluminationCaster + Send + Sync
{
    #[allow(dead_code)]
    pub fn new(world: WorldT, view: View) -> Self {
        let (width, height) = view.get_screen().get_resolution();
        let size = (width * height) as usize;
        let mut new_buffer: Vec<Option<Color>> = Vec::with_capacity(size);
        new_buffer.resize(size, None);

        Self {  world: Arc::new(world),
                view: view,
                result_buffer: Arc::new(Mutex::new(new_buffer)) }
    }

    fn map_pixel_to_buffer(&self, pixel: Point2Int) -> Option<usize> {
        let screen = self.view.get_screen();
        if let Ok(result) = screen.get_pixel_index_by_screen_coord(&pixel) {
            if result < 0 {
                panic!("Negative value as screen pixel index");
            }
            Some(result as usize)
        } else {
            None
        } 
    }
}

impl<WorldT> RayCaster for WorldView<WorldT>
    where WorldT: RayCaster + IlluminationCaster + Send + Sync
{
    fn cast_ray(&self, ray: &Ray) -> Option<Color> {
        self.world.cast_ray(ray)
    }

    fn cast_colored_light_ray(&self, ray: &Ray, intersection: &RayIntersection) -> Option<Color> {
        self.world.cast_colored_light_ray(ray, intersection)
    }

    fn cast_model_ray(&self, ray: &Ray) -> Option<RayIntersection> {
        self.world.cast_model_ray(ray)
    }
}

impl<WorldT> IlluminationCaster for WorldView<WorldT>
    where WorldT: RayCaster + IlluminationCaster + Send + Sync
{
    fn get_illumination_at(&self, intersection: &RayIntersection) -> Vec<LightIntersection> {
        self.world.get_illumination_at(intersection)
    }
}

impl<WorldT> Scene for WorldView<WorldT>
    where WorldT: RayCaster + IlluminationCaster + Send + Sync
{
    fn get_pixel_color(&self, pixel: Point2Int) -> Result<Color, SceneError> {
        if let Ok(ray) = self.view.get_ray_to_screen_coordinate(pixel) {
            match self.world.cast_ray(&ray) {
                Some(color) => Ok(color),
                None => Err(SceneError::NothingIntersected),
            }
        } else {
            Err(SceneError::InvalidInputCoord)
        }
    }

    fn get_pixel_intersection(&self, pixel: Point2Int) -> Result<RayIntersection, SceneError> {
        if let Ok(ray) = self.view.get_ray_to_screen_coordinate(pixel) {
            match self.world.cast_model_ray(&ray) {
                Some(intersection) => Ok(intersection),
                None => Err(SceneError::NothingIntersected),
            }
        } else {
            Err(SceneError::InvalidInputCoord)
        }
    }

    fn get_view(&self) -> &View {
        &self.view
    }

    fn get_ray_caster(&self) -> &RayCaster {
        self
    }

    fn get_illumination_caster(&self) -> &IlluminationCaster {
        self
    }
}

impl<WorldT> SceneBuffer for WorldView<WorldT>
    where WorldT: RayCaster + IlluminationCaster + Send + Sync
{
    fn set_pixel_value(&self, pixel: Point2Int, color: &Color) -> Result<(), SceneBufferError> {
        if let Some(index) = self.map_pixel_to_buffer(pixel) {
            if let Ok(ref mut result_buffer_acessor) = self.result_buffer.lock() {
                let buffer_item = result_buffer_acessor.get_mut(index).expect(&format!("set_pixel_value map_pixel_to_buffer should make this impossible. Index: {}", index));
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
            if let Ok(ref mut result_buffer_acessor) = self.result_buffer.lock() {
                let buffer_item = result_buffer_acessor.get_mut(index).expect(&format!("accumulate_pixel_value map_pixel_to_buffer should make this impossible. Index: {}", index));
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
            if let Ok(ref mut result_buffer_acessor) = self.result_buffer.lock() {
                let buffer_item = result_buffer_acessor.get_mut(index).expect(&format!("reset_pixel map_pixel_to_buffer should make this impossible. Index: {}", index));
                *buffer_item = None;
                Ok(())
            } else {
                Err(SceneBufferError::MutexLockError)
            }
        } else {
            Err(SceneBufferError::InvalidInputCoord)
        }
    }

    fn get_pixel_value(&self, pixel: Point2Int) -> Result<Option<Color>, SceneBufferError> {
        if let Some(index) = self.map_pixel_to_buffer(pixel) {
            if let Ok(ref mut result_buffer_acessor) = self.result_buffer.lock() {
                let buffer_item = result_buffer_acessor.get_mut(index).expect(&format!("get_pixel_value map_pixel_to_buffer should make this impossible. Index: {}", index));
                Ok(*buffer_item)
            } else {
                Err(SceneBufferError::MutexLockError)
            }
        } else {
            Err(SceneBufferError::InvalidInputCoord)
        }
    }
}

impl<WorldT> WorldViewTrait for WorldView<WorldT>
    where WorldT: RayCaster + IlluminationCaster + Send + Sync
{

}