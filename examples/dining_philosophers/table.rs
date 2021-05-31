use futures::channel::mpsc;
use armature::stator::*;

use crate::DPEvent;

#[derive(Clone, Copy)]
pub enum ForkState {
    Free,
    UsedLeft,
    UsedRight
}

pub struct Table {
    event_sender: mpsc::UnboundedSender<DPEvent>,
    count: usize,
    fork: Vec<ForkState>,
    is_hungry: Vec<bool>,
    stopped_number: usize
}

impl Table {

    pub fn new(sender: mpsc::UnboundedSender<DPEvent>, count: usize) -> Self {
        Self {
            event_sender: sender,
            count: count,
            fork: vec![ForkState::Free; count],
            is_hungry: vec![false; count],
            stopped_number: 0
        }
    }

    fn left(self, n: usize) -> usize {
        return (n + self.count - 1) % self.count
    }

    fn right(self, n: usize) -> usize {
        return (n + 1) % self.count
    }

    fn serving(&mut self, event: &DPEvent) -> Response<Self, DPEvent> {
        match *event {

            DPEvent::Hungry(n) => {
                assert!(n < self.count && !self.is_hungry[n]);
                let m = self.left(n);
                if let (ForkState::UsedLeft, ForkState::Free) = (self.fork[m], self.fork[n]) {
                    self.fork[m] = ForkState::UsedLeft;
                    self.fork[n] = ForkState::UsedRight;
                    self.emit(DPEvent::Eat(n));
                } else {
                    self.is_hungry[n] = true;
                }
                Response::Handled
            }

            DPEvent::Done(n) => {
                assert!(n < self.count);
                self.fork[self.left(n)] = ForkState::Free;
                self.fork[n] = ForkState::Free;
                let neighbor = self.right(n);
                if let (true, ForkState::Free) = (self.is_hungry[neighbor], self.fork[neighbor]) {
                    self.fork[n] = ForkState::UsedLeft;
                    self.fork[neighbor] = ForkState::UsedLeft;
                    self.is_hungry[neighbor] = false;
                    self.emit(DPEvent::Eat(neighbor));
                }
                if let (true, ForkState::Free) = (self.is_hungry[neighbor], self.fork[self.left(neighbor)]) {
                    self.fork[self.left(neighbor)] = ForkState::UsedLeft;
                    self.fork[neighbor] = ForkState::UsedRight;
                    self.is_hungry[neighbor] = false;
                    self.emit(DPEvent::Eat(neighbor));
                }
                Response::Handled
            }

            DPEvent::Stop(n) => {
                self.stopped_number += 1;
                if self.stopped_number == self.count {
                    self.emit(DPEvent::Terminate)
                }
                Response::Handled
            }

            DPEvent::Terminate => {
                panic!();
            }

            _ => Response::Parent(Table::root)
        }
    }

    fn root(&mut self, event: &DPEvent) -> Response<Self, DPEvent> {
        Response::Handled
    }
}

pub struct Philosopher {

}

impl Philosopher {

    pub fn thinking(&mut self, event: &DPEvent) -> Response<Self, DPEvent> {
        match *event {

            DPEvent::OnEntry => {
                todo!();
                Response::Handled
            }

            DPEvent::Timeout(n) => {
                Response::Transition(Self::hungry)
            }

            _ => Response::Parent(Self::root)
        }
    }

    pub fn hungry(&mut self, event: &DPEvent) -> Response<Self, DPEvent> {
        match *event {

            DPEvent::OnEntry => {
                
            }
        }
    }

    pub fn root(&mut self, event: &DPEvent) -> Response<Self, DPEvent> {
        Response::Handled
    }
}

impl Emitter<DPEvent> for Table {

    fn get_sender(&mut self) -> &mut mpsc::UnboundedSender<DPEvent> {
        &mut self.event_sender
    }

}