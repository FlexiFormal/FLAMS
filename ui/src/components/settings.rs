
use crate::components::UITab;
use ratatui::{prelude::*, style::palette::tailwind, widgets::*};
use crossterm::event::{Event, KeyCode, MouseEvent, MouseEventKind};
use oxrdf::Quad;
use ratatui::layout::Constraint::Length;
use ratatui::prelude::Constraint::Fill;
use immt_system::controller::Controller;
use crate::ui::{fill_horizontal, fill_vertical, split_line, text_line_styled};

pub struct Settings {
    rel_mem:usize,
    backend_mem:usize,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            rel_mem:0,
            backend_mem:0,
        }
    }
}

impl Settings {
    fn status(&mut self,area:Rect,buf:&mut Buffer) {
        let area = text_line_styled("Status",Modifier::BOLD,area,buf);
        let (line,area) = split_line(area);
        fill_horizontal(line,buf);
        let area = text_line_styled(format!("Relational: {}",pretty_bytes::converter::convert(self.rel_mem as f64)),Modifier::empty(),area,buf);
        let _ = text_line_styled(format!("Backend: {}",pretty_bytes::converter::convert(self.backend_mem as f64)),Modifier::empty(),area,buf);
    }
}
impl UITab for Settings {
    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let [left,sep,right] = Layout::horizontal([Fill(3),Length(1),Fill(1)]).areas(area);
        Paragraph::new::<&str>("Hier könnte Ihre Werbung stehen")
            .render(left, buf);
        fill_vertical(sep,buf);
        self.status(right,buf);
    }
    fn activate(&mut self, controller: &Controller) {
        self.rel_mem = controller.relational_manager().size() * std::mem::size_of::<Quad>();
        self.backend_mem = std::mem::size_of_val(controller.archives());
    }
}