use std::collections::HashSet;
use std::fmt::Debug;

use crate::message::*;
use crate::Sender;

pub type ActorObject<E> = Box<dyn Actor<Message = E>>;

impl<E: Message> Clone for ActorObject<E> {
    fn clone(&self) -> ActorObject<E> {
        todo!();
    }
}

pub trait Actor: Send {
    type Message: Message;

    /// Handle event
    fn handle(&mut self, _: &Envelope<Self::Message>);

    /// Lifecycle method that is called when the event handler is attached
    /// to the commutator. The `sender` can be cloned and used to send events
    /// to the commutator.
    fn on_attach(&mut self, _: &Sender<Self::Message>) {}

    /// Lifecycle method that is called when the event handler is detached from
    /// the commutator.
    fn on_detach(&mut self) {}

    /// Init method that is called when the commutator starts running, or if
    /// the commutator is already running right when the event handler is attached.
    fn init(&mut self) {}

    /// Deinit method that is called before the actor is detached.
    fn deinit(&mut self) {}

    /// Get the initial subscriptions of the event handler.
    fn default_subscriptions(&self) -> Vec<<Self::Message as Message>::MessageType> {
        Vec::new()
    }

    /// Get the id of the event handler.
    fn id(&self) -> usize {
        self as *const Self as *const () as usize
    }

    /// Insert a subscription.
    fn insert_subscription(&mut self, _sig: <Self::Message as Message>::MessageType) {}

    /// Remove a subscription from the `Actor`.
    fn remove_subscription(&mut self, _sig: <Self::Message as Message>::MessageType) {}
}

#[derive(Debug)]
pub struct HandlerComponent<S>
where
    S: MessageType,
{
    _id: Option<usize>,
    subscriptions: HashSet<S>,
}

impl<S> Default for HandlerComponent<S>
where
    S: MessageType,
{
    fn default() -> Self {
        Self {
            _id: None,
            subscriptions: HashSet::new(),
        }
    }
}

impl<S> Clone for HandlerComponent<S>
where
    S: MessageType,
{
    fn clone(&self) -> Self {
        Self {
            _id: None,
            subscriptions: self.subscriptions.clone(),
        }
    }
}
