use super::Page;

pub(super) struct LoggingView {}

impl LoggingView {}

impl Page for LoggingView {
    fn draw(&mut self, _frame: &mut ratatui::Frame) {
        todo!();
    }

    fn handle_event(&mut self, _event: super::AppEvent) {
        todo!();
    }
}
