use crate::components::UITab;
use crossterm::event::{Event, KeyCode, MouseEvent, MouseEventKind};
use immt_system::buildqueue::BuildQueueState;
use immt_system::controller::Controller;
use ratatui::{prelude::*, style::palette::tailwind, widgets::*};
use std::io::Error;

#[derive(Default)]
pub struct BuildqueueUI {
    controller: Option<Controller>,
}

impl UITab for BuildqueueUI {
    fn activate(&mut self, controller: &Controller) {
        if self.controller.is_none() {
            self.controller = Some(controller.clone());
        }
    }
    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        if let Some(controller) = &self.controller {
            let state = controller.build_queue().state();
            Paragraph::new::<_>(format!(
                "New:{}, Stale:{}, Deleted: {}, Queued: {}\n{}",
                state.new,
                state.stale,
                state.deleted,
                state.queued,
                state.running.join("\n")
            ))
            .render(area, buf);
        }
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
