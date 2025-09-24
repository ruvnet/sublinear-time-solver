//! Lock-free message bus for nano-agents

use crossbeam::queue::ArrayQueue;
use std::sync::Arc;

/// Message for inter-agent communication
#[derive(Clone, Debug)]
pub struct Message {
    pub topic: &'static str,
    pub data: MessageData,
    pub timestamp_ns: u128,
}

/// Message data variants
#[derive(Clone, Debug)]
pub enum MessageData {
    U64(u64),
    F64(f64),
    Bool(bool),
    Bytes([u8; 32]),
    Empty,
}

/// Lock-free message bus using fixed-size queue
pub struct NanoBus {
    queue: Arc<ArrayQueue<Message>>,
    capacity: usize,
}

impl NanoBus {
    /// Create a new message bus with given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            queue: Arc::new(ArrayQueue::new(capacity)),
            capacity,
        }
    }

    /// Publish a message (non-blocking, may drop if full)
    #[inline(always)]
    pub fn publish(&self, message: Message) -> bool {
        self.queue.push(message).is_ok()
    }

    /// Try to receive a message (non-blocking)
    #[inline(always)]
    pub fn try_recv(&self) -> Option<Message> {
        self.queue.pop()
    }

    /// Drain up to `max` messages without blocking
    #[inline(always)]
    pub fn drain(&self, max: usize) -> Vec<Message> {
        let mut messages = Vec::with_capacity(max.min(16));
        for _ in 0..max {
            match self.try_recv() {
                Some(msg) => messages.push(msg),
                None => break,
            }
        }
        messages
    }

    /// Get current queue length
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Check if queue is full
    pub fn is_full(&self) -> bool {
        self.queue.len() >= self.capacity
    }

    /// Clone the bus for sharing between agents
    pub fn clone_bus(&self) -> Self {
        Self {
            queue: self.queue.clone(),
            capacity: self.capacity,
        }
    }
}

impl Clone for NanoBus {
    fn clone(&self) -> Self {
        self.clone_bus()
    }
}

/// Topic-based message filter
pub struct TopicFilter {
    topics: Vec<&'static str>,
}

impl TopicFilter {
    pub fn new(topics: Vec<&'static str>) -> Self {
        Self { topics }
    }

    #[inline(always)]
    pub fn matches(&self, message: &Message) -> bool {
        self.topics.iter().any(|&t| t == message.topic)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nano_bus() {
        let bus = NanoBus::new(10);

        // Publish messages
        let msg1 = Message {
            topic: "test",
            data: MessageData::U64(42),
            timestamp_ns: 1000,
        };

        assert!(bus.publish(msg1.clone()));
        assert_eq!(bus.len(), 1);

        // Receive message
        let received = bus.try_recv().unwrap();
        assert_eq!(received.topic, "test");
        match received.data {
            MessageData::U64(val) => assert_eq!(val, 42),
            _ => panic!("Wrong message type"),
        }

        assert!(bus.is_empty());
    }

    #[test]
    fn test_bus_overflow() {
        let bus = NanoBus::new(2);

        let msg = Message {
            topic: "test",
            data: MessageData::Empty,
            timestamp_ns: 0,
        };

        assert!(bus.publish(msg.clone()));
        assert!(bus.publish(msg.clone()));
        assert!(!bus.publish(msg.clone())); // Should fail when full

        assert!(bus.is_full());
    }

    #[test]
    fn test_topic_filter() {
        let filter = TopicFilter::new(vec!["sensor", "control"]);

        let msg1 = Message {
            topic: "sensor",
            data: MessageData::Empty,
            timestamp_ns: 0,
        };

        let msg2 = Message {
            topic: "debug",
            data: MessageData::Empty,
            timestamp_ns: 0,
        };

        assert!(filter.matches(&msg1));
        assert!(!filter.matches(&msg2));
    }
}