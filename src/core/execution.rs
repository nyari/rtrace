
pub trait RenderingTask: Send + Sync {
    fn execute(self: Box<Self>);
}

pub trait RenderingTaskProducer: Send + Sync {
    fn create_task_iterator() -> Iterator<Item=Box<RenderingTask>>;
}