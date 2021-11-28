use crate::message::*;
use crate::Actor;
use crate::Sender;
use futures::channel::mpsc;

/// Trait for sending events to the commutator.
pub trait Publisher {
    type Message: Message;

    /// Get the message sender.
    fn sender(&self) -> &Sender<Self::Message>;

    fn origin(&self) -> Origin {
        Origin::Anonymous
    }

    /// Publish a message to all actors.
    fn publish(&self, message: Self::Message) {
        let envelope = Envelope {
            origin: self.origin(),
            destination: Destination::All,
            message,
        };
        self.send(envelope);
    }

    /// Post a message to a specific actor.
    fn post(&self, message: Self::Message, actor_id: usize) {
        let envelope = Envelope {
            origin: self.origin(),
            destination: Destination::Single(actor_id),
            message,
        };
        self.send(envelope);
    }

    /// Send an envelope.
    fn send(&self, envelope: Envelope<Self::Message>) {
        self.sender()
            .unbounded_send(envelope)
            .expect("could not send envelope");
    }

    /// Create a deputy publisher that is associated with the current actor.
    fn deputy(&self) -> DeputyPublisher<<Self as Publisher>::Message>
    where
        Self: Actor<Message = <Self as Publisher>::Message> + Sized,
    {
        DeputyPublisher::from(self)
    }
}

impl<M: Message> Publisher for mpsc::UnboundedSender<Envelope<M>> {
    type Message = M;

    fn sender(&self) -> &Sender<Self::Message> {
        self
    }
}

/// A deputy publisher can be passed to a task, so it can publish messages in
/// name of the associated actor.
#[derive(Clone)]
pub struct DeputyPublisher<M>
where
    M: Message,
{
    sender: Sender<M>,
    actor_id: usize,
}

impl<M> DeputyPublisher<M>
where
    M: Message,
{
    pub fn actor_id(&self) -> usize {
        self.actor_id
    }
}

impl<A, M> From<&A> for DeputyPublisher<M>
where
    A: Actor<Message = M> + Publisher<Message = M>,
    M: Message,
{
    fn from(actor: &A) -> Self {
        Self {
            sender: actor.sender().clone(),
            actor_id: actor.id(),
        }
    }
}

impl<M> Publisher for DeputyPublisher<M>
where
    M: Message,
{
    type Message = M;

    fn sender(&self) -> &Sender<Self::Message> {
        &self.sender
    }

    fn origin(&self) -> Origin {
        Origin::Actor(self.actor_id)
    }
}
