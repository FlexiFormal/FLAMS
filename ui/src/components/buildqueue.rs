
use crate::components::UITab;
use ratatui::{prelude::*, style::palette::tailwind, widgets::*};
use crossterm::event::{Event, KeyCode, MouseEvent, MouseEventKind};

pub struct BuildqueueUI {}
impl Default for BuildqueueUI {
    fn default() -> Self {
        Self {
        }
    }
}

impl BuildqueueUI {

}
impl UITab for BuildqueueUI {
    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        Paragraph::new::<&str>("Hier könnte Ihre Werbung stehen")
            .render(area, buf);
    }
}