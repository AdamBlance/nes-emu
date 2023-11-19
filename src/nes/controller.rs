use crate::util::to_mask;

#[derive(Copy, Clone, Default, Debug)]
pub struct Controller {
    pub button_state: u8,
    pub shift_register: u8,
    pub sr_latch_pin: bool,
}

pub struct ButtonState {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub a: bool,
    pub b: bool,
    pub start: bool,
    pub select: bool,
}

const UP:     u8 = 0b0001_0000;
const DOWN:   u8 = 0b0010_0000;
const LEFT:   u8 = 0b0100_0000;
const RIGHT:  u8 = 0b1000_0000;
const START:  u8 = 0b0000_1000;
const SELECT: u8 = 0b0000_0100;
const A:      u8 = 0b0000_0001;
const B:      u8 = 0b0000_0010;

impl Controller {
    pub fn shift_out_button_state(&mut self) -> u8 {
        let button_state = self.shift_register & 1;
        self.shift_register >>= 1;
        button_state
    }
    pub fn write_to_data_latch(&mut self, val: u8) {
        // If latch was high and first bit of written byte is low,
        // copy controller state into shift register.
        if self.sr_latch_pin && (val & 1) == 0 {
            self.shift_register = self.button_state;
        }
        self.sr_latch_pin = (val & 1) == 1;
    }

    pub fn update_button_state(&mut self, new_state: ButtonState) {
        self.button_state = 0b00000000;
        self.button_state |= to_mask(new_state.up) & UP;
        self.button_state |= to_mask(new_state.down) & DOWN;
        self.button_state |= to_mask(new_state.left) & LEFT;
        self.button_state |= to_mask(new_state.right) & RIGHT;
        self.button_state |= to_mask(new_state.start) & START;
        self.button_state |= to_mask(new_state.select) & SELECT;
        self.button_state |= to_mask(new_state.a) & A;
        self.button_state |= to_mask(new_state.b) & B;
    }

}
