use crate::message::*;
use crate::{Publisher, Sender};
use async_std::task;
use futures::future::{AbortHandle, Abortable};
use std::fmt;
use std::time::Duration;

pub struct Timer<E: Message> {
    sender: Option<Sender<E>>,
    _id: Option<usize>,
    abort_handle: Option<AbortHandle>,
    pub duration: Duration,
    pub on_elapsed: fn(&Self),
}

impl<E: Message> Clone for Timer<E> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            _id: self._id.clone(),
            abort_handle: self.abort_handle.clone(),
            duration: self.duration.clone(),
            on_elapsed: self.on_elapsed.clone(),
        }
    }
}

impl<E: Message + 'static> Timer<E> {
    pub fn new(duration: Duration) -> Self {
        Self {
            sender: None,
            _id: None,
            abort_handle: None,
            duration,
            on_elapsed: |_this| {},
        }
    }

    /// Schedule the timer to run once.
    pub fn start(&mut self) {
        // Cancel the timer if was already running
        self.cancel();
        let this = self.clone();
        // Create the future
        let task = async move {
            task::sleep(this.duration).await;
            (this.on_elapsed)(&this);
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
        let this = self.clone();
        task::spawn(async move {
            loop {
                task::sleep(this.duration).await;
                (this.on_elapsed)(&this);
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

impl<E> Publisher for Timer<E>
where
    E: Message,
{
    type Message = E;

    fn sender(&self) -> &Sender<E> {
        match &self.sender {
            Some(sender) => sender,
            None => panic!(),
        }
    }
}

impl<E: Message> Default for Timer<E> {
    fn default() -> Self {
        Self {
            sender: None,
            _id: None,
            abort_handle: None,
            duration: Duration::from_secs(1),
            on_elapsed: |_this| {},
        }
    }
}

impl<E: Message> fmt::Debug for Timer<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_struct("Timer")
            .field("duration", &self.duration)
            .finish()
    }
}
