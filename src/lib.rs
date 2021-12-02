pub mod actor;
/// Armature is a framework to design event-driven systems with stateful
/// actors.
///
/// **Event driven**: every change in the system happens in response to an
/// external event such as a key-press or an internal event such as a timer
/// elapsing. These events are then dispatched to the different stators
/// that are part of the system.
///
/// **Stators**: actors that contain a hierarchial state machine that responds
/// to incoming events and are able to spawn tasks inside the async runtime.
pub mod commutator;
pub mod message;
pub mod publisher;
mod store;
pub mod utils;

pub use actor::{Actor, ActorObject};
pub use commutator::{Commutator, InterceptResult, Interceptor, Receiver, Sender};
pub use message::{Destination, Envelope, Message, MessageType, Origin};
pub use publisher::{DeputyPublisher, Publisher};

pub use armature_macro::MessageType;
