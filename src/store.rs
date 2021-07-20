use crate::sender::*;
use crate::event::*;
use std::ops::Deref;
use std::fmt;

#[derive(Clone)]
pub struct Store<T, E>
where
T: Default + Clone + fmt::Debug,
E: IsEvent<Event = E> {
    sender_component: SenderComponent<E>,
    pub value: T,
    pub on_mutate: fn(&Self)
}

impl<T, E> Store<T, E>
where
T: Default + Clone + fmt::Debug,
E: IsEvent<Event = E> {

    pub fn new(value: T) -> Self {
        Self {
            sender_component: Default::default(),
            value,
            on_mutate: |_this| {}
        }
    }

    pub fn mutate(&mut self, value: T) {
        self.value = value;
        (self.on_mutate)(self);
    }
}

impl<T, E> Default for Store<T, E> 
where
T: Default + Clone + fmt::Debug,
E: IsEvent<Event = E> {
    
    fn default() -> Self {
        Self {
            sender_component: Default::default(),
            value: Default::default(),
            on_mutate: |_this| {}
        }
    }
}

impl<T, E> fmt::Debug for Store<T, E>
where
T: Default + Clone + fmt::Debug,
E: IsEvent<Event = E> {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Store")
         .field("sender_component", &self.sender_component)
         .field("value", &self.value)
         .finish()
    }
}

impl<T, E> Sender for Store<T, E>
where
T: Default + Clone + fmt::Debug,
E: IsEvent<Event = E> {
    type Event = E;

    fn get_sender_component_mut(&mut self) -> &mut SenderComponent<Self::Event> {
        &mut self.sender_component
    }

    fn get_sender_component(&self) -> &SenderComponent<Self::Event> {
        &self.sender_component
    }
}

impl<T, E> Deref for Store<T, E>
where
T: Default + Clone + fmt::Debug,
E: IsEvent<Event = E> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}