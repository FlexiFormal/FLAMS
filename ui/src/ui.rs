use std::borrow::Cow;
use std::time::Duration;
use crossterm::event::{Event, KeyEventKind, KeyModifiers, KeyCode};
use crossterm::{event, ExecutableCommand};
use crossterm::event::KeyCode::Char;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::buffer::Buffer;
use ratatui::layout::{Layout, Rect};
use ratatui::layout::Constraint::{Fill, Length, Min};
use ratatui::widgets::WidgetRef;
use ratatui::{prelude::*, style::palette::tailwind, widgets::*};
use immt_system::controller::Controller;
use crate::components::progress::{ProgressBar, Summary};
use crate::components::UITab;

#[derive(Default, Clone)]
enum UIState {
    Running,
    #[default]
    Hidden,
    Quitting,
}

pub struct Ui {
    tabs:Vec<(&'static str,Box<dyn UITab>)>,
    selected_tab:usize,
    state:UIState,
    tabs_width:u16,
    millis_per_frame:u64,
    act_framerate:u64,
    progress:Vec<ProgressBar>
}

impl Ui {
    pub fn new() -> (Self,super::components::log::layer::Layer) {
        let (log,layer) = super::components::log::Logger::new();
        let s = Self {
            millis_per_frame:100,
            act_framerate:0,
            selected_tab:0,
            state:UIState::Hidden,
            progress:vec!(),
            tabs:vec!(
                (LIBRARY,Box::<super::components::library::Library>::default()),
                (LOG,Box::new(log)),
                (BUILD,Box::<super::components::buildqueue::BuildqueueUI>::default()),
                (SETTINGS,Box::<super::components::settings::Settings>::default()),
            ),
            tabs_width:
            LIBRARY_LEN + 2 + 1 +
                LOG_LEN + 2 + 1 +
                BUILD_LEN + 2 + 1 +
                SETTINGS_LEN + 2
        };
        (s,layer)
    }
    pub fn add_progress(&mut self,pb:ProgressBar) {
        self.progress.push(pb);
    }
    pub fn run(&mut self,controller: Controller) -> std::io::Result<()> {
        self.tabs.first_mut().unwrap().1.activate(&controller);
        self.state = UIState::Running;
        let mut terminal = init_terminal()?;
        /* // {---
        let mut start = std::time::Instant::now();
        let mut frames = 0;
        // ---} */
        let mut now = std::time::Instant::now();
        loop {
            match &self.state {
                UIState::Running => {
                    /* // {---
                    if start.elapsed().as_millis() > 3000 {
                        self.act_framerate = frames / 3;
                        start = std::time::Instant::now();
                        frames = 0;
                    }
                    // ---} */
                    terminal.draw(|frame| frame.render_widget(&mut *self, frame.size()))?;
                    /* // {---
                    frames += 1;
                    // ---} */
                    let ms = now.elapsed().as_millis() as u64;
                    now = std::time::Instant::now();
                    if ms < self.millis_per_frame {
                        self.handle_events(self.millis_per_frame - ms,&controller)?;
                    } else {
                        self.handle_events(0,&controller)?;
                    }
                },
                UIState::Quitting => {
                    restore_terminal()?;
                    std::process::exit(0);
                },
                UIState::Hidden => {
                    terminal.draw(|frame| frame.render_widget(
                        Text::styled("UI Hidden. Press CTRL+H to show.",
                                     Style::new().fg(tailwind::RED.c800).bold()
                        ),
                        frame.size())
                    )?;
                    loop {
                        if let Event::Key(key) = event::read()? {
                            if key.kind == KeyEventKind::Press &&
                                key.modifiers == KeyModifiers::CONTROL &&
                                key.code == KeyCode::Char('h') {
                                self.state = UIState::Running;
                                break;
                            }
                        }
                    }
                }
            }
        }
        restore_terminal()?;
        Ok(())
    }
    fn handle_events(&mut self,wait:u64,controller:&Controller) -> Result<(), std::io::Error> {
        if event::poll(Duration::from_millis(wait))? {
            let e = event::read()?;
            if let Event::Key(key) = &e {
                if key.kind == KeyEventKind::Press {
                    use KeyCode::*;
                    if key.modifiers == KeyModifiers::CONTROL {
                        match key.code {
                            Right => {
                                self.next_tab(controller);
                                return Ok(())
                            },
                            Left => {
                                self.previous_tab(controller);
                                return Ok(())
                            },
                            Char('c') => {
                                self.quit();
                                return Ok(())
                            },
                            Char('h') => {
                                self.hide();
                                return Ok(())
                            },
                            _ => ()
                        }
                    }
                }
            }
            let tab = &mut self.tabs.get_mut(self.selected_tab).unwrap().1;
            tab.handle_event(controller, e)?;
        }
        Ok(())
    }

