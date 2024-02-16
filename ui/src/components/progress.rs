use ratatui::buffer::Buffer;
use ratatui::layout::{Layout, Rect};
use ratatui::layout::Constraint::{Fill, Length, Max, Percentage};
use ratatui::prelude::Widget;
use ratatui::style::palette::tailwind::{BLUE, GREEN};
use ratatui::text::Line;
use ratatui::widgets::WidgetRef;
use immt_api::FinalStr;
use immt_system::utils::progress::{PROGRESS_BARS, ProgressBarState};
/*
pub(crate) struct ProgressBarManager {
    bars:parking_lot::RwLock<Vec<ProgressBarI>>
}
impl ProgressBarManager {
    fn register(&self,bar:ProgressBarI) -> usize {
        let mut lock = self.bars.write();
        let len = lock.len();
        lock.push(bar);
        len
    }

    fn tick(&self,idx:usize) {
        let mut lock = self.bars.write();
        if let Some(pb) = lock.get_mut(idx){
            pb.current += 1;
            pb.ms_per_tick = (pb.ms_per_tick + (pb.last_tick.elapsed().as_millis() as usize)) / 2;
            pb.last_tick = std::time::Instant::now();
        }
    }

    fn done(&self,idx:usize) -> bool {
        let lock = self.bars.read();
        if let Some(pb) = lock.get(idx) {
            pb.current >= pb.length
        } else { true }
    }
}
pub(crate) static PROGRESS_BARS:ProgressBarManager = ProgressBarManager { bars: parking_lot::RwLock::new(vec!()) };

struct ProgressBarI {
    prefix:&'static str,
    curr_label:String,
    length:usize,
    current:usize,
    percentage:bool,
    last_tick:std::time::Instant,
    ms_per_tick:usize,
}
#[derive(Clone,Copy)]
pub struct ProgressBar(usize);
impl ProgressBar {
    pub fn dummy(label:&str,length:usize) -> Self {
        let p = Self(PROGRESS_BARS.register(ProgressBarI {
            prefix: "Dummy",
            curr_label: label.to_string(),
            length,
            current: 0,
            percentage: false,
            last_tick: std::time::Instant::now(),
            ms_per_tick: 0,
        }));
        std::thread::spawn(move || {
            loop {
                p.tick();
                if p.done() {break}
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        });
        p
    }
    pub fn dummy_pctg(label:&str,length:usize) -> Self {
        let p = Self(PROGRESS_BARS.register(ProgressBarI {
            prefix: "Dummy",
            curr_label: label.to_string(),
            length,
            current: 0,
            percentage: true,
            last_tick: std::time::Instant::now(),
            ms_per_tick: 0,
        }));
        std::thread::spawn(move || {
            loop {
                p.tick();
                if p.done() {break}
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        });
        p
    }

    pub fn tick(&self) {
        PROGRESS_BARS.tick(self.0);
    }
    pub fn done(&self) -> bool {
        PROGRESS_BARS.done(self.0)
    }
    fn with(&self,f:impl FnOnce(&ProgressBarI)) {
        let lock = PROGRESS_BARS.bars.read();
        if let Some(pb) = lock.get(self.0) {
            f(pb);
        }
    }
    fn render(area:Rect,buf:&mut Buffer,percentage:bool,current:usize,length:usize,ms_per_tick:usize,prefix:&str,curr_label:&str) {
        use ratatui::style::Stylize;
        use ratatui::widgets::Widget;

        let plabel = if percentage {
            format!(" {}% ",((current as f64 / length as f64) * 100.0).floor() as u8)
        } else {
            format!(" {}/{} ",current,length)
        };
        let etalabel = if ms_per_tick == 0 {
            "(ca. ∞)".to_string()
        } else {
            let eta = ((length - current) as u64 * ms_per_tick as u64) / 1000;
            let eta = std::time::Duration::from_secs(eta + 1);
            format!("(ca. {:02}:{:02})", eta.as_secs() / 60, eta.as_secs() % 60)
        };

        let [prefix_a,bar,pl,eta,label] = Layout::horizontal([
            Length(prefix.len() as u16 + 1),
            Max(60),
            Length(10),
            Length(12),
            Length(curr_label.len() as u16),
        ]).areas(area);
        format!("{} ",prefix).bold().render(prefix_a,buf);
        let [done,doing,todo] = Layout::horizontal([Percentage(((current as f64 / length as f64) * 100.0).floor() as u16),Length(1),Fill(1)]).areas(bar);
        Line::styled((0..done.width).map(|_|"▰").collect::<String>(),GREEN.c500)
            .render(done, buf);
        Line::styled("▱",GREEN.c500).render(doing,buf);
        Line::styled((0..todo.width).map(|_|"▱").collect::<String>(),BLUE.c500)
            .render(todo, buf);
        Line::raw(plabel).render(pl,buf);
        Line::raw(curr_label).render(label,buf);
        Line::raw(etalabel).render(eta,buf);
    }
}
impl WidgetRef for ProgressBar {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.with(|lock| {
            Self::render(area,buf,lock.percentage,lock.current,lock.length,lock.ms_per_tick,lock.prefix,lock.curr_label.as_str());
        })
    }
}
pub struct Summary;
impl Widget for Summary {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let lock = PROGRESS_BARS.bars.read();
        let prs = lock.iter().filter(|pb| pb.current < pb.length).collect::<Vec<_>>();
        let prefix = format!("{} running tasks",prs.len());
        let current = prs.iter().map(|pb| pb.current).sum::<usize>();
        let length = prs.iter().map(|pb| pb.length).sum::<usize>();
        let rem = length - current;
        let ms_per_tick = prs.iter().map(|pb| pb.ms_per_tick * (pb.length - pb.current)).sum::<usize>() / rem;
        ProgressBar::render(area,buf,true,current,length,ms_per_tick,prefix.as_str(),"");
    }
}

 */

