mod table;
mod philosopher;
mod dp_event;

use table::Table;
use dp_event::DPEvent;

fn main () {

    let (sender, receiver) = mpsc::unbounded::<DPEvent>();

    let table = Table::new(sender, 13);

}