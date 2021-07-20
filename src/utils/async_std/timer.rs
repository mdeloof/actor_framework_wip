use crate::event::*;
use crate::sender::*;
use std::time::Duration;
use std::fmt;
use futures::future::{AbortHandle, Abortable};
use async_std::task;


#[derive(Clone)]
pub struct Timer<E: IsEvent<Event = E>> {
    sender_component: SenderComponent<E>,
    abort_handle: Option<AbortHandle>,
    pub duration: Duration,
    pub on_elapsed: fn(&SenderComponent<E>)
}

impl<E: IsEvent<Event = E>> Timer<E> {

    pub fn new(duration: Duration) -> Self {
        Self {
            sender_component: Default::default(),
            abort_handle: None,
            duration,
            on_elapsed: |_this| {}
        }
    }

    /// Schedule the timer to run once.
    pub fn start(&mut self) {
        // Cancel the timer if was already running
        self.cancel();
        // Create new variables that will be moved into the future
        let duration = self.duration;
        let on_elapsed = self.on_elapsed;
        let sender_component = self.get_sender_component().clone();
        // Create the future
        let task = async move {
            let test = sender_component;
            task::sleep(duration).await;
            on_elapsed(&test);
        };
        // We want to be able to abort the future
        let (abort_handle, abort_registration) = AbortHandle::new_pair();
        self.abort_handle = Some(abort_handle);
        let abortable_task = Abortable::new(task, abort_registration);
        task::spawn(abortable_task);
    }

    /// Schedule the timer to run repeatedly.
    pub fn start_interval(&mut self) {
        // Cancel the timer if was already running
        self.cancel();
        // Create new variables that will be moved into the future
        let duration = self.duration;
        let on_elapsed = self.on_elapsed;
        let sender_component = self.get_sender_component().clone();
        task::spawn(async move {
            let sender_component = sender_component.clone();
            loop {
                task::sleep(duration).await;
                on_elapsed(&sender_component);
            }
        });
    }

    /// Cancel the timer if it was running.
    pub fn cancel(&mut self) {
        if let Some(abort_handle) = &mut self.abort_handle {
            abort_handle.abort();
        }
    }

}

impl<E: IsEvent<Event = E>> Sender for Timer<E> {
    type Event = E;

    fn get_sender_component_mut(&mut self) -> &mut SenderComponent<Self::Event> {
        &mut self.sender_component
    }

    fn get_sender_component(&self) -> &SenderComponent<Self::Event> {
        &self.sender_component
    }

}

impl<E: IsEvent<Event = E>> Default for Timer<E> {

    fn default() -> Self {
        Self {
            sender_component: Default::default(),
            abort_handle: None,
            duration: Duration::from_secs(1),
            on_elapsed: |_this| {}
        }
    }
}

impl<E: IsEvent<Event = E>> fmt::Debug for Timer<E> {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_struct("Timer")
         .field("duration", &self.duration)
         .finish()
    }
}