pub struct PBWidget<'a>{pub state:&'a ProgressBarState,pub bar_size: u16}
impl<'a> PBWidget<'a> {
    fn render(area:Rect,buf:&mut Buffer,percentage:bool,current:usize,length:usize,ms_per_tick:usize,prefix:&str,curr_label:&str,bar_size:u16) {
        use ratatui::style::Stylize;
        use ratatui::widgets::Widget;

        let plabel = if percentage {
            format!(" {}% ",((current as f64 / length as f64) * 100.0).floor() as u8)
        } else {
            format!(" {}/{} ",current,length)
        };
        let etalabel = if ms_per_tick == 0 {
            "(ca. ∞)".to_string()
        } else {
            let eta = ((length - current) as u64 * ms_per_tick as u64) / 1000;
            let eta = std::time::Duration::from_secs(eta + 1);
            format!("(ca. {:02}:{:02})", eta.as_secs() / 60, eta.as_secs() % 60)
        };

        let [prefix_a,bar,pl,eta,label] = Layout::horizontal([
            Length(prefix.len() as u16 + 1),
            Max(bar_size),
            Length(10),
            Length(12),
            Length(curr_label.len() as u16),
        ]).areas(area);
        format!("{} ",prefix).bold().render(prefix_a,buf);
        let [done,doing,todo] = Layout::horizontal([Percentage(((current as f64 / length as f64) * 100.0).floor() as u16),Length(1),Fill(1)]).areas(bar);
        Line::styled((0..done.width).map(|_|"▰").collect::<String>(),GREEN.c500)
            .render(done, buf);
        Line::styled("▱",GREEN.c500).render(doing,buf);
        Line::styled((0..todo.width).map(|_|"▱").collect::<String>(),BLUE.c500)
            .render(todo, buf);
        Line::raw(plabel).render(pl,buf);
        Line::raw(curr_label).render(label,buf);
        Line::raw(etalabel).render(eta,buf);
    }
}
impl<'a> Widget for PBWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) where Self: Sized {
        Self::render(area,buf,self.state.percentage,self.state.current,self.state.length,self.state.ms_per_tick,self.state.prefix,&*self.state.curr_label,self.bar_size);
    }
}

pub struct Summary<'a,I:Iterator<Item = &'a ProgressBarState>>{
    pub states:I,
    pub bar_size:u16
}
impl<'a,I:Iterator<Item = &'a ProgressBarState>> Widget for Summary<'a,I> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut count = 0;
        let mut current = 0;
        let mut length = 0;
        let mut ms_per_tick = 0;
        for pb in self.states {
            count += 1;
            current += pb.current;
            length += pb.length;
            ms_per_tick += pb.ms_per_tick * (pb.length - pb.current);
        }
        let rem = length - current;
        let ms_per_tick = ms_per_tick / rem;
        let prefix = format!("{} running tasks", count);
        PBWidget::render(area,buf,true,current,length,ms_per_tick,prefix.as_str(),"",self.bar_size);
    }
}