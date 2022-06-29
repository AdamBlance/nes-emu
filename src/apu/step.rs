
use crate::{nes::Nes, util::get_bit};
use crate::mem::read_mem;
use super::channels::{*, self};

const STEP_1: u16 = 3729;
const STEP_2: u16 = 7457;
const STEP_3: u16 = 11186;
const STEP_4: u16 = 14915;
const STEP_5: u16 = 18641;

/*

Unlike most other NES components, it helps to think of the APU as consisting of many independent
units that send messages to one another. 

https://www.nesdev.org/apu_ref.txt



--------------
Square Channel
--------------

                   +---------+    +---------+
                   |  Sweep  |--->|Timer / 2|
                   +---------+    +---------+
                        |              |
                        |              v 
                        |         +---------+    +---------+
                        |         |Sequencer|    | Length  |
                        |         +---------+    +---------+
                        |              |              |
                        v              v              v
    +---------+        |\             |\             |\          +---------+
    |Envelope |------->| >----------->| >----------->| >-------->|   DAC   |
    +---------+        |/             |/             |/          +---------+



----------------
Triangle Channel
----------------

                   +---------+    +---------+
                   |LinearCtr|    | Length  |
                   +---------+    +---------+
                        |              |
                        v              v
    +---------+        |\             |\         +---------+    +---------+ 
    |  Timer  |------->| >----------->| >------->|Sequencer|--->|   DAC   |
    +---------+        |/             |/         +---------+    +---------+ 


-------------   
Noise Channel
-------------

    +---------+    +---------+    +---------+
    |  Timer  |--->| Random  |    | Length  |
    +---------+    +---------+    +---------+
                        |              |
                        v              v
    +---------+        |\             |\         +---------+
    |Envelope |------->| >----------->| >------->|   DAC   |
    +---------+        |/             |/         +---------+

------------------------------
Delta Modulation Channel (DMC)
------------------------------

    +----------+    +---------+
    |DMA Reader|    |  Timer  |
    +----------+    +---------+
         |               |
         |               v
    +----------+    +---------+     +---------+     +---------+ 
    |  Buffer  |----| Output  |---->| Counter |---->|   DAC   |
    +----------+    +---------+     +---------+     +---------+ 

*/













const H: bool = true;
const L: bool = false;
pub static SQUARE_SEQUENCES: [[bool; 8]; 4] = [
    [L, H, L, L, L, L, L, L],  // 12.5% duty
    [L, H, H, L, L, L, L, L],  // 25.0% duty
    [L, H, H, H, H, L, L, L],  // 50.0% duty
    [H, L, L, H, H, H, H, H],  // 75.0% duty
];

pub static TRIANGLE_SEQUENCE: [u8; 32] = [
    15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0,
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
];





pub static SAMPLE_RATE_TABLE: [u16; 16] = [
    428, 380, 340, 320, 286, 254, 226, 214, 
    190, 160, 142, 128, 106,  84,  72,  54,
];


















