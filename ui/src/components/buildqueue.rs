use crate::components::UITab;
use crossterm::event::{Event, KeyCode, MouseEvent, MouseEventKind};
use immt_system::controller::Controller;
use ratatui::{prelude::*, style::palette::tailwind, widgets::*};
use std::io::Error;

pub struct BuildqueueUI {
    new: usize,
    stale: usize,
    deleted: usize,
}
impl Default for BuildqueueUI {
    fn default() -> Self {
        Self {
            new: 0,
            stale: 0,
            deleted: 0,
        }
    }
}

impl BuildqueueUI {}
impl UITab for BuildqueueUI {
    fn activate(&mut self, controller: &Controller) {
        let (stale, new, deleted) = controller.build_queue().unqueued();
        self.new = new;
        self.stale = stale;
        self.deleted = deleted;
    }
    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        Paragraph::new::<_>(format!(
            "No currently running build tasks\n\
        New:     {}\n\
        Stale:   {}\n\
        Deleted: {}",
            self.new, self.stale, self.deleted
        ))
        .render(area, buf);
    }
    fn handle_event(&mut self, controller: &Controller, event: Event) -> Result<(), Error> {
        if let Event::Key(key) = event {
            if let KeyCode::Char('q') = key.code {
                controller.build_queue().run_all()
            }
        }
        Ok(())
    }
}
