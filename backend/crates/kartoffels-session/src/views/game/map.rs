use super::{Event, Mode, State};
use crate::BotIdExt;
use glam::ivec2;
use kartoffels_ui::{theme, Ui};
use kartoffels_world::prelude::{Dir, Tile, TileBase};
use ratatui::layout::Rect;
use std::time::Instant;
use termwiz::input::{KeyCode, Modifiers};

#[derive(Debug)]
pub struct Map {
    pub blink: Instant,
}

impl Map {
    pub fn render(&self, ui: &mut Ui<Event>, state: &State) {
        self.render_tiles(ui, state);
        self.render_cursor(ui, state);
        self.process_keys(ui);
    }

    fn render_tiles(&self, ui: &mut Ui<Event>, state: &State) {
        let offset = state.camera.pos()
            - ivec2(ui.area.width as i32, ui.area.height as i32) / 2;

        for dy in 0..ui.area.height {
            for dx in 0..ui.area.width {
                let area = Rect {
                    x: ui.area.x + dx,
                    y: ui.area.y + dy,
                    width: 1,
                    height: 1,
                };

                ui.clamp(area, |ui| {
                    let tile = state
                        .snapshot
                        .render()
                        .get(offset + ivec2(dx as i32, dy as i32));

                    self.render_tile(ui, state, tile);
                });
            }
        }
    }

    fn render_cursor(&self, ui: &mut Ui<Event>, state: &State) {
        if let Mode::SpawningPrefabBot {
            cursor_screen: Some(cursor_screen),
            cursor_valid,
            ..
        } = &state.mode
        {
            let cursor_screen = cursor_screen.as_ivec2()
                - ivec2(ui.area.x as i32, ui.area.y as i32);

            if cursor_screen.x >= 0
                && cursor_screen.y >= 0
                && cursor_screen.x < ui.area.width as i32
                && cursor_screen.y < ui.area.height as i32
            {
                let cursor_screen =
                    (cursor_screen.x as u16, cursor_screen.y as u16);

                let cursor_bg = if *cursor_valid {
                    theme::GREEN
                } else {
                    theme::RED
                };

                ui.buf[cursor_screen]
                    .set_char('@')
                    .set_fg(theme::BG)
                    .set_bg(cursor_bg);
            }
        }
    }

    fn render_tile(&self, ui: &mut Ui<Event>, state: &State, tile: Tile) {
        let ch;
        let mut fg;
        let mut bg;

        match tile.base {
            TileBase::BOT => {
                ch = '@';

                fg = state
                    .snapshot
                    .bots()
                    .alive()
                    .by_idx(tile.meta[0])
                    .map(|bot| bot.id.color())
                    .unwrap();

                bg = theme::BG;
            }

            TileBase::BOT_CHEVRON => {
                ch = match Dir::from(tile.meta[1]) {
                    Dir::N => '↑',
                    Dir::E => '→',
                    Dir::S => '↓',
                    Dir::W => '←',
                };

                fg = state
                    .snapshot
                    .bots()
                    .alive()
                    .by_idx(tile.meta[0])
                    .map(|bot| bot.id.color())
                    .unwrap();

                bg = theme::BG;
            }

            TileBase::FLOOR => {
                ch = '.';
                fg = theme::DARK_GRAY;
                bg = theme::BG;
            }

            TileBase::WALL_H => {
                ch = '-';
                fg = theme::GRAY;
                bg = theme::BG;
            }

            TileBase::WALL_V => {
                ch = '|';
                fg = theme::GRAY;
                bg = theme::BG;
            }

            _ => {
                ch = ' ';
                fg = theme::FG;
                bg = theme::BG;
            }
        };

        if ui.enabled {
            if state.paused && tile.base != TileBase::BOT {
                fg = theme::DARK_GRAY;
                bg = theme::BG;
            }

            if tile.base == TileBase::BOT {
                let id = state
                    .snapshot
                    .bots()
                    .alive()
                    .by_idx(tile.meta[0])
                    .map(|bot| bot.id)
                    .unwrap();

                if ui.mouse_over(ui.area) && !state.perms.hero_mode {
                    fg = theme::BG;
                    bg = theme::GREEN;

                    if ui.mouse_pressed() {
                        ui.throw(Event::JoinBot { id });
                    }
                } else {
                    #[allow(clippy::collapsible_else_if)]
                    if let Some(bot) = &state.bot
                        && bot.id == id
                        && self.blink.elapsed().as_millis() % 1000 <= 500
                    {
                        fg = theme::BG;
                        bg = theme::GREEN;
                    }
                }
            }
        }

        let pos = ui.area.as_position();

        ui.buf[pos].set_char(ch).set_fg(fg).set_bg(bg);
    }

    fn process_keys(&self, ui: &mut Ui<Event>) {
        if !ui.enabled {
            return;
        }

        let offset = ivec2(ui.area.width as i32, ui.area.height as i32) / 3;

        if ui.key(KeyCode::Char('w'), Modifiers::NONE)
            || ui.key(KeyCode::UpArrow, Modifiers::NONE)
        {
            ui.throw(Event::MoveCamera {
                delta: ivec2(0, -offset.y),
            });
        }

        if ui.key(KeyCode::Char('a'), Modifiers::NONE)
            || ui.key(KeyCode::LeftArrow, Modifiers::NONE)
        {
            ui.throw(Event::MoveCamera {
                delta: ivec2(-offset.x, 0),
            });
        }

        if ui.key(KeyCode::Char('s'), Modifiers::NONE)
            || ui.key(KeyCode::DownArrow, Modifiers::NONE)
        {
            ui.throw(Event::MoveCamera {
                delta: ivec2(0, offset.y),
            });
        }

        if ui.key(KeyCode::Char('d'), Modifiers::NONE)
            || ui.key(KeyCode::RightArrow, Modifiers::NONE)
        {
            ui.throw(Event::MoveCamera {
                delta: ivec2(offset.x, 0),
            });
        }
    }
}

impl Default for Map {
    fn default() -> Self {
        Self {
            blink: Instant::now(),
        }
    }
}
