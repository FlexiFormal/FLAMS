pub mod layer;

use chrono::Local;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use either::Either;
use log::LevelFilter;
use crate::components::UITab;
use ratatui::{prelude::*, style::palette::tailwind, widgets::*};
use ratatui::layout::Constraint::{Fill, Length};
use ratatui::layout::Size;
use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
//use tui_logger::{TuiLoggerLevelOutput, TuiLoggerWidget, TuiWidgetState};
use immt_system::controller::Controller;
use tui_scrollview::{ScrollView, ScrollViewState};
use immt_api::utils::HMap;
use crate::components::log::layer::{LogLine, LogStore, OpenSpan, SimpleLogLine, SpanLine, StringVisitor};
use crate::ui::{split_line, text_line_styled};
use crate::utils::Depth;

enum LogL {
    Simple(Line<'static>),
    Spinning(Line<'static>,&'static[&'static str],u8)
}

pub struct Logger{
    //log_state:TuiWidgetState,
    scroll_state:ScrollViewState,
    log_store:LogStore,
    log_state:Vec<LogL>,
    filter:Level
}
impl Default for Logger {
    fn default() -> Self {
        let (layer,log_store) = layer::Layer::new();
        tracing_subscriber::registry()
            //.with(tui_logger::tracing_subscriber_layer())
            .with(layer)
            .init();
        let log_state = /*TuiWidgetState::new()
            .set_default_display_level(LevelFilter::Trace);*/
            Vec::new();
        Self {
            log_state,
            scroll_state:ScrollViewState::default(),
            log_store,
            filter:Level::INFO
        }
    }
}

impl Logger {
    pub fn new() -> (Self,layer::Layer) {
        let (layer,log_store) = layer::Layer::new();
        let s = Self {
            log_state:Vec::new(),
            scroll_state:ScrollViewState::default(),
            log_store,
            filter:Level::INFO
        };
        (s,layer)
    }
    fn do_line(line:&mut LogL,rect:Rect,buf:&mut Buffer) {
        match line {
            LogL::Simple(l) => l.clone().render(rect,buf),
            LogL::Spinning(l,spinner,frame) => {
                let l = l.clone();
                let len = spinner.first().unwrap().chars().count() as u16;
                let spinstr = spinner[*frame as usize];
                *frame += 1;
                if *frame == (spinner.len() as u8) { *frame = 0 }
                let [spin,rest] = Layout::horizontal([Length(len + 1), Fill(1)])
                    .areas(rect);
                Line::raw(format!("{} ",spinstr)).render(spin,buf);
                l.render(rest,buf);
            }
        }
    }

    fn simple_line(level: Level,message:&str,timestamp:&chrono::DateTime<Local>,attrs:&StringVisitor,depth:u8,has_next:bool) -> LogL {
        LogL::Simple(Line::default().spans(vec!(Span::styled(
            format!("{}{} {}: {} {}",Depth(depth,has_next),timestamp.format("%Y-%m-%d %H:%M:%S"),level,message,attrs),match level {
                Level::ERROR => tailwind::RED.c500,
                Level::WARN => tailwind::ORANGE.c500,
                Level::INFO => tailwind::YELLOW.c500,
                Level::DEBUG => tailwind::CYAN.c500,
                Level::TRACE => tailwind::WHITE
            }
        ))))
    }

    fn complex_line(level:Level,message:&str,timestamp:&chrono::DateTime<Local>,attrs:&StringVisitor,spinner:&'static [&'static str],depth:u8,has_next:bool) -> LogL {
        LogL::Spinning(
            Line::default().spans(vec!(Span::styled(
                format!("{}{} {}: {} {}",Depth(depth,has_next),timestamp.format("%Y-%m-%d %H:%M:%S"),level,message,attrs),match level {
                    Level::ERROR => tailwind::RED.c500,
                    Level::WARN => tailwind::ORANGE.c500,
                    Level::INFO => tailwind::YELLOW.c500,
                    Level::DEBUG => tailwind::CYAN.c500,
                    Level::TRACE => tailwind::WHITE
                }
            ))),
            spinner,0
        )
    }
    fn do_lines(target:&mut Vec<LogL>,children:&Vec<LogLine>,filter:Level) {
        let mut curr = children;
        let mut depth = 1;
        let mut idx = 0;
        let mut stack = Vec::new();
        loop {
            while let Some(e) = curr.get(idx) {
                match e {
                    LogLine::Simple(SimpleLogLine{level,message,timestamp,attrs}) if *level <= filter => {
                        target.push(Self::simple_line(*level,message,timestamp,attrs,depth,idx < curr.len() - 1));
                        idx += 1;
                    }
                    LogLine::Span(SpanLine{level,name,attrs,timestamp,children,open:None,..}) => {
                        if *level <= filter {
                            target.push(Self::simple_line(*level, name, timestamp, attrs, depth, idx < curr.len() - 1))
                        }
                        let old = std::mem::replace(&mut curr,children);
                        stack.push((idx,old));
                        idx = 0;
                        depth += 1;
                    }
                    LogLine::Span(SpanLine{level,name,attrs,timestamp,children,open:Some(OpenSpan{spinner,..}),..}) => {
                        if *level <= filter{ target.push(Self::complex_line(*level,name,timestamp,attrs,spinner,depth,idx < curr.len() - 1)) }
                        let old = std::mem::replace(&mut curr,children);
                        stack.push((idx,old));
                        idx = 0;
                        depth += 1;
                    }
                    _ => idx += 1
                }
            }
            depth -= 1;
            match stack.pop() {
                Some((i,old)) => {
                    curr = old;
                    idx = i + 1;
                }
                None => break
            }
        }
    }
    fn process_change(&mut self) {
        use tracing::metadata::Level;
        let mut lock = self.log_store.write();
        lock.1 = false;
        self.log_state.clear();
        for e in lock.0.iter().rev() {
            match e {
                LogLine::Simple(SimpleLogLine{level,message,timestamp,attrs}) if *level <= self.filter => {
                    self.log_state.push(Self::simple_line(*level,message,timestamp,attrs,0,false))
                }
                LogLine::Span(SpanLine{level,name,attrs,timestamp,children,open:None,..}) => {
                    if *level <= self.filter {self.log_state.push(Self::simple_line(*level,name,timestamp,attrs,0,false))}
                    Self::do_lines(&mut self.log_state,children,self.filter);
                }
                LogLine::Span(SpanLine{level,name,attrs,timestamp,children,open:Some(OpenSpan{spinner,..}),..}) => {
                    if *level <= self.filter {self.log_state.push(Self::complex_line(*level,name,timestamp,attrs,spinner,0,false)) }
                    Self::do_lines(&mut self.log_state,children,self.filter);
                }
                _ => ()
            }
        }
    }

    fn logview(&mut self, buf: &mut Buffer,max:u16) {
        use Constraint::*;
        let area = buf.area;
        if self.scroll_state.offset().y > self.log_state.len() as u16 {
            let mut old = self.scroll_state.offset();
            old.y = if self.log_state.len() < 10 { 0 } else {self.log_state.len() as u16 - 10 };
            self.scroll_state.set_offset(old);
        }
        let offset =  self.scroll_state.offset().y as usize;
        let start = if offset < 20 { 0 } else { offset - 20 };
        let end = self.log_state.len().min(start + (max as usize));
        // the scrollview (currently) allocates the full width of the buffer, but overwrites the
        // last column with a scrollbar. This means that we need to account for this when laying
        // out the widgets
        let [mut body, _scrollbar] =
            Layout::horizontal([Fill(1), Length(1)]).areas(area);
        for l in self.log_state.iter_mut().skip(start).take(end - start) {
            let (line,area) = split_line(body);
            body = area;
            Self::do_line(l,line,buf);
        }
/*
        TuiLoggerWidget::default()
            .state(&self.log_state)
            .style_error(Style::default().fg(Color::Red))
            .style_debug(Style::default().fg(Color::Green))
            .style_warn(Style::default().fg(Color::Yellow))
            .style_trace(Style::default().fg(Color::Magenta))
            .style_info(Style::default().fg(Color::Cyan))
            .output_separator('|')
            .output_timestamp(Some("%F %H:%M:%S%.3f".to_string()))
            .output_level(Some(TuiLoggerLevelOutput::Long))
            .output_target(false)
            .output_file(false)
            .output_line(false)
            .render(body, buf);
        // TODO something with body

 */
    }
}
impl UITab for Logger {
    fn handle_event(&mut self, _controller: &Controller, event: Event) -> Result<(), std::io::Error> {
        use KeyCode::*;
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                Down => self.scroll_state.scroll_down(),
                Up => self.scroll_state.scroll_up(),
                PageDown => self.scroll_state.scroll_page_down(),
                PageUp => self.scroll_state.scroll_page_up(),
                Home => self.scroll_state.scroll_to_top(),
                End => self.scroll_state.scroll_to_bottom(),
                Char('e') if self.filter != Level::ERROR => {
                    self.filter = Level::ERROR;
                    self.process_change();
                }
                Char('w') if self.filter != Level::WARN => {
                    self.filter = Level::WARN;
                    self.process_change();
                }
                Char('i') if self.filter != Level::INFO => {
                    self.filter = Level::INFO;
                    self.process_change();
                }
                Char('d') if self.filter != Level::DEBUG => {
                    self.filter = Level::DEBUG;
                    self.process_change();
                }
                Char('t') if self.filter != Level::TRACE => {
                    self.filter = Level::TRACE;
                    self.process_change();
                }
                _ => (),
            },
            _ => ()
        }
        Ok(())
    }
    fn activate(&mut self, _controller: &Controller) {}

    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let [top,area] = Layout::vertical([Length(1), Fill(1)]).areas(area);
        text_line_styled("(E)rror | (W)arning | (I)nfo | (D)ebug | (T)race",Style::new().fg(tailwind::WHITE).add_modifier(Modifier::BOLD).underlined(),top,buf);
        if self.log_store.read().1 { self.process_change() }
        let len = (self.log_state.len() as u16).min(u16::MAX / buf.area.width);
        let size = Size::new(area.width, len);
        let mut scroll_view = ScrollView::new(size);
        let max =scroll_view.area().height;
        self.logview(scroll_view.buf_mut(),max);
        scroll_view.render(area, buf, &mut self.scroll_state);
    }
}
