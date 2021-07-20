use std::collections::HashMap;
use std::collections::HashSet;
use futures::channel::mpsc;
use futures::stream::StreamExt;


use crate::event::*;
use crate::handler::*;

/// The commutator dispatches events to the stators attached to it.

pub struct Commutator<E, S>
where E: IsEvent<Event = E, Sig = S>, S: IsSig<Sig = S> {
    sender: mpsc::UnboundedSender<Envelope<E>>,
    event_receiver: mpsc::UnboundedReceiver<Envelope<E>>,
    pub handlers: HashMap<usize, Box<dyn Handler<Event = E, Sig = S>>>,
    pub event_map: HashMap<S, HashSet<usize>>
}

impl<E, S> Commutator<E, S>
where E: IsEvent<Event = E, Sig = S>, S: IsSig<Sig = S> {

    pub fn new() -> Commutator<E, S> {
        let (sender, event_receiver) = mpsc::unbounded::<Envelope<E>>();
        // Every event is a key in the hashmap, the value is a hashset of
        // all the event handlers that are subscribed to that event.
        let mut event_map: HashMap<S, HashSet<usize>> = HashMap::new();
        for event_sig in S::list_all().into_iter() {
            event_map.insert(event_sig, HashSet::new());
        }
        Commutator {
            sender,
            event_receiver,
            handlers: HashMap::new(),
            event_map
        }
    }

    pub async fn run(&mut self) {
        self.init();
        loop {
            if let Some(Envelope {destination, event}) = self.event_receiver.next().await {
                // There are few special events that are handled by the 
                // commutator itself instead of being dispatched to the
                // attached event handlers, namely the `Attach` and `Detach`
                // events that are used for attaching and detaching stators
                // dynamically.
                if event.as_sig().is_terminate_sig() {
                    break;
                } else if event.as_sig().is_attach_sig() {
                    let id = self.attach(event.try_into_attach_event().unwrap());
                    self.handlers.get_mut(&id).expect("handler was not inserted").init();
                } else if event.as_sig().is_detach_sig() {
                    self.detach(event.try_into_detach_event().unwrap());
                // All other events are dispatched to the attached event 
                // handlers according to the destination defined in the 
                // event envelope.
                } else {
                    match destination {
                        Destination::Single(id) => {
                            if let Some(handler) = self.get_handler(id) {
                                handler.handle(&event);
                            }
                        }
                        Destination::All => {
                            self.dispatch(&event);
                        }
                    }
                }
            }
        }
    }

    fn init(&mut self) {
        for handler in self.handlers.values_mut() {
            handler.init();
        }
    }

    /// Dispatch an event to all the attached event handlers who are 
    /// subscribed to the given event.
    fn dispatch(&mut self, event: &E) {
        for handler_id in self.event_map.get(&event.as_sig()).unwrap().into_iter() {
            self.handlers.get_mut(handler_id).unwrap().handle(event);
        } 
    }

    /// Attach an event handler to the commutator
    pub fn attach(&mut self, mut handler: Box<dyn Handler<Event = E, Sig = S>>) -> usize {
        // The id of an event handler is the memory address of the Box that
        // contains it.
        let id = &*handler as *const dyn Handler<Event = E, Sig = S> as *const () as usize;
        handler.set_sender(self.sender.clone());
        handler.set_id(id);
        handler.set_associated_handler_id(id);
        let init_subscriptions = handler.get_init_subscriptions();
        for sig in &init_subscriptions {
            self.event_map.get_mut(&sig).unwrap().insert(id);
        }
        handler.set_subscriptions(init_subscriptions);
        handler.on_attach();
        self.handlers.insert(id, handler);
        return id;
    }

    /// Detach an event handler from the commutator.
    pub fn detach(&mut self, id: usize) -> Option<Box<dyn Handler<Event = E, Sig = S>>> {
        // Remove all the references to the handler in the event map
        for handler_ids in self.event_map.values_mut() {
            handler_ids.remove(&id);
        }
        if let Some(mut handler) = self.handlers.remove(&id) {
            handler.on_detach();
            if self.handlers.len() == 0 {
                self.publish(E::new_terminate_event());
            }
            return Some(handler)
        } else {
            return None
        }
    }

    /// Get a mutable reference to an event handler.
    pub fn get_handler(&mut self, key: usize) -> Option<&mut Box<dyn Handler<Event = E, Sig = S>>> {
        self.handlers.get_mut(&key)
    }

    /// Publish an event to all handlers.
    pub fn publish(&mut self, event: E) {
        let envelope = Envelope {
            destination: Destination::All,
            event: event
        };
        self.sender.unbounded_send(envelope).unwrap();
    }

    /// Drain all the events that are currently in the receiver.
    pub fn drain(&mut self) -> Vec<Envelope<E>> {
        let mut events = Vec::new();
        loop {
            match self.event_receiver.try_next() {
                Ok(Some(event)) => events.push(event),
                Ok(None) => break,
                Err(_) => break
            }
        }
        return events
    }

}