    pub fn add_tab(&mut self,name:&'static str,widget:impl UITab + 'static,correct:Option<u16>) {
        let mut len = name.chars().count() as u16 + 2;
        if let Some(c) = correct { len += c; }
        self.tabs_width += len;
        if !self.tabs.is_empty() {
            self.tabs_width += 1;
        }
        self.tabs.push((name,Box::new(widget)));
    }

    fn activate_tab(&mut self,controller:&Controller) {
        let tab = &mut self.tabs.get_mut(self.selected_tab).unwrap().1;
        tab.activate(controller);
    }

    fn next_tab(&mut self,controller:&Controller) {
        let next = self.selected_tab + 1;
        if next < self.tabs.len() {
            self.selected_tab = next;
        } else {
            self.selected_tab = 0;
        }
        self.activate_tab(controller);
    }

    fn previous_tab(&mut self,controller:&Controller) {
        if self.selected_tab > 0 {
            self.selected_tab -= 1;
        } else {
            self.selected_tab = self.tabs.len() - 1;
        }
        self.activate_tab(controller);
    }


    pub fn hide(&mut self) {
        self.state = UIState::Hidden;
    }
    fn quit(&mut self) {
        self.state = UIState::Quitting;
    }

    fn progress_sep(area:Rect, buf:&mut Buffer) {
        let [fill_left,content,fill_right ] =
            Layout::horizontal([Fill(1),Length(TASKS_LEN),Fill(1)])
                .areas(area);
        fill_horizontal(fill_left, buf);
        Line::raw(TASKS)
            .centered()
            .render(content, buf);
        fill_horizontal(fill_right, buf);
    }

    fn progress(&mut self,mut area:Rect,buf:&mut Buffer) {
        let (top,b) = split_line(area);
        Self::progress_sep(top, buf);

        area = b;
        for p in &self.progress {
            let (a,b) = split_line(area);
            p.render_ref(a,buf);
            area = b;
        }
    }
}
impl Widget for &mut Ui {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use Constraint::*;
        let [header_area, center_area, footer_area] =
            Layout::vertical([Length(1), Min(0), Length(1)])
                .areas(area);
        self.render_top(header_area, buf);

        let [left,main,right] = Layout::horizontal([Length(1), Fill(1), Length(1)]).areas(center_area);
        fill_vertical(left, buf);
        self.progress.retain(|p|!p.done());
        if self.progress.is_empty() {
            self.tabs.get_mut(self.selected_tab).unwrap().1.render(main, buf);
        } else if self.progress.len() > 2 {
/*
            let [main,summary,rest] = Layout::vertical([Fill(1),Length(2),Length(self.progress.len() as u16 + 1)]).areas(main);
            self.tabs.get_mut(self.selected_tab).unwrap().1.render(main, buf);
            let [line,bot] = Layout::vertical([Length(1),Length(1)]).areas(summary);
            fill_horizontal(line, buf);
            Summary.render(bot,buf);
            self.progress(rest,buf);
*/
            let [main,line,bot] = Layout::vertical([Fill(1),Length(1),Length(1)]).areas(main);
            self.tabs.get_mut(self.selected_tab).unwrap().1.render(main, buf);
            Ui::progress_sep(line, buf);
            Summary.render(bot,buf);
        } else {
            let [main,bot] = Layout::vertical([Fill(1),Length(self.progress.len() as u16 + 1)]).areas(main);
            self.tabs.get_mut(self.selected_tab).unwrap().1.render(main, buf);
            self.progress(bot,buf);
        }
        fill_vertical(right, buf);
        render_footer(footer_area, buf,None);
        //render_footer(footer_area, buf,Some(&format!("{}fps",self.act_framerate)));
    }
}

impl Ui {
    fn render_top(&self, area: Rect, buf: &mut Buffer) {
        let [
        ul,   title_area,       pre_line,  sep_l, tabs_area, sep_r, remainder, ur ] = Layout::horizontal([
            Length(2),          Length(TITLE_LEN), Fill(1),         Length(1),   Length(self.tabs_width), Length(1), Fill(1),        Length(2)])
            .areas(area);
        upper_left(ul, buf);
        fill_horizontal(pre_line, buf);
        Line::raw(" ").render(sep_l, buf);
        render_title(title_area, buf);
        self.render_tabs(tabs_area, buf);
        Line::raw(" ").render(sep_r, buf);
        fill_horizontal(remainder, buf);
        upper_right(ur, buf);
    }

