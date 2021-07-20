#[cfg(test)]
mod tests {

use armature::stator::*;
use std::vec::Vec;

#[armature_macro::event]
enum Event {
    A,
    B,
    C,
    D
}

#[derive(Copy, Clone, Debug)]
pub enum Action {
    Entry,
    Exit
}

#[armature_macro::stator]
mod foo {

use super::*;
use std::fmt;

#[derive(Clone, Default)]
#[stator_struct]
struct Foo {
    pub path: Vec<(State<Self, Event>, Action)>
}

#[stator_states]
impl Foo {
    
    const INIT: State<Self, Event> = Self::s11;

    pub fn s11(&mut self, event: &Event) -> Response<Self, Event> {
        match event {
            Event::OnEntry => {
                self.path.push((Self::s11, Action::Entry));
                Response::Handled
            }
            Event::OnExit => {
                self.path.push((Self::s11, Action::Exit));
                Response::Handled
            }
            Event::A => { Response::Transition(Self::s11) }
            Event::B => { Response::Transition(Self::s12) }
            _ => Response::Parent(Self::s1)
        }
    }

    pub fn s12(&mut self, event: &Event) -> Response<Self, Event> {
        match event {
            Event::OnEntry => {
                self.path.push((Self::s12, Action::Entry));
                Response::Handled
            }
            Event::OnExit => {
                self.path.push((Self::s12, Action::Exit));
                Response::Handled
            }
            Event::C => { Response::Transition(Self::s211) }
            _ => Response::Parent(Self::s1)
        }
    }


    pub fn s1(&mut self, event: &Event) -> Response<Self, Event> {
        match event {
            Event::OnEntry => {
                self.path.push((Self::s1, Action::Entry));
                Response::Handled
            }
            Event::OnExit => {
                self.path.push((Self::s1, Action::Exit));
                Response::Handled
            }
            _ => Response::Parent(Self::s)
        }
    }

    pub fn s211(&mut self, event: &Event) -> Response<Self, Event> {
        match event {
            Event::OnEntry => {
                self.path.push((Self::s211, Action::Entry));
                Response::Handled
            }
            Event::OnExit => {
                self.path.push((Self::s211, Action::Exit));
                Response::Handled
            }
            _ => Response::Parent(Self::s21)
        }
    }

    pub fn s21(&mut self, event: &Event) -> Response<Self, Event> {
        match event {
            Event::OnEntry => {
                self.path.push((Self::s21, Action::Entry));
                Response::Handled
            }
            Event::OnExit => {
                self.path.push((Self::s21, Action::Exit));
                Response::Handled
            }
            _ => Response::Parent(Self::s2)
        }
    }

    pub fn s2(&mut self, event: &Event) -> Response<Self, Event> {
        match event {
            Event::OnEntry => {
                self.path.push((Self::s2, Action::Entry));
                Response::Handled
            }
            Event::OnExit => {
                self.path.push((Self::s2, Action::Exit));
                Response::Handled
            }
            Event::D => { Response::Transition(Self::s11) }
            _ => Response::Parent(Self::s)
        }
    }

    pub fn s(&mut self, event: &Event) -> Response<Self, Event> {
        match event {
            Event::OnEntry => {
                self.path.push((Self::s, Action::Entry));
                Response::Handled
            }
            Event::OnExit => {
                self.path.push((Self::s, Action::Exit));
                Response::Handled
            }
            _ => Response::Handled
        }
    }

}

impl fmt::Debug for Foo {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "StatorComponent")
    }
}

}

use foo::Foo;


#[test]
fn stator_transition() {

    let mut foo = Foo::default();

    foo.init();
    foo.handle(&Event::A);
    foo.handle(&Event::B);
    foo.handle(&Event::C);
    foo.handle(&Event::D);

    let expected_path: [(State<Foo, Event>, Action); 17] = [
        (Foo::s, Action::Entry),
        (Foo::s1, Action::Entry),
        (Foo::s11, Action::Entry),
        (Foo::s11, Action::Exit),
        (Foo::s11, Action::Entry),
        (Foo::s11, Action::Exit),
        (Foo::s12, Action::Entry),
        (Foo::s12, Action::Exit),
        (Foo::s1, Action::Exit),
        (Foo::s2, Action::Entry),
        (Foo::s21, Action::Entry),
        (Foo::s211, Action::Entry),
        (Foo::s211, Action::Exit),
        (Foo::s21, Action::Exit),
        (Foo::s2, Action::Exit),
        (Foo::s1, Action::Entry),
        (Foo::s11, Action::Entry)
    ];

    for i in 0..expected_path.len() {
        let actual_state = foo.path[i].0 as usize;
        let expected_state = expected_path[i].0 as usize;
        if actual_state != expected_state {
            panic!("Transition path is wrong.")
        } else {
            continue;
        }
    }
}

}