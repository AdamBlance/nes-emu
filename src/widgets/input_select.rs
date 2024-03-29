use eframe::egui;
use eframe::egui::{Color32, FontId, Response, Ui, Vec2, Widget};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum Input {
    Key(egui::Key),
    ControllerButton(gilrs::ev::Button),
    ControllerAxis(gilrs::ev::Axis, bool),
    #[default]
    Unspecified,
}

impl Input {
    pub fn specified_and(self, f: impl FnOnce(Self) -> bool) -> bool {
        match self {
            Input::Unspecified => false,
            x => f(x),
        }
    }
}

impl Display for Input {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Key(k) => write!(f, "{:?}", *k),
            Self::ControllerButton(b) => write!(f, "{:?}", *b),
            Self::ControllerAxis(a, dir) => write!(f, "{:?} {}", *a, if *dir { "+" } else { "-" }),
            Self::Unspecified => write!(f, ""),
        }
    }
}

pub struct InputSelect<'a> {
    pub pressed_input: Option<Input>,
    pub stored_input: Option<&'a mut Input>,
    pub unique_id: &'static str,
}

impl<'a> InputSelect<'a> {
    pub fn new(
        pressed_input: Option<Input>,
        stored_input: Option<&'a mut Input>,
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
            self.stored_input
                .as_ref()
                .map_or("".to_owned(), |input| input.to_string()),
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
        } else if listening
            && (response.clicked_elsewhere()
                || self
                    .pressed_input
                    .is_some_and(|b| b == Input::Key(egui::Key::Escape)))
        {
            listening = false;
            response.mark_changed();
        } else if listening && self.pressed_input.is_some() {
            listening = false;
            if let Some(si) = self.stored_input {
                if let Some(pressed_input) = self.pressed_input {
                    *si = pressed_input;
                }
            }
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