    fn render_tabs(&self, area: Rect, buf: &mut Buffer) {
        let titles = self.tabs.iter().map::<Line<'static>,_>(|(s,tab)|
            format!(" {s} ")
                .fg(tailwind::SLATE.c200)
                .bg(tailwind::BLUE.c900)
                .into()
        );
        let highlight_style = (Color::default(), tailwind::LIME.c800);
        Tabs::new(titles)
            //.block(Block::default().title("Tabs").borders(Borders::ALL))
            .highlight_style(highlight_style)
            .select(self.selected_tab)
            .padding("", "")
            .divider(symbols::line::VERTICAL)
            .render(area, buf);
    }
}

//const BACKGROUND:crossterm::style::Color = crossterm::style::Color::DarkRed;

fn init_terminal() -> std::io::Result<Terminal<impl Backend>> {
    use std::io::stdout;
    let mut stdout = stdout();
    enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    //let _ = crossterm::execute!(stdout,crossterm::style::SetBackgroundColor(BACKGROUND));
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal() -> std::io::Result<()> {
    use std::io::stdout;
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn render_title(area: Rect, buf: &mut Buffer) {
    TITLE.set_style(tailwind::INDIGO.c500).bold().render(area, buf);
}

fn render_footer(area: Rect, buf: &mut Buffer,add:Option<&str>) {
    match add {
        Some(s) => {
            let [ll,fill_lefta,add,fill_leftb,content,fill_right,lr ] =
                Layout::horizontal([Length(2), Fill(1),Length(s.len() as u16),Fill(1),Length(FOOTER_LEN),Fill(2),Length(2)])
                    .areas(area);
            lower_left(ll, buf);
            fill_horizontal(fill_lefta, buf);
            Line::raw(s).render(add, buf);
            fill_horizontal(fill_leftb, buf);
            Line::raw(FOOTER)
                .centered()
                .render(content, buf);
            fill_horizontal(fill_right, buf);
            lower_right(lr, buf);
        }
        None => {
            let [ll,fill_left,content,fill_right,lr ] =
                Layout::horizontal([Length(2), Fill(1),Length(FOOTER_LEN),Fill(1),Length(2)])
                    .areas(area);
            lower_left(ll, buf);
            fill_horizontal(fill_left, buf);
            Line::raw(FOOTER)
                .centered()
                .render(content, buf);
            fill_horizontal(fill_right, buf);
            lower_right(lr, buf);
        }
    }
}

fn lower_left(area: Rect, buf: &mut Buffer) {
    Line::raw("╰─").render(area, buf);
}
fn lower_right(area: Rect, buf: &mut Buffer) {
    Line::raw("─╯").render(area, buf);
}
fn upper_left(area: Rect, buf: &mut Buffer) {
    Line::raw("╭─").render(area, buf);
}
fn upper_right(area: Rect, buf: &mut Buffer) {
    Line::raw("─╮").render(area, buf);
}
pub fn fill_horizontal(area: Rect, buf: &mut Buffer) {
    Line::raw((0..area.width).map(|_|"─").collect::<String>()).render(area, buf);
}
pub fn fill_vertical(area: Rect, buf: &mut Buffer) {
    for y in 0..area.height {
        Line::raw("│").render(Rect::new(area.x, area.y + y, 1, 1), buf);
    }
}

pub fn split_line(area:Rect) -> (Rect,Rect) {
    let mut line = area;
    line.height = 1;
    let mut rest = area;
    rest.y += 1;
    (line,rest)
}

pub fn text_line_styled<'a,T:Into<Cow<'a,str>>,S:Into<Style>>(text:T,style:S,area:Rect,buf:&mut Buffer) -> Rect {
    let (line,rest) = split_line(area);
    Line::default().spans(vec!(Span::styled(text,style))).render(line,buf);
    rest
}

macro_rules! len_const {
    ($id:ident = $l:literal) => {
        len_const!($id = $l;0);
    };
    ($id:ident = $l:literal;$i:literal) => {
        const $id:&str = $l;
        paste::paste!(const [<$id _LEN>]:u16 = const_str::to_char_array!($l).len() as u16 + $i;);
    };
}
len_const!(TITLE = " ⟨iMMT⟩ ");
len_const!(FOOTER = " CTRL+◄/► to change tab | CTRL+C to quit | CTRL+H to hide ");
len_const!(LOG = "📋 Log";1);
len_const!(BUILD = "👷 Building";1);
len_const!(LIBRARY = "📚 Library";1);
len_const!(SETTINGS = "⚙ Settings");
len_const!(TASKS = " 🚀 Active Processes ";1);