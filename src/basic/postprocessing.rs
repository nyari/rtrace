use std::collections::{HashSet};
use std::sync::{Arc, Mutex};
use std;

use defs::{IntType, FloatType, Point2Int};
use core::{WorldViewTrait, Color, Screen, RayIntersection, RayPropagator};

use uuid::{Uuid};
use rand;
use rand::{Rng};

#[derive(Clone, Copy)]
pub struct UuidColor {
    pub id: Uuid,
    pub color: Color
}

impl UuidColor {
    pub fn new(id: Uuid, color: Color) -> Self {
        Self {
            id: id,
            color: color
        }
    }
}

struct GlobalIlluminationShaderState {
    pub buffer: Vec<Option<UuidColor>>,
    pub visible: HashSet<Uuid>
}

impl GlobalIlluminationShaderState {
    pub fn new(input: Vec<Option<UuidColor>>) -> Self {
        Self {
            buffer: input,
            visible: HashSet::new()
        }
    }

    pub fn set_color(&mut self, index: usize, input: UuidColor) {
        self.visible.insert(input.id.clone());
        self.buffer[index] = Some(input);
    }
}

pub struct GlobalIlluminationShader {
    worldview: Arc<WorldViewTrait>,
    sampling_size: IntType,
    maximum_yaw_angle: FloatType,
    state: Mutex<GlobalIlluminationShaderState>
}

impl GlobalIlluminationShader {
    pub fn new(worldview: Arc<WorldViewTrait>, sampling_size: IntType, max_yaw_angle: FloatType) -> Self {
        let buffer_size = worldview.get_view().get_screen().get_pixel_count() as usize;
        let mut buffer: Vec<Option<UuidColor>> = Vec::with_capacity(buffer_size);
        buffer.resize(buffer_size, None);
        Self {
            worldview: worldview,
            sampling_size: sampling_size,
            maximum_yaw_angle: max_yaw_angle,
            state: Mutex::new(GlobalIlluminationShaderState::new(buffer))
        }
    }

    fn get_screen(&self) -> &Screen {
        self.worldview.get_view().get_screen()
    }

    fn to_buffer_index(&self, coord: &Point2Int) -> usize {
        self.get_screen().get_pixel_index_by_screen_coord(coord).expect("GlobalIlluminationShader::to_buffer_index coord should always be valud") as usize
    }

    fn calculate_global_color_for_intersection(&self, intersection: &RayIntersection) -> Option<Color> {
        let propagator = RayPropagator::new(intersection);
        let mut result: Option<Color> = None;
        let mut random_generator = rand::thread_rng();
        for _counter in 0..self.sampling_size {
            let yaw: FloatType = self.maximum_yaw_angle * random_generator.gen::<FloatType>();
            let pitch: FloatType = 2.0 * std::f64::consts::PI * random_generator.gen::<FloatType>();
            if let Ok(ray) = propagator.get_diffuse_direction_ray(pitch, yaw) {
                if let Some(new_color) = self.worldview.get_ray_caster().cast_ray(&ray) {
                    if let Some(ref mut accumulated_color) = result{ 
                        *accumulated_color += new_color;
                    } else {
                        result = Some(new_color)
                    }
                }
            }
        }

        result
    }

    pub fn new_calculate_uuid_color_for_pixel(&self, coord: &Point2Int) -> Option<UuidColor> {
        if let Ok(intersection) = self.worldview.get_pixel_intersection(*coord) {
            if let Some(model_identifier) = intersection.get_model_identifier() {
                if let Some(resulting_color) = self.calculate_global_color_for_intersection(&intersection) {
                    Some(UuidColor::new(*model_identifier, resulting_color))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn set_to_buffer(&self, coord: &Point2Int, entry: UuidColor) {
        if let Ok(mut unlocked_state) = self.state.lock() {
            unlocked_state.set_color(self.to_buffer_index(coord), entry);
        } else {
            panic!("Mutex lock error inside GlobalIlluminationShader");
        }
    }
}

// impl SceneBuffer for GlobalIlluminationShader {
//     fn set_pixel_value(&self, pixel: Point2Int, color: &Color) -> Result<(), SceneBufferError> {
//         let buffer_index = self.to_buffer_index();
//         if let Ok(ref mut unlocked_buffer) = self.output_buffer.lock() {
//             unlocked_buffer.get_mut().unwrap() = Some(*color);
//             Ok(())
//         } else {
//             Err(SceneBufferError::MutexLockError)
//         }
//     }

//     fn accumulate_pixel_value(&self, pixel: Point2Int, color: &Color) -> Result<(), SceneBufferError>
//     {
//         let buffer_index = self.to_buffer_index();
//         if let Ok(ref mut unlocked_buffer) = self.output_buffer.lock() {
//             let mut pixel_in_question = unlocked_buffer.get_mut().unwrap();
//             if Some(ref mut original_color) = pixel_in_question {
//                 original_color += color;
//             } else {
//                 pixel_in_question = Some(*color);
//             }
//             Ok(())
//         } else {
//             Err(SceneBufferError::MutexLockError)
//         }
//     }

//     fn reset_pixel(&self, pixel: Point2Int) -> Result<(), SceneBufferError> {
//         let buffer_index = self.to_buffer_index();
//         if let Ok(ref mut unlocked_buffer) = self.output_buffer.lock() {
//             unlocked_buffer.get_mut().unwrap() = None;
//             Ok(())
//         } else {
//             Err(SceneBufferError::MutexLockError)
//         }
//     }

//     fn get_pixel_value(&self, pixel: Point2Int) -> Result<Option<Color>, SceneBufferError> {
//         let buffer_index = self.to_buffer_index();
//         if let Ok(ref mut unlocked_buffer) = self.output_buffer.lock() {
//             Ok(unlocked_buffer.get().unwrap())
//         } else {
//             Err(SceneBufferError::MutexLockError)
//         }
//     }

//     fn get_screen(&self) -> &Screen {
//         self.worldview.get_view().get_screen()
//     }
// }