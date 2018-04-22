use std::collections::{HashSet};
use std::sync::{Arc, Mutex};
use std;

use defs::{IntType, FloatType, Point2Int};
use core::{WorldViewTrait, Color, Screen, ScreenIterator, RayIntersection, 
           RayPropagator, BasicSceneBuffer, SceneBuffer,
           RenderingTask, RenderingTaskProducer, ThreadSafeIterator};

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

#[derive(Debug)]
pub enum GlobalIlluminationShaderError {
    InvalidCoord,
    MutexLockError,
    NotExistingModelId
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
    maximum_pitch_angle: FloatType,
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
            maximum_pitch_angle: max_yaw_angle,
            state: Mutex::new(GlobalIlluminationShaderState::new(buffer))
        }
    }

    pub fn get_screen(&self) -> &Screen {
        self.worldview.get_view().get_screen()
    }

    fn to_buffer_index(&self, coord: &Point2Int) -> usize {
        self.get_screen().get_pixel_index_by_screen_coord(coord).expect("GlobalIlluminationShader::to_buffer_index coord should always be valud") as usize
    }

    fn calculate_global_color_for_intersection(&self, intersection: &RayIntersection) -> Option<Color> {
        let propagator = RayPropagator::new(intersection);
        let mut result: Option<Color> = None;
        let mut random_generator = rand::thread_rng();
        let mut actually_sampled = 0;
        for _counter in 0..self.sampling_size {
            let pitch: FloatType = self.maximum_pitch_angle * random_generator.gen::<FloatType>();
            let yaw: FloatType = 2.0 * std::f64::consts::PI * random_generator.gen::<FloatType>();
            if let Ok(ray) = propagator.get_diffuse_direction_ray(pitch, yaw) {
                if let Some(new_color) = self.worldview.get_ray_caster().cast_ray(&ray) {
                    if let Some(ref mut accumulated_color) = result{ 
                        *accumulated_color += new_color;
                        actually_sampled += 1;
                    } else {
                        result = Some(new_color)
                    }
                }
            }
        }

        match result {
            Some(color) => Some(color.mul_scalar(&(actually_sampled as FloatType).recip())),
            None => None
        }
    }

    pub fn new_calculate_uuid_color_for_pixel(&self, coord: &Point2Int) -> Result<Option<UuidColor>, GlobalIlluminationShaderError> {
        if let Ok(intersection) = self.worldview.get_pixel_intersection(*coord) {
            if let Some(model_identifier) = intersection.get_model_identifier() {
                if let Some(resulting_color) = self.calculate_global_color_for_intersection(&intersection) {
                    Ok(Some(UuidColor::new(*model_identifier, resulting_color)))
                } else {
                    Ok(None)
                }
            } else {
                Err(GlobalIlluminationShaderError::MutexLockError)
            }
        } else {
            Err(GlobalIlluminationShaderError::InvalidCoord)
        }
    }

    pub fn set_to_buffer(&self, coord: &Point2Int, entry: UuidColor) -> Result<(), GlobalIlluminationShaderError> {
        if let Ok(ref mut unlocked_state) = self.state.lock() {
            unlocked_state.set_color(self.to_buffer_index(coord), entry);
            Ok(())
        } else {
            Err(GlobalIlluminationShaderError::MutexLockError)
        }
    }

    pub fn get_entire_buffer(&self) -> Result<Box<SceneBuffer>, GlobalIlluminationShaderError> {
        if let Ok(unlocked_state) = self.state.lock() {
            let transformed_buffer = unlocked_state.buffer.iter()
                                                          .map(|&uuid_color_option| uuid_color_option.map(|uc| uc.color))
                                                          .collect();
            Ok(Box::new(BasicSceneBuffer::with_buffer(*self.worldview.get_screen(), transformed_buffer).unwrap()))
        } else {
            Err(GlobalIlluminationShaderError::MutexLockError)
        }
    }

    pub fn get_model_buffer(&self, model_unid: Uuid) -> Result<Box<SceneBuffer>, GlobalIlluminationShaderError> {
        if let Ok(unlocked_state) = self.state.lock() {
            let transformed_buffer = unlocked_state.buffer.iter()
                                                          .map(|&uuid_color_option| {
                                                              match uuid_color_option {
                                                                  Some(uuidcolor) => {
                                                                      if model_unid == uuidcolor.id {
                                                                          Some(uuidcolor.color)
                                                                      } else {
                                                                          None
                                                                      }
                                                                  },
                                                                  None => None,
                                                              }
                                                          })
                                                          .collect();
            Ok(Box::new(BasicSceneBuffer::with_buffer(*self.worldview.get_screen(), transformed_buffer).unwrap()))
        } else {
            Err(GlobalIlluminationShaderError::MutexLockError)
        }
    }

    pub fn get_all_model_ids_on_buffer(&self) -> Result<HashSet<Uuid>, GlobalIlluminationShaderError> {
        if let Ok(unlocked_state) = self.state.lock() {
            Ok(unlocked_state.visible.clone())
        } else {
            Err(GlobalIlluminationShaderError::MutexLockError)
        }
    }
}

