use core::{RenderingTaskProducer, RenderingTask, SceneError, Screen, WorldViewTrait};
use defs::{Point2Int, IntType};
use std::sync::{Arc};

pub struct WorldViewTaskProducer {
    worldview: Arc<WorldViewTrait>,
}

impl WorldViewTaskProducer {
    pub fn new(worldview: Arc<WorldViewTrait>) -> Self {
        Self {
            worldview: worldview
        }
    }   
}

impl RenderingTaskProducer for WorldViewTaskProducer {
    fn create_task_iterator(&self) -> Box<Iterator<Item=Box<RenderingTask>>> {
        Box::new(WorldViewTaskIterator::new(Arc::clone(&self.worldview)))
    }
}

pub struct WorldViewTaskIterator {
    worldview: Arc<WorldViewTrait>,
    screen: Screen,
    screen_pixel_index: IntType,
}

impl WorldViewTaskIterator {
    pub fn new(worldview: Arc<WorldViewTrait>) -> Self {
        let screen_clone = worldview.get_view().get_screen().clone();
        
        Self {
            worldview: worldview,
            screen: screen_clone,
            screen_pixel_index: 0
        }
    }

    fn create_task(&self, coord: Point2Int) -> Box<WorldViewTask> {
        Box::new(WorldViewTask::new(Arc::clone(&self.worldview), coord))
    }
}

impl Iterator for WorldViewTaskIterator {
    type Item = Box<RenderingTask>;

    fn next(&mut self) -> Option<Box<RenderingTask>> {
        let coord_result = self.screen.get_pixel_screen_coord_by_index(self.screen_pixel_index);
        self.screen_pixel_index += 1;

        if let Ok(coord) = coord_result {
            Some(self.create_task(coord))
        } else {
            None
        }
    }
}

pub struct WorldViewTask {
    worldview: Arc<WorldViewTrait>,
    coord: Point2Int
}

impl WorldViewTask {
    pub fn new(worldview: Arc<WorldViewTrait>, coord: Point2Int) -> Self {
        Self {
            worldview: worldview,
            coord: coord
        }
    }
}

impl RenderingTask for WorldViewTask {
    fn execute(self: Box<Self>) {
        match self.worldview.get_pixel_color(self.coord) {
            Ok(color) => self.worldview.accumulate_pixel_value(self.coord, &color).expect("WorldViewTask: There should be no buffer error$"),
            Err(SceneError::NothingIntersected) => (),
            Err(error) => panic!("WorldViewTask: Unrecoverable SceneError: {:?}", error)
        }
    }
}