#[cfg(test)]
mod tests {

use std::sync::Arc;
use async_std::task::block_on;
use armature::{Commutator, Store};

extern crate armature_macro;
use armature_macro::{event, stator};

#[event]
pub enum Event {
    PathEvent,
    TupleStructEvent(usize),
    StructEvent { foo: usize }
}

#[stator]
pub mod foo {

    use super::*;

    #[stator_struct]
    #[derive(Clone, Default, Debug)]
    pub struct Foo { }

    #[stator_states]
    impl Foo {

        const INIT: State<Self, Event> = Self::bar;

        fn bar(&mut self, event: &Event) -> Response<Self, Event> {
            match event {
                Event::OnEntry => {
                    self.publish(Event::Detach(self.get_id().unwrap()));
                    Response::Handled
                }
                Event::PathEvent => {
                    Response::Handled
                },
                Event::TupleStructEvent(_) => {
                    Response::Handled
                }
                Event::StructEvent { .. } => {
                    Response::Handled
                },
                _ => Response::Handled
            }
        }
    }
}

#[test]
fn store_mutation() {

    let foo = foo::Foo::default();

    let mut commutator = Commutator::new();

    commutator.attach(Box::new(foo));

    let timeout = std::time::Duration::from_millis(1000);

    assert!(block_on(async_std::future::timeout(timeout, commutator.run())).is_ok());
    assert!(commutator.drain().len() == 0);
}

}