#[cfg(test)]
mod tests {

use armature::Commutator;
use std::vec::Vec;
use async_std::task::block_on;

#[armature_macro::event]
pub enum Event {
    Call(usize),
    Respond(usize)
}

#[armature_macro::stator]
mod listener {

use super::*;

#[stator_struct]
#[derive(Clone, Default, Debug)]
struct Listener {
    listeners: Vec<usize>
}

#[stator_states]
impl Listener {

    const INIT: State<Self, Event> = Self::listen;

    fn listen(&mut self, event: &Event) -> Response<Self, Event> {
        match event {
            Event::OnEntry => {
                self.publish(Event::Call(self.get_id().unwrap()));
                Response::Handled
            }
            Event::Call(source) => {
                self.post(Event::Respond(self.get_id().unwrap()), *source);
                Response::Handled
            }
            Event::Respond(source) => {
                self.listeners.push(*source);
                if self.listeners.len() == 3 {
                    self.publish(Event::Detach(self.get_id().unwrap()));
                }
                println!("Received responses from : {:?}", &self.listeners);
                Response::Handled
            }
            _ => Response::Handled
        }
    }
}

}

#[test]
fn commutator_sending() {

    let l1 = listener::Listener::default();
    let l2 = listener::Listener::default();
    let l3 = listener::Listener::default();

    let mut commutator = Commutator::new();

    commutator.attach(Box::new(l1));
    commutator.attach(Box::new(l2));
    commutator.attach(Box::new(l3));

    let timeout = std::time::Duration::from_millis(1000);

    assert!(block_on(async_std::future::timeout(timeout, commutator.run())).is_ok());
    assert!(commutator.drain().len() == 0);

}

}