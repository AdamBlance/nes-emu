use eframe::egui;
use eframe::egui::{Color32, FontId, Response, Ui, Vec2, Widget};
use std::fmt::{Display, Formatter};

#[derive(Hash, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Input {
    Key(egui::Key),
    ControllerButton(gilrs::ev::Button),
    ControllerAxis(gilrs::ev::Axis, bool),
}

impl Display for Input {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Key(k) => write!(f, "{:?}", *k),
            Self::ControllerButton(b) => write!(f, "{:?}", *b),
            Self::ControllerAxis(a, dir) => write!(f, "{:?} {}", *a, if *dir { "+" } else { "-" }),
        }
    }
}

pub struct InputSelect<'a> {
    pub pressed_input: Option<Input>,
    pub stored_input: &'a mut Option<Input>,
    pub unique_id: &'static str,
}

impl<'a> InputSelect<'a> {
    pub fn new(
        pressed_input: Option<Input>,
        stored_input: &'a mut Option<Input>,
        unique_id: &'static str,
    ) -> Self {
        InputSelect {
            pressed_input,
            stored_input,
            unique_id,
        }
    }
}

impl<'a> Widget for InputSelect<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let text_layout = ui.painter().layout_no_wrap(
            self.stored_input.map_or("".to_string(), |x| x.to_string()),
            FontId::default(),
            Color32::WHITE,
        );

        let (rect, mut response) = ui.allocate_exact_size(
            Vec2 {
                y: ui.spacing().interact_size.y,
                x: text_layout.size().x.max(25.0),
            },
            egui::Sense::click(),
        );

        let state_id = ui.id().with(self.unique_id);

        let mut listening = ui
            .ctx()
            .data(|x| x.get_temp::<bool>(state_id).unwrap_or(false));

        if !listening && response.clicked() {
            listening = true;
            response.mark_changed();
        } else if listening && response.clicked_elsewhere() {
            listening = false;
            response.mark_changed();
        } else if listening
            && self
                .pressed_input
                .is_some_and(|b| b == Input::Key(egui::Key::Escape))
        {
            listening = false;
            response.mark_changed();
        } else if listening && self.pressed_input.is_some() {
            listening = false;
            *self.stored_input = self.pressed_input;
            response.mark_changed();
        }

        ui.data_mut(|data| data.insert_temp(state_id, listening));

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact_selectable(&response, listening);
            ui.painter()
                .rect(rect, 1.0, visuals.bg_fill, visuals.bg_stroke);

            let offset_pos = rect.center() - text_layout.rect.center();
            ui.painter().galley(offset_pos.to_pos2(), text_layout);
        }
        response
    }
}
