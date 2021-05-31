
use armature::event::*;

pub enum DPEvent {
    OnEntry,
    OnExit,
    Nop,
    Hungry(usize),
    Done(usize),
    Eat(usize),
    Stop(usize),
    Terminate,
    MaxPub,
    Timeout(usize)
}

impl Event for DPEvent {

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