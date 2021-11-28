/*
use std::time::Duration;

use armature::commutator::*;
use armature::actor::*;
use armature::sender::*;
use armature::utils::async_std::timer::Timer;
use armature_macro::Sig;
use async_std::task::block_on;

#[derive(Sig)]
enum Event {
    TimerElapsed
}

#[derive(Clone, Debug, Default)]
pub struct Led {
    sender: SenderPlug<Event>,
    timer: Timer<Event>,
    light: bool,
    counter: usize
}

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

impl Led {

    fn on_attach(&mut self) {
        self.timer.duration = Duration::from_millis(100);
        self.timer.set_sender_component(self.get_sender_component().clone());
        self.timer.on_elapsed = |this| { this.post_to_self(Event::TimerElapsed); };
    }

    fn on_detach(&mut self) {
        self.timer.clear_sender();
    }

}

impl Stator for Led {

    const INIT: armature::stator::State<Self, Event> = Self::INIT;

    fn get_stator_component_mut(&mut self) -> &mut StatorComponent<Self, Event> {
        &mut self.stator_component
    }

    fn get_stator_component(&self) -> &StatorComponent<Self, Self::Event> {
        &self.stator_component
    }

}

impl Actor for Led {
    type Sig = EventSig;

    fn get_handler_component_mut(&mut self) -> &mut HandlerComponent<Self::Sig> {
        &mut self.handler_component
    }

    fn get_handler_component(&self) -> &HandlerComponent<Self::Sig> {
        &self.handler_component
    }

    fn on_attach(&mut self) {
        Led::on_attach(self);
    }

    fn on_detach(&mut self) {
        Led::on_detach(self);
    }

    fn init(&mut self) {
        Stator::init(self);
    }

    fn handle(&mut self, event: &Self::Event) {
        Stator::handle(self, event);
    }

    fn default_subscriptions(&self) -> Vec<Self::Sig>  {
        Vec::from([Self::Sig::TimerElapsed])
    }
}

impl Sender for Led {
    type Event = Event;

    fn get_sender_component_mut(&mut self) -> &mut SenderComponent<Self::Event> {
        &mut self.sender_component
    }

    fn get_sender_component(&self) -> &SenderComponent<Self::Event> {
        &self.sender_component
    }
}

fn main () {

    let mut commutator = Commutator::new();
    for _ in 0..10 {
        let led = Led::default();
        commutator.attach(Box::new(led.clone()));
    }
    block_on(commutator.run());
}

*/

fn main() {
    println!("Waauw");
}
