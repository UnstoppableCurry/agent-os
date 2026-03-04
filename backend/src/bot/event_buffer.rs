use std::collections::VecDeque;

use crate::types::AgentEvent;

/// Ring buffer that retains the most recent N events for replay on new connections.
pub struct EventBuffer {
    events: VecDeque<AgentEvent>,
    capacity: usize,
}

impl EventBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            events: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Push an event into the buffer, evicting the oldest if at capacity.
    pub fn push(&mut self, event: AgentEvent) {
        if self.events.len() >= self.capacity {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    /// Return all cached events for replay.
    pub fn replay(&self) -> Vec<AgentEvent> {
        self.events.iter().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }
}
