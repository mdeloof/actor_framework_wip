use futures::channel::mpsc;
use crate::event::*;


/// Trait for sending events to the commutator.
pub trait Sender {
    type Event: IsEvent<Event = Self::Event>;

    /// Get a mutable reference to the event sender component.
    fn get_sender_component_mut(&mut self) -> &mut SenderComponent<Self::Event>;

    /// Get an immutable reference to the event sender component.
    fn get_sender_component(&self) -> &SenderComponent<Self::Event>;

    /// Set the event sender component.
    fn set_sender_component(&mut self, sender_component: SenderComponent<Self::Event>) {
        *self.get_sender_component_mut() = sender_component;
    }
    
    /// Get the event sender
    fn get_sender(&self) -> mpsc::UnboundedSender<Envelope<Self::Event>> {
        self.get_sender_component().sender.as_ref().unwrap().clone()
    }

    /// Set the event sender
    fn set_sender(&mut self, sender: mpsc::UnboundedSender<Envelope<Self::Event>>) {
        self.get_sender_component_mut().sender = Some(sender);
    }

    /// Clear the event sender
    fn clear_sender(&mut self) {
        self.get_sender_component_mut().sender = None;
    }

    /// Get associated event handler id, this is the id of the event handler 
    /// that owns the event sender.
    fn set_associated_handler_id(&mut self, id: usize) {
        self.get_sender_component_mut().associated_handler_id = Some(id);
    }

    /// Get the associated event handler id.
    fn get_associated_handler_id(&self) -> Option<usize> {
        self.get_sender_component().associated_handler_id
    }

    /// Clear the associated event handler id.
    fn clear_associated_handler_id(&mut self) {
        self.get_sender_component_mut().associated_handler_id = None;
    }

    /// Publish an event to all handlers.
    fn publish(&self, event: Self::Event) {
        let envelope = Envelope {
            destination: Destination::All,
            event
        };
        self.send(envelope);
    }

    /// Post an event to a specific handler.
    fn post(&self, event: Self::Event, handler_id: usize) {
        let envelope = Envelope {
            destination: Destination::Single(handler_id),
            event
        };
        self.send(envelope);
    }

    /// Post an event to event handler to which the event sender is attached.
    fn post_to_self(&self, event: Self::Event) {
        let id = self.get_associated_handler_id().expect(
            "Stator must be attached to commutator");
        self.post(event, id);
    }

    /// Send an envelope
    fn send(&self, envelope: Envelope<Self::Event>) {
        let sender_component = self.get_sender_component();
        if let Some(sender) = &sender_component.sender {
            let _ = sender.unbounded_send(envelope);
        } else {
            panic!("no event sender set");
        }
    }

}

#[derive(Clone, Debug)]
pub struct SenderComponent<E>
where
E: IsEvent<Event = E> {
    sender: Option<mpsc::UnboundedSender<Envelope<E>>>,
    associated_handler_id: Option<usize>
}

impl<E> Default for SenderComponent<E>
where 
E: IsEvent<Event = E> {

    fn default() -> Self {
        Self {
            sender: None,
            associated_handler_id: None
        }
    }

}

impl<E: IsEvent<Event = E>> Sender for SenderComponent<E> {
    type Event = E;

    fn get_sender_component_mut(&mut self) -> &mut SenderComponent<Self::Event> {
        self
    }

    fn get_sender_component(&self) -> &SenderComponent<Self::Event> {
        self
    }

}