pub(crate) mod gorbagana {
    pub(crate) mod wen_restart_proto {
        include!(concat!(env!("OUT_DIR"), "/gorbagana.wen_restart_proto.rs"));
    }
}

pub(crate) mod heaviest_fork_aggregate;
pub(crate) mod last_voted_fork_slots_aggregate;
pub mod wen_restart;
