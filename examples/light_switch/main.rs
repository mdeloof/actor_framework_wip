use std::collections::VecDeque;
use futures::channel::mpsc;
use futures::executor::block_on;
use armature::stator::*;
use armature::event::*;
use armature::commutator::*;

extern crate armature_macro;
use armature_macro::{event, stator};

#[event]
pub enum MyEvent {
    Buttonpress
}

#[stator(MyEvent)]
pub struct Led {
    pub light: bool
}


impl Led {

    pub fn on(&mut self, event: &MyEvent) -> Response<Self, MyEvent> {

        match event {
            MyEvent::Buttonpress => {
                println!("Cool");
                self.publish(MyEvent::Buttonpress);
                Response::Transition(Self::off)
            }
            _ => Response::Handled
        }
    }

    fn off(&mut self, event: &MyEvent) -> Response<Self, MyEvent> {

        match event {
            MyEvent::Buttonpress => {
                println!("Waauw");
                self.publish(MyEvent::Buttonpress);
                Response::Transition(Self::on)
            }
            _ => Response::Handled
        }
    }
}


fn main () {

    let led = Led::new(true);

    let mut commutator = Commutator::new();
    commutator.add_handler(Box::new(led));

    commutator.publish(MyEvent::Buttonpress);

    block_on(commutator.run());
}

