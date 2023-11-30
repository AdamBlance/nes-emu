#[derive(Copy, Clone, Default)]
pub struct Controller {
    pub button_state: u8,
    pub shift_register: u8,    
    pub sr_latch_pin: bool,
}

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
}
