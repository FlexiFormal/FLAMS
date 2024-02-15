
use crate::components::UITab;
use ratatui::{prelude::*, style::palette::tailwind, widgets::*};
use crossterm::event::{Event, KeyCode, MouseEvent, MouseEventKind};
use immt_system::controller::Controller;

pub struct BuildqueueUI {
    new:usize,
    stale:usize,
    deleted:usize
}
impl Default for BuildqueueUI {
    fn default() -> Self {
        Self { new:0,stale:0,deleted:0 }
    }
}

impl BuildqueueUI {

}
impl UITab for BuildqueueUI {
    fn activate(&mut self, controller: &Controller) {
        self.new = controller.build_queue().new.len();
        self.stale = controller.build_queue().stale.len();
        self.deleted = controller.build_queue().deleted.len();
    }
    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        Paragraph::new::<_>(format!("No currently running build tasks\n\
        New:     {}\n\
        Stale:   {}\n\
        Deleted: {}",self.new,self.stale,self.deleted))
            .render(area, buf);
    }
}