use serde::{Deserialize, Serialize};

type Cycle = u8;



#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum State {
    #[default]
    NotStarted,
    FetchedOpcode,
    InInterrupt(InterruptType, Cycle),
    AddrResolution(Cycle),
    FinishedAddrResolution,
    PendingCarry,

    RmwWrites(Cycle),
    SimpleCycle(Cycle),
    Finished,
}
