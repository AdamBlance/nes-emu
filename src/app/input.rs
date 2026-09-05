use eframe::egui;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum Input {
    Key(egui::Key),
    ControllerButton(gilrs::ev::Button),
    ControllerAxis(gilrs::ev::Axis, bool),
    #[default]
    Unspecified,
}

#[derive(PartialEq, Eq)]
pub enum InputType {
    Keyboard,
    Controller,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerConfig {
    pub name: String,
    pub input_mapping: InputMapping,
}

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
pub struct InputMapping {
    pub up: Input,
    pub down: Input,
    pub left: Input,
    pub right: Input,
    pub b: Input,
    pub a: Input,
    pub start: Input,
    pub select: Input,
    pub pause: Input,
    pub rewind: Input,
    pub fast_forward: Input,
}

pub struct NesButtonState {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub b: bool,
    pub a: bool,
    pub start: bool,
    pub select: bool,
}