pub enum Destination {
    All,
    Single(usize)
}

pub struct Envelope<E: Event> {
    pub destination: Destination,
    pub event: E
}

pub trait Event {

    fn get_exit_event() -> Self;

    fn get_entry_event() -> Self;
    
    fn get_nop_event() -> Self;
    
}