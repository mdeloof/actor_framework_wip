use std::{fmt::Debug, hash::Hash};

use crate::actor::ActorObject;

#[derive(Clone, Copy, Debug)]
pub enum Destination {
    All,
    Single(usize),
}

#[derive(Clone, Copy, Debug)]
pub enum Origin {
    Anonymous,
    Actor(usize),
}

/// Envelope wraps an event and defines its destination.
#[derive(Clone, Debug)]
pub struct Envelope<M: Message> {
    pub origin: Origin,
    pub destination: Destination,
    pub message: M,
}

/// Trait that must be implemented on the event enum.
pub trait Message
where
    Self: Sized + Send,
{
    type MessageType: MessageType<Message = Self>;
}

pub trait MessageType
where
    Self: Eq + Hash + Copy + Send + Clone + for<'a> From<&'a Self::Message>,
{
    type Message: Message<MessageType = Self>;
}
