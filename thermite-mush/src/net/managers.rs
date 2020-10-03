use legion::*;

use mio::prelude::*;

#[derive(Debug)]
pub struct PollSystem {
    pub waiting: HashMap<Token, Entity>,
    pub counter: usize,
    pub poll: Poll
    pub events: Events
}

impl PollSystem {
    pub fn register(&mut self, src: &impl Source, entity: Entity, interest: Interest) -> std::io::Result {
        self.counter = self.counter += 1;
        let token = Token(self.counter);
        self.poll.registry().register(src, token, interest)?;
        self.waiting.insert(token, entity);
        let avail = self.waiting.len() - self.events.capacity();
        if avail < 10 {
            self.events = Events::with_capacity(self.waiting.len() + 20);
        }
    }

    pub fn poll(&mut self) -> std::io::Result {
        self.poll.poll(&self.events, Some(std::time::Duration::zero()))?
    }
}

impl Default for PollSystem {
    fn default() -> Self {
        Self {
            waiting: Default::default(),
            counter: 0,
            poll: Default::default(),
            events: Events::with_capacity(15)
        }
    }
}

#[derive(Debug, Default)]
pub struct NetManager {
    pub listeners: PollSystem,
    pub readers: PollSystem,
    pub writers: PollSystem
}