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
    pub state: State<Self, MyEvent>,
    pub event_sender: Option<mpsc::UnboundedSender<Envelope<MyEvent>>>,
    pub light: bool
}

impl Handler<MyEvent> for Led {
    
    fn init(&mut self) {
        Stator::init(self);
    }

    fn handle(&mut self, event: &MyEvent) {
        Stator::handle(self, event);
    }

    fn set_event_sender(&mut self, event_sender: mpsc::UnboundedSender<Envelope<MyEvent>>) {
        Stator::set_event_sender(self, event_sender);
    }
}

impl Stator<MyEvent> for Led {

    fn get_state(&mut self) -> State<Self, MyEvent> {
        self.state
    }

    fn set_state(&mut self, state: State<Self, MyEvent>) {
        self.state = state
    }

    fn get_event_sender(&mut self) -> &mut Option<mpsc::UnboundedSender<Envelope<MyEvent>>> {
        &mut self.event_sender
    }

    fn set_event_sender(&mut self, event_sender: mpsc::UnboundedSender<Envelope<MyEvent>>) {
        self.event_sender = Some(event_sender);
    }

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

    

    let led = Led {
        state: Led::off,
        event_sender: None,
        light: true
    };

    let mut commutator = Commutator::new();
    commutator.add_handler(Box::new(led));

    commutator.publish(MyEvent::Buttonpress);


    block_on(commutator.run());
}

