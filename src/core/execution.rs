use std::sync::{Mutex};

pub trait ThreadSafeIterator: Send + Sync {
    type Item;

    fn next(&self) -> Option<Self::Item>;
}

pub trait RenderingTask: Send + Sync {
    fn execute(self: Box<Self>);
}

pub trait RenderingTaskProducer: Send + Sync {
    fn create_task_iterator(self: Box<Self>) -> Box<ThreadSafeIterator<Item=Box<RenderingTask>>>;
}

pub struct OrderedTaskProducers {
    producers: Option<Vec<Box<RenderingTaskProducer>>>
}

impl OrderedTaskProducers {
    pub fn new(producers: Vec<Box<RenderingTaskProducer>>) -> Box<RenderingTaskProducer> {
        Box::new(Self {
            producers: Some(producers)
        })
    }
}

impl RenderingTaskProducer for OrderedTaskProducers {
    fn create_task_iterator(mut self: Box<Self>) -> Box<ThreadSafeIterator<Item=Box<RenderingTask>>> {
        Box::new(OrderedTaskIterator::new(self.producers.take().expect("OrderedTaskProducers should have not been created empty")))
    }
}


struct OrderedTaskIteratorInternals {
    pub producers : Vec<Box<RenderingTaskProducer>>,
    pub current_terator : Option<Box<ThreadSafeIterator<Item=Box<RenderingTask>>>>
}

impl OrderedTaskIteratorInternals {
    pub fn new(mut producers : Vec<Box<RenderingTaskProducer>>) -> Self {
        producers.reverse();
        if let Some(first_producer) = producers.pop() {
            Self {
                producers: producers,
                current_terator: Some(first_producer.create_task_iterator())
            }
        } else {
            Self {
                producers: producers,
                current_terator: None
            }
        }
    }

    fn advance_producer(&mut self) {
        if let Some(next_producer) = self.producers.pop() {
            self.current_terator = Some(next_producer.create_task_iterator());
        } else {
            self.current_terator = None;
        }
    }

    pub fn next_internal(&mut self) -> Option<Box<RenderingTask>> {
        if self.current_terator.is_some() {
            if let Some(rendering_task) = self.current_terator.as_ref().unwrap().next() {
                Some(rendering_task)
            } else {
                self.advance_producer();
                self.next_internal()
            }
        } else {
            None
        }
    }
}

pub struct OrderedTaskIterator {
    internal_state : Mutex<OrderedTaskIteratorInternals>
}

impl OrderedTaskIterator {
    pub fn new(producers : Vec<Box<RenderingTaskProducer>>) -> Self {
        Self {
            internal_state : Mutex::new(OrderedTaskIteratorInternals::new(producers))
        }
    }
}

impl ThreadSafeIterator for OrderedTaskIterator {
    type Item = Box<RenderingTask>;

    fn next(&self) -> Option<Box<RenderingTask>> {
        if let Ok(ref mut internal_state) = self.internal_state.lock() {
            internal_state.next_internal()
        } else {
            panic!("Mutex lock error inside OrderedTaskIterator");
        }
    }
}
