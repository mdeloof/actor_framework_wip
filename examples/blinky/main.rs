use async_std::task::block_on;
use armature::Commutator;

extern crate armature_macro;
use armature_macro::{event, stator};

#[event]
pub enum Event {
    TimerElapsed
}

#[stator]
pub mod led {

    use super::*;
    use std::time::Duration;
    use armature::utils::async_std::timer::Timer;

    #[stator_struct]
    #[derive(Clone, Default, Debug)]
    pub struct Led {
        #[sender]
        pub timer: Timer<Event>,
        pub light: bool,
        pub counter: usize
    }

    #[stator_states]
    impl Led {

        const INIT: State<Self, Event> = Self::on;

        pub fn on(&mut self, event: &Event) -> Response<Self, Event> {
            match event {

                Event::OnEntry => {
                    self.light = true;
                    self.counter += 1;
                    if self.counter == 10 {
                        let id = self.get_id().unwrap();
                        self.publish(Event::Detach(id));
                        println!("{}: done", &id);
                    }
                    self.timer.start();
                    Response::Handled
                }

                Event::TimerElapsed => {
                    Response::Transition(Self::off)
                }

                _ => Response::Handled

            }
        }

        fn off(&mut self, event: &Event) -> Response<Self, Event> {
            match event {

                Event::OnEntry => {
                    self.light = false;
                    self.timer.start();
                    Response::Handled
                }

                Event::TimerElapsed => {
                    Response::Transition(Self::on)
                }

                _ => Response::Handled

            }
        }

    }

    #[stator_lifecycle]
    impl Led {

        fn on_attach(&mut self) {
            self.timer.on_elapsed = |this| { this.post_to_self(Event::TimerElapsed) };
            self.timer.duration = Duration::from_millis(100);
        }

    }

}

fn main () {
    let mut commutator = Commutator::new();
    for _ in 0..10 {
        let led = led::Led::default();
        commutator.attach(Box::new(led.clone()));
    }
    block_on(commutator.run());
}