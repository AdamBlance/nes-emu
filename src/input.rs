use std::collections::HashSet;
use eframe::egui;
use eframe::egui::Key;
use crate::nes::controller::ButtonState;

pub fn new_button_state(
    keys_down: &HashSet<egui::Key>,
    key_mapping: &KeyMapping,
) -> (ButtonState, ButtonState) {
    let con1 = ButtonState {
        up: keys_down.contains(&key_mapping.con1_up),
        down: keys_down.contains(&key_mapping.con1_down),
        left: keys_down.contains(&key_mapping.con1_left),
        right: keys_down.contains(&key_mapping.con1_right),
        a: keys_down.contains(&key_mapping.con1_a),
        b: keys_down.contains(&key_mapping.con1_b),
        start: keys_down.contains(&key_mapping.con1_start),
        select: keys_down.contains(&key_mapping.con1_select),
    };
    let con2 = ButtonState {
        up: keys_down.contains(&key_mapping.con2_up),
        down: keys_down.contains(&key_mapping.con2_down),
        left: keys_down.contains(&key_mapping.con2_left),
        right: keys_down.contains(&key_mapping.con2_right),
        a: keys_down.contains(&key_mapping.con2_a),
        b: keys_down.contains(&key_mapping.con2_b),
        start: keys_down.contains(&key_mapping.con2_start),
        select: keys_down.contains(&key_mapping.con2_select),
    };
    (con1, con2)
}

pub struct KeyMapping {
    con1_up: Key,
    con1_down: Key,
    con1_left: Key,
    con1_right: Key,
    con1_a: Key,
    con1_b: Key,
    con1_start: Key,
    con1_select: Key,
    con2_up: Key,
    con2_down: Key,
    con2_left: Key,
    con2_right: Key,
    con2_a: Key,
    con2_b: Key,
    con2_start: Key,
    con2_select: Key,
}
impl Default for KeyMapping {
    fn default() -> Self {
        KeyMapping {
            con1_up: Key::W,
            con1_down: Key::R,
            con1_left: Key::A,
            con1_right: Key::S,
            con1_a: Key::E,
            con1_b: Key::N,
            con1_start: Key::J,
            con1_select: Key::G,
            con2_up: Key::Num1,
            con2_down: Key::Num2,
            con2_left: Key::Num3,
            con2_right: Key::Num4,
            con2_a: Key::Num5,
            con2_b: Key::Num6,
            con2_start: Key::Num7,
            con2_select: Key::Num8,
        }
    }
}
