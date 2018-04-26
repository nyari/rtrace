use defs::{Point2Int};
use core::{RayCaster, IlluminationCaster, View, Color, RayIntersection, Screen, Ray, LightIntersection, 
           Scene, SceneError, BasicSceneBuffer, SceneBuffer, MutableSceneBuffer, ImmutableSceneBuffer, SceneBufferError};
use std::sync::{Arc};


pub trait WorldViewTrait: Scene + SceneBuffer {

}

pub struct WorldView<WorldT> {
    world: Arc<WorldT>,
    view: View,
    result_buffer: BasicSceneBuffer,
}

impl<WorldT> WorldView<WorldT>
    where WorldT: RayCaster + IlluminationCaster + Send + Sync
{
    #[allow(dead_code)]
    pub fn new(world: WorldT, view: View) -> Self {
        let screen_clone = view.get_screen().clone();
        Self {  world: Arc::new(world),
                view: view,
                result_buffer: BasicSceneBuffer::new(screen_clone) }
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

impl<WorldT> ImmutableSceneBuffer for WorldView<WorldT>
    where WorldT: RayCaster + IlluminationCaster + Send + Sync
{
    fn get_pixel_value(&self, pixel: Point2Int) -> Result<Option<Color>, SceneBufferError> {
        self.result_buffer.get_pixel_value(pixel)
    }

    fn get_screen(&self) -> &Screen {
        self.result_buffer.get_screen()
    }
}

impl<WorldT> MutableSceneBuffer for WorldView<WorldT>
    where WorldT: RayCaster + IlluminationCaster + Send + Sync
{
    fn set_pixel_value(&self, pixel: Point2Int, color: &Color) -> Result<(), SceneBufferError> {
        self.result_buffer.set_pixel_value(pixel, color)
    }

    fn accumulate_pixel_value(&self, pixel: Point2Int, color: &Color) -> Result<(), SceneBufferError> {
        self.result_buffer.accumulate_pixel_value(pixel, color)
    }

    fn reset_pixel(&self, pixel: Point2Int) -> Result<(), SceneBufferError> {
        self.result_buffer.reset_pixel(pixel)
    }
}

impl<WorldT> SceneBuffer for WorldView<WorldT>
    where WorldT: RayCaster + IlluminationCaster + Send + Sync {

}

impl<WorldT> WorldViewTrait for WorldView<WorldT>
    where WorldT: RayCaster + IlluminationCaster + Send + Sync
{

}