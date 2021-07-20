use std::fmt::Debug;
use std::collections::HashSet;

use crate::event::*;
use crate::sender::*;

pub trait Handler: Sender + Send + Debug {
    type Sig: IsSig<Sig = Self::Sig> + Debug;

    /// Get a mutable reference to the event handler component.
    fn get_handler_component_mut(&mut self) -> &mut HandlerComponent<Self::Sig>;

    /// Get a immutable reference to the event handler component.
    fn get_handler_component(&self) -> &HandlerComponent<Self::Sig>;

    /// Lifecycle method that is called when the event handler is attached
    /// to the commutator.
    fn on_attach(&mut self) {}

    /// Lifecycle method that is called when the event handler is detached from
    /// the commutator.
    fn on_detach(&mut self) {}

    /// Init method that is called when the commutator starts running, or if
    /// the commutator is already running right when the event handler is attached.
    fn init(&mut self) {}

    /// Handle event
    fn handle(&mut self, _event: &Self::Event) {}

    /// Get the initial subscriptions of the event handler.
    fn get_init_subscriptions(&self) -> Vec<Self::Sig> {
        Vec::new()
    }

    /// Get the id of the event handler.
    fn get_id(&self) -> Option<usize> {
        self.get_handler_component().id
    }

    /// Set the id of the event handler.
    fn set_id(&mut self, id: usize) {
        self.get_handler_component_mut().id = Some(id);
    }

    /// Set the subscriptions of the event handler.
    fn set_subscriptions(&mut self, subscriptions: Vec<Self::Sig>) {
        for sig in subscriptions {
            self.get_handler_component_mut().subscriptions.insert(sig);
        }  
    }

    /// Attach event senders to the event handler. As a result, calling 
    /// `post_to_self()` from the event sender will send the event only to
    /// this event handler.
    fn attach(&self, sender: &mut dyn Sender<Event = Self::Event>) {
        let id = self.get_id().expect(
            "Event sender itself must be attached in order to attach event senders to it.");
        sender.set_sender(self.get_sender().clone());
        sender.set_associated_handler_id(id);
    }

}

#[derive(Debug)]
pub struct HandlerComponent<S>
where
S: IsSig<Sig = S> {
    id: Option<usize>,
    subscriptions: HashSet<S>
}

impl<S> Default for HandlerComponent<S>
where
S: IsSig<Sig = S> {

    fn default() -> Self {
        Self {
            id: None,
            subscriptions: HashSet::new()
        }
    }

}

impl<S> Clone for HandlerComponent<S>
where
S: IsSig<Sig = S> {

    fn clone(&self) -> Self {
        Self {
            id: None,
            subscriptions: self.subscriptions.clone()
        }
    }

}