use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Widget},
};
use tui_logger::{TuiLoggerLevelOutput, TuiLoggerWidget};

use super::Page;

#[derive(Clone)]
pub(super) struct LoggingView {}

impl LoggingView {
    pub(super) fn new() -> Self {
        Self {}
    }
}

impl Page for LoggingView {
    fn draw(&mut self, frame: &mut ratatui::Frame, area: Rect) {
        TuiLoggerWidget::default()
            .block(Block::bordered().title("Unfiltered TuiLoggerWidget"))
            .output_separator('|')
            .output_timestamp(Some("%F %H:%M:%S%.3f".to_string()))
            .output_level(Some(TuiLoggerLevelOutput::Long))
            .output_target(false)
            .output_file(false)
            .output_line(false)
            .style(Style::default().fg(Color::White))
            .render(area, frame.buffer_mut());
    }

    fn handle_event(&mut self, _event: super::AppEvent) {}
}