pub struct GlobalIlluminationShaderTaskProducer {
    shader: Arc<GlobalIlluminationShader>
}

impl GlobalIlluminationShaderTaskProducer {
    pub fn new(shader: Arc<GlobalIlluminationShader>) -> Box<RenderingTaskProducer> {
        Box::new(Self {
            shader: shader
        })
    }   
}

impl RenderingTaskProducer for GlobalIlluminationShaderTaskProducer {
    fn create_task_iterator(self: Box<Self>) -> Box<ThreadSafeIterator<Item=Box<RenderingTask>>> {
        Box::new(GlobalIlluminationShaderTaskIterator::new(Arc::clone(&self.shader)))
    }
}

pub struct GlobalIlluminationShaderTaskIterator {
    shader: Arc<GlobalIlluminationShader>,
    screen_iterator: Mutex<ScreenIterator>
}

impl GlobalIlluminationShaderTaskIterator {
    pub fn new(shader: Arc<GlobalIlluminationShader>) -> Self {     
        let screen_iterator = ScreenIterator::new(shader.get_screen());
        Self {
            shader: shader,
            screen_iterator: Mutex::new(screen_iterator),
        }
    }

    fn create_task(&self, coord: Point2Int) -> Box<GlobalIlluminationShaderTask> {
        Box::new(GlobalIlluminationShaderTask::new(Arc::clone(&self.shader), coord))
    }
}

impl ThreadSafeIterator for GlobalIlluminationShaderTaskIterator {
    type Item = Box<RenderingTask>;

    fn next(&self) -> Option<Box<RenderingTask>> {
        if let Ok(ref mut unlocked_screen_iterator) = self.screen_iterator.lock() { 
            match unlocked_screen_iterator.next() {
                Some(coord) => Some(self.create_task(coord)),
                None => None
            }
        } else {
            panic!("Mutex lock error inside WorldViewTaskIterator");
        }
    }
}

pub struct GlobalIlluminationShaderTask {
    shader: Arc<GlobalIlluminationShader>,
    coord: Point2Int
}

impl GlobalIlluminationShaderTask {
    pub fn new(shader: Arc<GlobalIlluminationShader>, coord: Point2Int) -> Self {
        Self {
            shader: shader,
            coord: coord
        }
    }
}

impl RenderingTask for GlobalIlluminationShaderTask {
    fn execute(self: Box<Self>) {
        match self.shader.new_calculate_uuid_color_for_pixel(&self.coord) {
            Ok(Some(uuidcolor)) => self.shader.set_to_buffer(&self.coord, uuidcolor).expect("WorldViewTask: There should be no buffer error$"),
            Ok(_) => (),
            Err(error) => panic!("WorldViewTask: Unrecoverable SceneError: {:?}", error)
        }
    }
}