use std::collections::HashMap;
use futures::channel::mpsc;
use futures::stream::StreamExt;


use crate::event::*;
use crate::stator::*;

/// The commutator dispatches events to the stators attached to it.

pub struct Commutator<E: Event> {
    event_sender: mpsc::UnboundedSender<Envelope<E>>,
    event_receiver: mpsc::UnboundedReceiver<Envelope<E>>,
    pub handlers: HashMap<usize, Box<dyn Handler<E>>>
}

impl<E: Event> Commutator<E> {

    pub fn new() -> Commutator<E> {
        let (event_sender, event_receiver) = mpsc::unbounded::<Envelope<E>>();
        Commutator {
            event_sender: event_sender,
            event_receiver: event_receiver,
            handlers: HashMap::new()
        }
    }

    pub async fn run(&mut self) {
        self.init();
        loop {
            match self.event_receiver.next().await {
                Some(Envelope { destination: Destination::Single(id), event }) => {
                    if let Some(handler) = self.get_handler(id) {
                        handler.handle(&event);
                    }
                },
                Some(Envelope { destination: Destination::All, event }) => {
                    self.dispatch(&event);
                }
                None => break
            };
        }
    }

    fn init(&mut self) {
        for handler in self.handlers.values_mut() {
            handler.init();
        }
    }

    fn dispatch(&mut self, event: &E) {
        for handler in self.handlers.values_mut() {
            handler.handle(event);
        }
    }

    pub fn add_handler(&mut self, mut handler: Box<dyn Handler<E>>) {
        let id = &*handler as *const dyn Handler<E> as *const () as usize;
        handler.set_event_sender(self.event_sender.clone());
        handler.set_id(id);
        self.handlers.insert(id, handler);
    }

    pub fn get_handler(&mut self, key: usize) -> Option<&mut Box<dyn Handler<E>>> {
        self.handlers.get_mut(&key)
    }

    pub fn publish(&mut self, event: E) {
        let envelope = Envelope {
            destination: Destination::All,
            event: event
        };
        self.event_sender.unbounded_send(envelope).unwrap();
    }

}