pub mod ui;
pub mod utils;
pub mod components {
    pub mod log;
    pub mod library;
    pub mod buildqueue;
    pub mod settings;
    pub mod progress;
    use crossterm::event::Event;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::widgets::Widget;
    use immt_system::controller::Controller;

    pub trait UITab {
        fn handle_event(&mut self,_controller:&Controller,_event:Event) -> Result<(),std::io::Error> {Ok(())}
        fn activate(&mut self,_controller:&Controller) {}
        fn render(&mut self, area: Rect, buf: &mut Buffer);
    }

}