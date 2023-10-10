mod loc;
mod main;
mod module;
mod srv;

pub use self::loc::LocConfRef;
pub use self::main::{MainConfRef, PhaseRef, Phases};
pub use self::module::{
    loc_conf, loc_conf_mut, main_conf, main_conf_mut, module, srv_conf, srv_conf_mut,
};
pub use self::srv::SrvConfRef;
