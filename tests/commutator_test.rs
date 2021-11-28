#[cfg(test)]
mod tests {

    use armature;
    use armature::Actor;
    use armature::MessageType;
    use armature::{Commutator, Sender};
    use armature::{Envelope, Publisher};
    use async_std::task::block_on;
    use std::vec::Vec;

    #[derive(Debug, MessageType)]
    #[message_type(name = "Signal")]
    pub enum Event {
        Detach(usize),
        Call(usize),
        Respond(usize),
    }

    impl armature::Message for Event {
        type MessageType = Signal;

        fn match_detach_message(self) -> Option<usize> {
            match self {
                Self::Detach(id) => Some(id),
                _ => None,
            }
        }
    }

    impl armature::MessageType for Signal {
        type Message = Event;

        fn is_detach_sig(&self) -> bool {
            match self {
                Signal::Detach => true,
                _ => false,
            }
        }
    }

    #[derive(Clone, Default, Debug)]
    struct Listener {
        _id: Option<usize>,
        sender: Option<Sender<Event>>,
        _subscriptions: Vec<Signal>,
        listeners: Vec<usize>,
    }

    impl Actor for Listener {
        type Message = Event;

        fn on_attach(&mut self, sender: &Sender<Self::Message>) {
            self.sender = Some(sender.clone());
        }

        fn init(&mut self) {
            self.publish(Event::Call(self.id()));
        }

        fn default_subscriptions(&self) -> Vec<<Self::Message as armature::Message>::MessageType> {
            vec![Signal::Respond, Signal::Call]
        }

        fn handle(&mut self, envelope: &Envelope<Event>) {
            match envelope.message {
                Event::Call(id) => {
                    println!("Called");
                    self.post(Event::Respond(self.id()), id);
                }
                Event::Respond(id) => {
                    println!("Received {}", id);
                    self.listeners.push(id);
                    if self.listeners.len() >= 3 {
                        self.publish(Event::Detach(self.id()));
                    }
                }
                _ => {}
            }
        }
    }

    impl Publisher for Listener {
        type Message = Event;

        fn sender(&self) -> &Sender<Event> {
            match &self.sender {
                Some(sender) => sender,
                None => panic!(),
            }
        }
    }

    #[test]
    fn commutator_sending() {
        let l1 = Listener::default();
        let l2 = Listener::default();
        let l3 = Listener::default();

        let mut commutator = Commutator::new();

        commutator.attach(Box::new(l1));
        commutator.attach(Box::new(l2));
        commutator.attach(Box::new(l3));

        let timeout = std::time::Duration::from_millis(1000);

        assert!(block_on(async_std::future::timeout(timeout, commutator.run())).is_ok());
        assert!(commutator.drain().len() == 0);
    }
}
