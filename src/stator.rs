use std::collections::VecDeque;
use futures::channel::mpsc;
use crate::event::*;

const MAX_DEPTH: usize = 16;

pub type State<A, E> = fn(&mut A , &E) -> Response<A, E>;

pub enum Response<A, E: Event> {
    Handled,
    Parent(State<A, E>),
    Transition(State<A, E>)
}


pub trait Handler<E: Event> {

    fn init(&mut self);

    fn handle(&mut self, event: &E);

    fn set_event_sender(&mut self, event_sender: mpsc::UnboundedSender<Envelope<E>>);

    fn set_id(&mut self, id: usize);

}


pub trait Stator<E: Event>: Sized {

    fn get_id(&self) -> Option<usize>;

    fn get_state(&mut self) -> State<Self, E>;

    fn set_state(&mut self, state: State<Self, E>);

    fn get_event_sender(&mut self) -> &mut Option<mpsc::UnboundedSender<Envelope<E>>>;

    fn set_event_sender(&mut self, event_sender: mpsc::UnboundedSender<Envelope<E>>);

    fn get_defered_event_queue(&mut self) -> &mut VecDeque<E>;

    fn get_parent_state(&mut self, state: State<Self, E>) -> Result<State<Self, E>, &str> {
        let nop_event = E::get_nop_event();
        return match state(self, &nop_event) {
            Response::Parent(state) => Ok(state),
            _ => Err("root state has no parent state")
        }
    }

    fn handle(&mut self, event: &E) {
        let state = self.get_state();
        self.call_handler(state, event)
    }

    fn call_handler(&mut self, handler: State<Self, E>, event: &E) {
        match handler(self, event) {
            Response::Transition(target_state) => self.transition(target_state),
            Response::Parent(parent_state) => self.call_handler(parent_state, event),
            Response::Handled => ()
        }
    }

    fn init(&mut self) {
        let mut entry_path: Vec<State<Self, E>> = Vec::with_capacity(MAX_DEPTH);

        let mut entry_temp = self.get_state();

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

    fn transition(&mut self, target: State<Self, E>) {
        let mut exit_path: Vec<State<Self, E>> = Vec::with_capacity(MAX_DEPTH);
        let mut entry_path: Vec<State<Self, E>> = Vec::with_capacity(MAX_DEPTH);
        let source = self.get_state();

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
                // paths, and break out of the loop.
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
            match exit_state(self, &exit_event) {
                Response::Handled => {},
                Response::Transition(_) => panic!(
                    "Do not perform transition on exit event."),
                _ => {}
            }
        }

        // Execute the entry path into the target state
        let entry_event = E::get_entry_event();
        for entry_state in entry_path.into_iter().rev() {
            match entry_state(self, &entry_event) {
                Response::Handled => {},
                Response::Transition(_) => panic!(
                    "Do not perform transition on entry event."),
                _ => {}
            }
        }
        self.set_state(target);
    }

    /// Publish an event to all handlers.
    fn publish(&mut self, event: E) {
        if let Some(event_sender) = self.get_event_sender() {
            let envelope = Envelope {
                destination: Destination::All,
                event: event
            };
            event_sender.unbounded_send(envelope).unwrap(); 
        }
    }

    /// Post an event to a specific handler.
    fn post(&mut self, event: E, handler_id: usize) {
        if let Some(event_sender) = self.get_event_sender() {
            let envelope = Envelope {
                destination: Destination::Single(handler_id),
                event: event
            };
            event_sender.unbounded_send(envelope).unwrap();
        }
    }

    /// Post an event to self.
    fn post_to_self(&mut self, event: E) {
        let id = self.get_id().expect(
            "Stator must be attached to commutator");
        self.post(event, id);
    }

    /// Defer an event for the moment so you can recall it later.
    fn defer(&mut self, event: &E) {
        let queue = self.get_defered_event_queue();
        queue.push_back(*event);
    }

    /// Recall all events that had been defered.
    fn recall_all(&mut self) {
        let queue = self.get_defered_event_queue();
        for event in queue.pop_front() {
            self.post_to_self(event);
        }
    }

    /// Recall the event that is in the front of the defered event queue.
    fn recall_front(&mut self) {
        let queue = self.get_defered_event_queue();
        if let Some(event) = queue.pop_front() {
            self.post_to_self(event);
        }
    }

    /// Recall the event that is in the back of the defered event queue.
    fn recall_back(&mut self) {
        let queue = self.get_defered_event_queue();
        if let Some(event) = queue.pop_back() {
            self.post_to_self(event);
        }
    }

    /// Clear all the events from the defered event queue.
    fn clear_defered(&mut self) {
        let queue = self.get_defered_event_queue();
        queue.clear();
    }

}