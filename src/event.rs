use std::{fmt::Debug, hash::Hash};

use crate::handler::*;

#[derive(Clone, Copy, Debug)]
pub enum Destination {
    All,
    Single(usize)
}

/// Envelope wraps an event and defines its destination.
#[derive(Clone, Debug)]
pub struct Envelope<E: IsEvent> {
    pub destination: Destination,
    pub event: E
}

/// Trait that must be implemented on the event enum.
pub trait IsEvent: Clone + Send + Debug + 'static {
    type Event: IsEvent;
    type Sig: IsSig;
    

    fn new_exit_event() -> Self;

    fn new_entry_event() -> Self;
    
    fn new_nop_event() -> Self;

    fn new_terminate_event() -> Self;
    
    fn try_into_attach_event(self) -> Result<Box<(dyn Handler<Event = Self::Event, Sig = Self::Sig>)>, &'static str>;

    fn try_into_detach_event(self) -> Result<usize, &'static str>;

    fn as_sig(&self) -> Self::Sig;
    
}

pub trait IsSig: Debug + Eq + Hash + Send + Clone {
    type Sig: IsSig;

    fn list_all() -> std::vec::Vec<Self::Sig>;

    fn is_terminate_sig(&self) -> bool;

    fn is_attach_sig(&self) -> bool;

    fn is_detach_sig(&self) -> bool;

}

impl<E, S> Clone for Box<dyn Handler<Event = E, Sig = S>> {
    fn clone(&self) -> Box<dyn Handler<Event = E, Sig = S>> {
        todo!();
    }
}