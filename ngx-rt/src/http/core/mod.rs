mod loc;
mod main;
mod module;
mod srv;

pub use self::loc::LocConfRef;
pub use self::main::{MainConfRef, PhaseRef, Phases};
pub use self::module::module;
pub use self::srv::SrvConfRef;
