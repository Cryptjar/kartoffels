use crate::{theme, Render, Ui};
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Span;
use std::borrow::Cow;
use termwiz::input::{KeyCode, Modifiers};

#[derive(Clone, Debug)]
pub struct Button<'a, T> {
    pub key: Option<KeyCode>,
    pub label: Cow<'a, str>,
    pub throwing: Option<T>,
    pub alignment: Alignment,
    pub enabled: bool,
}

impl<'a, T> Button<'a, T> {
    pub fn new(
        key: impl Into<Option<KeyCode>>,
        label: impl Into<Cow<'a, str>>,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            throwing: None,
            alignment: Alignment::Left,
            enabled: true,
        }
    }

    pub fn throwing(mut self, event: T) -> Self {
        self.throwing = Some(event);
        self
    }

    pub fn centered(mut self) -> Self {
        self.alignment = Alignment::Center;
        self
    }

    pub fn right_aligned(mut self) -> Self {
        self.alignment = Alignment::Right;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn width(&self) -> u16 {
        if let Some(key) = self.key {
            (Self::key_name(key).len() + self.label.len() + 3) as u16
        } else {
            (self.label.len() + 2) as u16
        }
    }

    fn layout(&self, ui: &Ui<T>) -> Rect {
        let area = ui.area();
        let width = self.width();

        let x = match self.alignment {
            Alignment::Left => area.x,
            Alignment::Center => area.x + (area.width - width) / 2,
            Alignment::Right => area.x + area.width - width,
        };

        Rect {
            x,
            y: area.y,
            width,
            height: 1,
        }
    }

    fn response(&self, ui: &Ui<T>, area: Rect) -> ButtonResponse {
        let hovered = ui.enabled() && self.enabled && ui.mouse_over(area);

        let pressed = {
            let by_mouse = hovered && ui.mouse_pressed();

            let by_keyboard = ui.enabled()
                && self.enabled
                && self.key.map_or(false, |key| ui.key(key, Modifiers::NONE));

            by_mouse || by_keyboard
        };

        ButtonResponse { hovered, pressed }
    }

    fn style(&self, ui: &Ui<T>, response: &ButtonResponse) -> (Style, Style) {
        let key = if ui.enabled() && self.enabled {
            if response.pressed || response.hovered {
                Style::new().bold().bg(theme::GREEN).fg(theme::BG)
            } else {
                Style::new().fg(theme::GREEN)
            }
        } else {
            Style::new().fg(theme::DARK_GRAY)
        };

        let label = if ui.enabled() && self.enabled {
            if response.pressed || response.hovered {
                Style::new().bg(theme::GREEN).fg(theme::BG)
            } else {
                Style::default()
            }
        } else {
            Style::new().fg(theme::DARK_GRAY)
        };

        (key, label)
    }

    fn key_name(key: KeyCode) -> String {
        match key {
            KeyCode::Char(' ') => "spc".into(),
            KeyCode::Char(ch) => ch.to_string(),
            KeyCode::Enter => "enter".into(),
            KeyCode::Escape => "esc".into(),

            key => unimplemented!("{:?}", key),
        }
    }
}

impl<T> Render<T> for Button<'_, T> {
    type Response = ButtonResponse;

    fn render(self, ui: &mut Ui<T>) -> Self::Response {
        let area = self.layout(ui);
        let resp = self.response(ui, area);
        let (key_style, label_style) = self.style(ui, &resp);

        let key = self.key.map(Self::key_name);
        let label = &*self.label;

        ui.clamp(area, |ui| {
            ui.row(|ui| {
                if let Some(key) = key {
                    ui.span(Span::styled("[", label_style));
                    ui.span(Span::styled(key, key_style));
                    ui.span(Span::styled("] ", label_style));
                    ui.span(Span::styled(label, label_style));
                } else {
                    ui.span(Span::styled("[", label_style));
                    ui.span(Span::styled(label, key_style));
                    ui.span(Span::styled("]", label_style));
                }
            });
        });

        if ui.layout().is_row() {
            ui.space(area.width);
        } else {
            ui.space(area.height);
        }

        if resp.pressed {
            if let Some(event) = self.throwing {
                ui.throw(event);
            }
        }

        resp
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ButtonResponse {
    pub hovered: bool,
    pub pressed: bool,
}
