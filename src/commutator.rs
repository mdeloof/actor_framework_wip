use futures::channel::mpsc;
use futures::stream::StreamExt;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::collections::HashSet;

use crate::actor::*;
use crate::message::*;

pub type Sender<M> = mpsc::UnboundedSender<Envelope<M>>;
pub type Receiver<M> = mpsc::UnboundedReceiver<Envelope<M>>;

/// The commutator dispatches events to the actors attached to it.
pub struct Commutator<E, S>
where
    E: Message<MessageType = S>,
    S: MessageType<Message = E>,
{
    message_sender: Sender<E>,
    message_receiver: Receiver<E>,
    handlers: HashMap<usize, ActorObject<E>>,
    message_map: HashMap<S, HashSet<usize>>,
}

impl<E, S> Commutator<E, S>
where
    E: Message<MessageType = S>,
    S: MessageType<Message = E>,
{
    pub fn new() -> Commutator<E, S> {
        let (message_sender, message_receiver) = mpsc::unbounded::<Envelope<E>>();
        // Every event is a key in the hashmap, the value is a hashset of
        // all the event handlers that are subscribed to that event.
        let event_map: HashMap<S, HashSet<usize>> = HashMap::new();
        Commutator {
            message_sender,
            message_receiver,
            handlers: HashMap::new(),
            message_map: event_map,
        }
    }

    /// Run the commutator. Messages inside the channel will be read and
    /// dispatched to the attached actors.
    pub async fn run(&mut self) {
        self.init();
        loop {
            let envelope = self.message_receiver.next().await;
            if let Some(envelope) = envelope {
                let Envelope {
                    origin,
                    destination,
                    message,
                } = envelope;

                // There are few special events that are handled by the
                // commutator itself instead of being dispatched to the
                // attached event handlers, namely the `Attach` and `Detach`
                // events that are used for attaching and detaching actors
                // dynamically.
                match message {
                    // Stop running the commutator by breaking out of the loop.
                    message if S::from(&message).is_terminate_sig() => break,

                    // Attach new actors to the commutator.
                    message if S::from(&message).is_attach_sig() => {
                        let id = self.attach(message.match_attach_message().unwrap());
                        println!("Attached {}", id);
                        self.handlers
                            .get_mut(&id)
                            .expect("handler was not inserted")
                            .init();
                    }

                    // Detach actors from the commutator.
                    message if S::from(&message).is_detach_sig() => {
                        let id = message.match_detach_message().unwrap();
                        println!("Detached {}", id);
                        self.detach(id);
                    }

                    // All other events are dispatched to the attached event
                    // handlers according to the destination defined in the
                    // event envelope.
                    message => match destination {
                        Destination::Single(id) => {
                            if let Some(handler) = self.get_handler(id) {
                                let envelope = Envelope {
                                    origin,
                                    message,
                                    destination,
                                };
                                handler.handle(&envelope);
                            }
                        }
                        Destination::All => {
                            let envelope = Envelope {
                                origin,
                                message,
                                destination,
                            };
                            self.dispatch(&envelope);
                        }
                    },
                }
            }
            if self.handlers.is_empty() {
                break;
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
    fn dispatch(&mut self, envelope: &Envelope<E>) {
        let handlers = match self.message_map.get(&S::from(&envelope.message)) {
            Some(handlers) => handlers,
            None => return,
        };

        for handler_id in handlers.into_iter() {
            self.handlers.get_mut(handler_id).unwrap().handle(&envelope);
        }
    }

    /// Attach an event handler to the commutator
    pub fn attach(&mut self, mut actor: Box<dyn Actor<Message = E>>) -> usize {
        // The id of an event handler is the memory address of the Box that
        // contains it.
        let id = actor.id();
        actor.on_attach(&self.message_sender);
        let default_subscriptions = actor.default_subscriptions();
        for sig in default_subscriptions {
            match self.message_map.entry(sig) {
                Entry::Occupied(mut entry) => {
                    entry.get_mut().insert(id);
                }
                Entry::Vacant(entry) => {
                    let mut subscribers = HashSet::new();
                    subscribers.insert(id);
                    entry.insert(subscribers);
                }
            }
            self.message_map.get_mut(&sig).unwrap().insert(id);
            actor.insert_subscription(sig);
        }
        self.handlers.insert(id, actor);
        return id;
    }

    /// Detach an event handler from the commutator.
    pub fn detach(&mut self, id: usize) -> Option<Box<dyn Actor<Message = E>>> {
        // Remove all the references to the handler in the event map
        for handler_ids in self.message_map.values_mut() {
            handler_ids.remove(&id);
        }
        if let Some(mut handler) = self.handlers.remove(&id) {
            handler.on_detach();
            return Some(handler);
        } else {
            return None;
        }
    }

    /// Get a mutable reference to an event handler.
    pub fn get_handler(&mut self, key: usize) -> Option<&mut Box<dyn Actor<Message = E>>> {
        self.handlers.get_mut(&key)
    }

    /// Publish an event to all handlers.
    pub fn publish(&mut self, event: E) {
        let envelope = Envelope {
            origin: Origin::Anonymous,
            destination: Destination::All,
            message: event,
        };
        self.message_sender.unbounded_send(envelope).unwrap();
    }

    /// Drain all the events that are currently in the receiver.
    pub fn drain(&mut self) -> Vec<Envelope<E>> {
        let mut events = Vec::new();
        loop {
            match self.message_receiver.try_next() {
                Ok(Some(event)) => events.push(event),
                Ok(None) => break,
                Err(_) => break,
            }
        }
        return events;
    }
}
