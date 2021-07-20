#[cfg(test)]
mod tests {

use std::sync::Arc;
use async_std::task::block_on;
use armature::{Commutator, Store};

extern crate armature_macro;
use armature_macro::{event, stator};

#[event]
pub enum Event {
    PositionUpdate(Arc<Position>)
}

#[derive(Default, Debug, Clone)]
pub struct Position {
    x: isize,
    y: isize
}

#[stator]
pub mod joystick {

    use super::*;

    #[stator_struct]
    #[derive(Clone, Default, Debug)]
    pub struct Joystick {
        #[sender]
        pub position: Store<Position, Event>,
    }

    #[stator_states]
    impl Joystick {

        const INIT: State<Self, Event> = Self::read;

        fn read(&mut self, event: &Event) -> Response<Self, Event> {
            match event {
                Event::OnEntry => {
                    let position = Position {
                        x: 11,
                        y: -54
                    };
                    self.position.mutate(position);
                    self.publish(Event::Detach(self.get_id().unwrap()));
                    Response::Handled
                }
                _ => Response::Handled
            }
        }
    }

    #[stator_lifecycle]
    impl Joystick {

        fn on_attach(&mut self) {
            self.position.on_mutate = |this| {
                this.publish(Event::PositionUpdate(Arc::new(this.value.clone())))
            }
        }
    }
}

#[stator]
pub mod cursor {

    use armature::Response;

    use super::*;

    #[stator_struct]
    #[derive(Clone, Default, Debug)]
    pub struct Cursor {
        pub position: Arc<Position>
    }

    #[stator_states]
    impl Cursor {

        const INIT: State<Self, Event> = Self::draw;

        fn draw(&mut self, event: &Event) -> Response<Self, Event> {
            match event {
                Event::PositionUpdate(position) => {
                    self.position = Arc::clone(position);
                    println!("{:#?}", &self.position);
                    self.publish(Event::Detach(self.get_id().unwrap()));
                    Response::Handled
                }
                _ => Response::Handled
            }
        }
    }
}

#[test]
fn store_mutation() {

    let joystick = joystick::Joystick::default();
    let cursor = cursor::Cursor::default();

    let mut commutator = Commutator::new();

    commutator.attach(Box::new(joystick));
    commutator.attach(Box::new(cursor));

    let timeout = std::time::Duration::from_millis(1000);

    assert!(block_on(async_std::future::timeout(timeout, commutator.run())).is_ok());
    assert!(commutator.drain().len() == 0);
}

}

