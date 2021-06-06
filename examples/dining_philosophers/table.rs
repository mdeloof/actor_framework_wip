use futures::channel::mpsc;
use armature::stator::*;
use armature::event::*;

use crate::DPEvent;

#[derive(Clone, Copy)]
pub enum ForkState {
    Free,
    UsedLeft,
    UsedRight
}

pub struct Table {
    id: Option<usize>,
    state: State<Self, DPEvent>,
    event_sender: Option<mpsc::UnboundedSender<Envelope<DPEvent>>>,
    count: usize,
    fork: Vec<ForkState>,
    is_hungry: Vec<bool>,
    stopped_number: usize
}

impl Handler<DPEvent> for Table {

    fn init(&mut self) {
        Stator::init(self);
    }

    fn handle(&mut self, event: &DPEvent) {
        Stator::handle(self, event);
    }

    fn set_event_sender(&mut self, event_sender: mpsc::UnboundedSender<Envelope<DPEvent>>) {
        Stator::set_event_sender(self, event_sender);
    }

    fn set_id(&mut self, id: usize) {
        self.id = Some(id);
    }

}

impl Stator<DPEvent> for Table {

    fn get_id(&self) -> Option<usize> {
        self.id
    }

    fn get_state(&mut self) -> State<Self, DPEvent> {
        self.state
    }

    fn set_state(&mut self, state: State<Self, DPEvent>) {
        self.state = state
    }

    fn get_event_sender(&mut self) -> &mut Option<mpsc::UnboundedSender<Envelope<DPEvent>>> {
        &mut self.event_sender
    }

    fn set_event_sender(&mut self, event_sender: mpsc::UnboundedSender<Envelope<DPEvent>>) {
        self.event_sender = Some(event_sender);
    }

}

impl Table {

    pub fn new(count: usize) -> Self {
        Self {
            id: None,
            state: Self::serving,
            event_sender: None,
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
                    self.publish(DPEvent::Eat(n));
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
                    self.publish(DPEvent::Eat(neighbor));
                }
                if let (true, ForkState::Free) = (self.is_hungry[neighbor], self.fork[self.left(neighbor)]) {
                    self.fork[self.left(neighbor)] = ForkState::UsedLeft;
                    self.fork[neighbor] = ForkState::UsedRight;
                    self.is_hungry[neighbor] = false;
                    self.publish(DPEvent::Eat(neighbor));
                }
                Response::Handled
            }

            DPEvent::Stop(n) => {
                self.stopped_number += 1;
                if self.stopped_number == self.count {
                    self.publish(DPEvent::Terminate)
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