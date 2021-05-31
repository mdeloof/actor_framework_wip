use futures::channel::mpsc;
use crate::event::*;

const MAX_DEPTH: usize = 16;

pub type State<A, E> = fn(&mut A , &E) -> Response<A, E>;


pub enum Response<A, E: Event> {
    Handled,
    Parent(State<A, E>),
    Transition(State<A, E>)
}

#[derive(Clone)]
pub struct Stator<A, E: Event> {
    pub state: State<A, E>,
    pub active_object: A
}

impl<A, E: Event> Stator<A, E> {

    pub fn get_parent_state(&mut self, state: State<A, E>) -> Result<State<A, E>, &str>{
        let nop_event = E::get_nop_event();
        return match state(&mut self.active_object, &nop_event) {
            Response::Parent(state) => Ok(state),
            _ => Err("root state has no parent state")
        }
    }

    fn handle(&mut self, event: &E) {
        let handler = self.state;
        self.call_handler(handler, event)
    }

    fn call_handler(&mut self, handler: State<A, E>, event: &E) {
        match handler(&mut self.active_object, event) {
            Response::Transition(target_state) => self.transition(target_state),
            Response::Parent(parent_state) => self.call_handler(parent_state, event),
            Response::Handled => ()
        }
    }

    fn init(&mut self) {
        let mut entry_path: Vec<State<A,E>> = Vec::with_capacity(MAX_DEPTH);

        let mut entry_temp = self.state;

        // Get the path from the initial state to the root state
        for i in 0..(MAX_DEPTH + 1) {
            entry_path.push(entry_temp);
            match self.get_parent_state(entry_temp) {
                Ok(parent_state) => entry_temp = parent_state,
                // Reached the top state
                Err(_) => break
            }
            if i == MAX_DEPTH {
                panic!("Reached max state nesting depth of {}", MAX_DEPTH)
            }
        }
    }

    fn transition(&mut self, target: State<A, E>) {
        let mut exit_path: Vec<State<A,E>> = Vec::with_capacity(MAX_DEPTH);
        let mut entry_path: Vec<State<A,E>> = Vec::with_capacity(MAX_DEPTH);
        let source = self.state;

        let mut exit_temp = source;
        let mut entry_temp = target;

        // Get the path from the source state to the root state
        for i in 0..(MAX_DEPTH + 1) {
            exit_path.push(exit_temp);
            match self.get_parent_state(exit_temp) {
                Ok(parent_state) => exit_temp = parent_state,
                // Reached the top state
                Err(_) => break
            }
            assert_ne!(i, MAX_DEPTH, "Reached max state nesting depth of {}", MAX_DEPTH);
        }

        // Get the path from the target state to the root states
        for i in 0..(MAX_DEPTH + 1) {
            entry_path.push(entry_temp);
            match self.get_parent_state(entry_temp) {
                Ok(parent_state) => entry_temp = parent_state,
                // Reached the top state
                Err(_) => break
            }
            assert_ne!(i, MAX_DEPTH, "Reached max state nesting depth of {}", MAX_DEPTH);
        }

        // Starting from the root state, trim the entry and exit paths so
        // only uncommon states remain.
        for i in 0..(MAX_DEPTH + 1) {
            // If all states are descendants of a single root state, there
            // will always be at leat one shared shared parent state in the 
            // entry and exit paths.
            entry_temp = *entry_path.last().expect(
                "Only perform transitions to leaf states, i.e. states
                 that do not contain other sub-states");
            exit_temp = *exit_path.last().expect(
                "Only perform transitions to leaf states, i.e. states
                 that do not contain other sub-states");
            if exit_temp as usize != entry_temp as usize {
                // Found the top most parent state that is not shared
                break;
            } else {
                // The parent state is shared, so we should remove it from
                // the path. But if this is also the last state in both 
                // paths that means we're dealing with a self-transition. 
                // In that case we keep this state in the entry and exit 
                // paths and break out of the loop.
                if entry_path.len() == 1 && exit_path.len() == 1 {
                    break;
                } else {
                    entry_path.pop();
                    exit_path.pop();
                }
            }
            assert_ne!(i, MAX_DEPTH, "Reached max state nesting depth of {}", MAX_DEPTH);
        }

        // Execute the exit path out of the source state
        let exit_event = E::get_exit_event();
        for exit_state in exit_path.into_iter() {
            match exit_state(&mut self.active_object, &exit_event) {
                Response::Handled => {},
                Response::Transition(_) => panic!(
                    "Do not perform transition on exit event."),
                _ => {}
            }
        }

        // Execute the entry path into the target state
        let entry_event = E::get_entry_event();
        for entry_state in entry_path.into_iter().rev() {
            match entry_state(&mut self.active_object, &entry_event) {
                Response::Handled => {},
                Response::Transition(_) => panic!(
                    "Do not perform transition on entry event."),
                _ => {}
            }
        }
        
        self.state = target;

    }
}

impl<A, E: Event> Handler<E> for Stator<A, E> {

    fn init(&mut self) {
        self.init();
    }

    fn handle(&mut self, event: &E) {
        self.handle(event);
    }

}

pub trait Emitter<E: Event> {

    fn get_sender(&mut self) -> &mut mpsc::UnboundedSender<E>;
    
    fn emit(&mut self, event: E ) {
        let sender = self.get_sender();
        sender.unbounded_send(event).unwrap();
    }

    fn set_sender(&mut self, sender: mpsc::UnboundedSender<E>) {
        *self.get_sender() = sender;
    }

}

pub trait Handler<E: Event> {

    fn init(&mut self);

    fn handle(&mut self, event: &E);

}