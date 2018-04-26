use defs::{IntType}
use core::{SceneBuffer, BasicSceneBuffer};

struct MedianFilter<'obuffer> {
    original_buffer: &'obuffer SceneBuffer,
    rect_radius: IntType,
    result_buffer: BasicSceneBuffer
}

impl<'obuffer> MedianFilter<'obuffer> {
    pub fn new(original_buffer: &SceneBuffer, rect_radius: IntType)
}