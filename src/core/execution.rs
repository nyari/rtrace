pub trait ThreadSafeIterator: Send + Sync {
    type Item;

    fn next(&self) -> Option<Self::Item>;
}

pub trait RenderingTask: Send + Sync {
    fn execute(self: Box<Self>);
}

pub trait RenderingTaskProducer: Send + Sync {
    fn create_task_iterator(&self) -> Box<ThreadSafeIterator<Item=Box<RenderingTask>>>;
}



// pub struct OrderedTaskProducers {
//     producers: Vec<Box<RenderingTaskProducer>>
// }

// impl OrderedTaskProducers {
//     pub fn new(producers: [Box<RenderingTaskProducer>]) -> Self {
//         Self {
//             producers: producers
//         }
//     }
// }

// impl RenderingTaskProducer for OrderedTaskProducers {
//     fn create_task_iterator(&self) -> Iterator<Item=Box<RenderingTask>> {

//     }
// }