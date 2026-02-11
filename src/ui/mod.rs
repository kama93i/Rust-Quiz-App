mod quiz;
mod result;
mod welcome;

use ratatui::{prelude::*, widgets::Block};

use crate::app::App;
use crate::models::AppState;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    frame.render_widget(Block::default().bg(Color::Reset), area);

    match app.state {
        AppState::Welcome => welcome::render(frame, area),
        AppState::Quiz => quiz::render(frame, area, app),
        AppState::Result => result::render(frame, area, app),
    }
}
