use std::collections::{VecDeque};
use std::fmt;

use crate::event::*;
use crate::sender::*;
use crate::handler::*;

// The max depth states can be nested inside each other.
const MAX_DEPTH: usize = 16;

// Type alias for the signature of a state handler function.
pub type State<T, E> = fn(&mut T , &E) -> Response<T, E>;

pub enum Response<A, E: IsEvent> {
    Handled,
    Parent(State<A, E>),
    Transition(State<A, E>)
}

pub trait Stator: Sender + Handler + Sized + Send {

    // The initial state of the stator.
    const INIT: State<Self, Self::Event>;

    /// Get a mutable reference to the stator component.
    fn get_stator_component_mut(&mut self) -> &mut StatorComponent<Self, Self::Event>;

    /// Get a immutable reference to the stator component.
    fn get_stator_component(&self) -> &StatorComponent<Self, Self::Event>;

    /// Get the parent state of a given state. If a state has no parent
    /// state (most likely because it is the root state) the result will 
    /// be an error.
    fn get_parent_state(&mut self, state: State<Self, Self::Event>) -> Option<State<Self, Self::Event>> {
        let nop_event = Self::Event::new_nop_event();
        return match state(self, &nop_event) {
            Response::Parent(state) => Some(state),
            _ => None
        }
    }

    /// Handle an event from within the current state.
    fn handle(&mut self, event: &Self::Event) {
        let state = self.get_stator_component_mut().state;
        self.call_handler(state, event);
    }

    /// Handle an event from a given state.
    fn call_handler(&mut self, handler: State<Self, Self::Event>, event: &Self::Event) {
        match handler(self, event) {
            Response::Transition(target_state) => self.transition(target_state),
            Response::Parent(parent_state) => self.call_handler(parent_state, event),
            Response::Handled => ()
        }
    }

    /// Perform the transition into the initial state starting from the 
    /// root state.
    fn init(&mut self) {
        let mut entry_path: Vec<State<Self, Self::Event>> = Vec::with_capacity(MAX_DEPTH);

        let mut entry_temp = self.get_stator_component_mut().state;

        // Get the path from the initial state to the root state
        for i in 0..(MAX_DEPTH + 1) {
            entry_path.push(entry_temp);
            match self.get_parent_state(entry_temp) {
                Some(parent_state) => entry_temp = parent_state,
                // Reached the top state
                None => break
            }
            if i == MAX_DEPTH {
                panic!("Reached max state nesting depth of {}", MAX_DEPTH)
            }
        }

        // Execute the entry path into the target state
        let entry_event = Self::Event::new_entry_event();
        for entry_state in entry_path.into_iter().rev() {
            match entry_state(self, &entry_event) {
                Response::Handled => {},
                Response::Transition(_) => panic!(
                    "Do not perform transition on entry event."),
                _ => {}
            }
        }
    }

    /// Perform a transition from the current state towards the target
    /// state.
    fn transition(&mut self, target: State<Self, Self::Event>) {
        let mut exit_path: Vec<State<Self, Self::Event>> = Vec::with_capacity(MAX_DEPTH);
        let mut entry_path: Vec<State<Self, Self::Event>> = Vec::with_capacity(MAX_DEPTH);
        let source = self.get_stator_component_mut().state;

        let mut exit_temp = source;
        let mut entry_temp = target;

        // Get the path from the source state to the root state
        for i in 0..(MAX_DEPTH + 1) {
            exit_path.push(exit_temp);
            match self.get_parent_state(exit_temp) {
                Some(parent_state) => exit_temp = parent_state,
                // Reached the top state
                None => break
            }
            assert_ne!(i, MAX_DEPTH, "Reached max state nesting depth of {}", MAX_DEPTH);
        }

        // Get the path from the target state to the root states
        for i in 0..(MAX_DEPTH + 1) {
            entry_path.push(entry_temp);
            match self.get_parent_state(entry_temp) {
                Some(parent_state) => entry_temp = parent_state,
                // Reached the top state
                None => break
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
        let exit_event = Self::Event::new_exit_event();
        for exit_state in exit_path.into_iter() {
            match exit_state(self, &exit_event) {
                Response::Handled => {},
                Response::Transition(_) => panic!(
                    "Do not perform transition on exit event."),
                _ => {}
            }
        }

        // Execute the entry path into the target state
        let entry_event = Self::Event::new_entry_event();
        for entry_state in entry_path.into_iter().rev() {
            match entry_state(self, &entry_event) {
                Response::Handled => {},
                Response::Transition(_) => panic!(
                    "Do not perform transition on entry event."),
                _ => {}
            }
        }
        self.get_stator_component_mut().state = target;
    }

    /// Defer an event for the moment so you can recall it later.
    fn defer(&mut self, event: &Self::Event) {
        let queue = &mut self.get_stator_component_mut().defered_event_queue;
        queue.push_back(event.clone());
    }

    /// Recall all events that had been defered.
    fn recall_all(&mut self) {
        let queue = &mut self.get_stator_component_mut().defered_event_queue;
        for event in queue.pop_front() {
            self.post_to_self(event);
        }
    }

    /// Recall the event that is in the front of the defered event queue.
    fn recall_front(&mut self) {
        let queue = &mut self.get_stator_component_mut().defered_event_queue;
        if let Some(event) = queue.pop_front() {
            self.post_to_self(event);
        }
    }

    /// Recall the event that is in the back of the defered event queue.
    fn recall_back(&mut self) {
        let queue = &mut self.get_stator_component_mut().defered_event_queue;
        if let Some(event) = queue.pop_back() {
            self.post_to_self(event);
        }
    }

    /// Clear all the events from the defered event queue.
    fn clear_defered(&mut self) {
        let queue = &mut self.get_stator_component_mut().defered_event_queue;
        queue.clear();
    }

}

pub struct StatorComponent<T, E>
where 
T: Stator<Event = E>,
E: IsEvent<Event = E> {
    state: State<T, E>,
    defered_event_queue: VecDeque<E>
}

impl<T, E> Default for StatorComponent<T, E>
where
T: Stator<Event = E>,
E: IsEvent<Event = E> {

    fn default() -> Self {
        Self {
            state: T::INIT,
            defered_event_queue: VecDeque::new()
        }
    }
}

impl<T, E> Clone for StatorComponent<T, E> 
where
T: Stator<Event = E>,
E: IsEvent<Event = E> {

    fn clone(&self) -> Self {
        Self {
            state: self.state,
            defered_event_queue: self.defered_event_queue.clone()
        }
    }

}

impl<T, E > fmt::Debug for StatorComponent<T, E > 
where
T: Stator<Event = E>,
E: IsEvent<Event = E> {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "StatorComponent")
    }
}