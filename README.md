# Armature

Armature is an event-driven stateful actor framework for Rust.

## What does that mean?

- **Event-driven:** events are the driving for behind any change to the system state. Events are gathered in a queue and dispatched to the actors.
- **Stateful actor:** every actor contains a hierarchical state machine that processes incoming events.

## Stator (Stateful Actor)

``` rust

pub struct Led {
    pub event_sender: mpsc::UnboundedSender<MyEvent>,
    pub light: bool
}

impl Led {

    // Every state is a function that takes an event as an argument
    pub fn on(&mut self, event: &MyEvent) -> Response<Self, MyEvent> {

        match event {
            MyEvent::Buttonpress => {
                self.emit(MyEvent::Buttonpress);
                Response::Transition(Self::off)
            }
            _ => Response::Handled
        }
    }

    fn off(&mut self, event: &MyEvent) -> Response<Self, MyEvent> {

        match event {
            MyEvent::Buttonpress => {
                self.emit(MyEvent::Buttonpress);
                Response::Transition(Self::on)
            }
            _ => Response::Handled
        }
    }
}
```

