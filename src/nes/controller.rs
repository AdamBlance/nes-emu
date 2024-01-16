use crate::util::to_mask;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize)]
pub struct Controller {
    pub button_state: u8,
    pub shift_register: u8,
    pub sr_latch_pin: bool,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum NesButton {
    Up,
    Down,
    Left,
    Right,
    B,
    A,
    Start,
    Select,
}

const UP: u8 = 0b0001_0000;
const DOWN: u8 = 0b0010_0000;
const LEFT: u8 = 0b0100_0000;
const RIGHT: u8 = 0b1000_0000;
const START: u8 = 0b0000_1000;
const SELECT: u8 = 0b0000_0100;
const A: u8 = 0b0000_0001;
const B: u8 = 0b0000_0010;

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

    // TODO: Use bitfields here
    pub fn update_button_state(&mut self, pressed_buttons: &HashSet<NesButton>) {
        self.button_state = 0b00000000;
        self.button_state |= to_mask(pressed_buttons.contains(&NesButton::Up)) & UP;
        self.button_state |= to_mask(pressed_buttons.contains(&NesButton::Down)) & DOWN;
        self.button_state |= to_mask(pressed_buttons.contains(&NesButton::Left)) & LEFT;
        self.button_state |= to_mask(pressed_buttons.contains(&NesButton::Right)) & RIGHT;
        self.button_state |= to_mask(pressed_buttons.contains(&NesButton::Start)) & START;
        self.button_state |= to_mask(pressed_buttons.contains(&NesButton::Select)) & SELECT;
        self.button_state |= to_mask(pressed_buttons.contains(&NesButton::A)) & A;
        self.button_state |= to_mask(pressed_buttons.contains(&NesButton::B)) & B;
    }
}
