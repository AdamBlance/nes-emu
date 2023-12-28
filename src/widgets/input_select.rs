use std::fmt::{Display, Formatter};
use eframe::egui;
use eframe::egui::{Color32, FontId, Response, Ui, Vec2, Widget};

#[derive(Clone, Copy, PartialEq)]
pub enum Button {
    Key(egui::Key),
    ControllerButton(gilrs::ev::Button),
    ControllerAxis(gilrs::ev::Axis, bool),

}

impl Display for Button {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Key(k) => write!(f, "{:?}", *k),
            Self::ControllerButton(b) => write!(f, "{:?}", *b),
            Self::ControllerAxis(a, dir) => write!(f, "{:?} {}", *a, if *dir {"+"} else {"-"}),
        }
    }
}

#[derive(Default)]
pub struct InputSelectState {
    pub listening: bool,
    pub button: Option<Button>,
}

pub struct InputSelect<'a> {
    state: &'a mut InputSelectState,
    input: Option<Button>
}

impl<'a> InputSelect<'a> {
    pub fn new(input: Option<Button>, state: &'a mut InputSelectState) -> Self {
        InputSelect {
            state,
            input,
        }
    }
}

impl<'a> Widget for InputSelect<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let text_layout = ui.painter().layout_no_wrap(
            self.state.button.map_or("".to_string(), |x| x.to_string()),
            FontId::default(),
            Color32::WHITE
        );

        let (rect, mut response) = ui.allocate_exact_size(
            Vec2 { y: ui.spacing().interact_size.y, x: text_layout.size().x.max(25.0) },
            egui::Sense::click()
        );

        if !self.state.listening && response.clicked() {
            self.state.listening = true;
            response.mark_changed();
        } else if self.state.listening && response.clicked_elsewhere() {
            self.state.listening = false;
            response.mark_changed();
        } else if self.state.listening && self.input.is_some_and(|b| b == Button::Key(egui::Key::Escape)) {
            self.state.listening = false;
            response.mark_changed();
        } else if self.state.listening && self.input.is_some() {
            self.state.listening = false;
            self.state.button = self.input;
            response.mark_changed();
        }

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact_selectable(&response, self.state.listening);
            ui.painter().rect(rect, 1.0, visuals.bg_fill, visuals.bg_stroke);

            let offset_pos = rect.center() - text_layout.rect.center();
            ui.painter().galley(offset_pos.to_pos2(), text_layout);
        }
        response
    }
}
