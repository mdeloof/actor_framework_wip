use futures::channel::mpsc;
use futures::executor::block_on;
use armature::stator::*;
use armature::event::*;
use armature::commutator::*;

pub enum MyEvent {
    OnEntry,
    OnExit,
    Nop,
    Buttonpress
}

impl Event for MyEvent {

    fn get_entry_event() -> Self {
        Self::OnEntry
    }

    fn get_exit_event() -> Self {
        Self::OnExit
    }

    fn get_nop_event() -> Self {
        Self::Nop
    }

}

pub struct Led {
    pub event_sender: mpsc::UnboundedSender<MyEvent>,
    pub light: bool
}

impl Led {

    pub fn on(&mut self, event: &MyEvent) -> Response<Self, MyEvent> {

        match event {
            MyEvent::Buttonpress => {
                println!("Cool");
                self.emit(MyEvent::Buttonpress);
                Response::Transition(Self::off)
            }
            _ => Response::Handled
        }
    }

    fn off(&mut self, event: &MyEvent) -> Response<Self, MyEvent> {

        match event {
            MyEvent::Buttonpress => {
                println!("Waauw");
                self.emit(MyEvent::Buttonpress);
                Response::Transition(Self::on)
            }
            _ => Response::Handled
        }
    }
}

impl Emitter<MyEvent> for Led {

    fn get_sender(&mut self) -> &mut mpsc::UnboundedSender<MyEvent> {
        &mut self.event_sender
    }

}

fn main () {

    let (sender, receiver) = mpsc::unbounded::<MyEvent>();

    let led_object = Led {
        event_sender: sender.clone(),
        light: true
    };

    let mut led = Stator {
        state: Led::on,
        active_object: led_object
    };

    let mut commutator = Commutator {
        event_receiver: receiver,
        event_sender: sender,
        handlers: vec![&mut led]
    };

    commutator.event_sender.unbounded_send(MyEvent::Buttonpress).unwrap();

    block_on(commutator.run());
}

