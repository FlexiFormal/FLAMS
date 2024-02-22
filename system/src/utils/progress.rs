use std::any::TypeId;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use tracing::Id;

pub mod spinners {
    /*! // https://jsbin.com/lezohatoho/edit?js,output */
    pub const DOTS: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    pub const DOTS2: &[&str] = &["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];
    pub const DOTS3: &[&str] = &["⠋", "⠙", "⠚", "⠞", "⠖", "⠦", "⠴", "⠲", "⠳", "⠓"];
    pub const DOTS4: &[&str] = &[
        "⠄", "⠆", "⠇", "⠋", "⠙", "⠸", "⠰", "⠠", "⠰", "⠸", "⠙", "⠋", "⠇", "⠆",
    ];
    pub const DOTS5: &[&str] = &[
        "⠋", "⠙", "⠚", "⠒", "⠂", "⠂", "⠒", "⠲", "⠴", "⠦", "⠖", "⠒", "⠐", "⠐", "⠒", "⠓", "⠋",
    ];
    pub const DOTS6: &[&str] = &[
        "⠁", "⠉", "⠙", "⠚", "⠒", "⠂", "⠂", "⠒", "⠲", "⠴", "⠤", "⠄", "⠄", "⠤", "⠴", "⠲", "⠒", "⠂",
        "⠂", "⠒", "⠚", "⠙", "⠉", "⠁",
    ];
    pub const DOTS7: &[&str] = &[
        "⠈", "⠉", "⠋", "⠓", "⠒", "⠐", "⠐", "⠒", "⠖", "⠦", "⠤", "⠠", "⠠", "⠤", "⠦", "⠖", "⠒", "⠐",
        "⠐", "⠒", "⠓", "⠋", "⠉", "⠈",
    ];
    pub const DOTS8: &[&str] = &[
        "⠁", "⠁", "⠉", "⠙", "⠚", "⠒", "⠂", "⠂", "⠒", "⠲", "⠴", "⠤", "⠄", "⠄", "⠤", "⠠", "⠠", "⠤",
        "⠦", "⠖", "⠒", "⠐", "⠐", "⠒", "⠓", "⠋", "⠉", "⠈", "⠈",
    ];
    pub const DOTS9: &[&str] = &["⢹", "⢺", "⢼", "⣸", "⣇", "⡧", "⡗", "⡏"];
    pub const DOTS10: &[&str] = &["⢄", "⢂", "⢁", "⡁", "⡈", "⡐", "⡠"];
    pub const DOTS11: &[&str] = &["⠁", "⠂", "⠄", "⡀", "⢀", "⠠", "⠐", "⠈"];
    pub const DOTS12: &[&str] = &[
        "⢀⠀", "⡀⠀", "⠄⠀", "⢂⠀", "⡂⠀", "⠅⠀", "⢃⠀", "⡃⠀", "⠍⠀", "⢋⠀", "⡋⠀", "⠍⠁", "⢋⠁", "⡋⠁", "⠍⠉",
        "⠋⠉", "⠋⠉", "⠉⠙", "⠉⠙", "⠉⠩", "⠈⢙", "⠈⡙", "⢈⠩", "⡀⢙", "⠄⡙", "⢂⠩", "⡂⢘", "⠅⡘", "⢃⠨", "⡃⢐",
        "⠍⡐", "⢋⠠", "⡋⢀", "⠍⡁", "⢋⠁", "⡋⠁", "⠍⠉", "⠋⠉", "⠋⠉", "⠉⠙", "⠉⠙", "⠉⠩", "⠈⢙", "⠈⡙", "⠈⠩",
        "⠀⢙", "⠀⡙", "⠀⠩", "⠀⢘", "⠀⡘", "⠀⠨", "⠀⢐", "⠀⡐", "⠀⠠", "⠀⢀", "⠀⡀",
    ];
    pub const LINE: &[&str] = &["-", "\\", "|", "/"];
    pub const LINE2: &[&str] = &["⠂", "-", "–", "—", "–", "-"];
    pub const PIPE: &[&str] = &["┤", "┘", "┴", "└", "├", "┌", "┬", "┐"];
    pub const SIMPLE_DOTS: &[&str] = &[".  ", ".. ", "...", "   "];
    pub const SIMPLE_DOTS_SCROLLING: &[&str] = &[".  ", ".. ", "...", " ..", "  .", "   "];
    pub const STAR: &[&str] = &["✶", "✸", "✹", "✺", "✹", "✷"];
    pub const STAR2: &[&str] = &["+", "x", "*"];
    pub const FLIP: &[&str] = &["_", "_", "_", "-", "`", "`", "'", "´", "-", "_", "_", "_"];
    pub const HAMBURGER: &[&str] = &["☱", "☲", "☴"];
    pub const GROW_VERTICAL: &[&str] = &["▁", "▃", "▄", "▅", "▆", "▇", "▆", "▅", "▄", "▃"];
    pub const GROW_HORIZONTAL: &[&str] =
        &["▏", "▎", "▍", "▌", "▋", "▊", "▉", "▊", "▋", "▌", "▍", "▎"];
    pub const BALLOON: &[&str] = &[" ", ".", "o", "O", "@", "*", " "];
    pub const BALLOON2: &[&str] = &[".", "o", "O", "°", "O", "o", "."];
    pub const NOISE: &[&str] = &["▓", "▒", "░"];
    pub const BOUNCE: &[&str] = &["⠁", "⠂", "⠄", "⠂"];
    pub const BOX_BOUNCE: &[&str] = &["▖", "▘", "▝", "▗"];
    pub const BOX_BOUNCE2: &[&str] = &["▌", "▀", "▐", "▄"];
    pub const TRIANGLE: &[&str] = &["◢", "◣", "◤", "◥"];
    pub const ARC: &[&str] = &["◜", "◠", "◝", "◞", "◡", "◟"];
    pub const CIRCLE: &[&str] = &["◡", "⊙", "◠"];
    pub const SQUARE_CORNERS: &[&str] = &["◰", "◳", "◲", "◱"];
    pub const CIRCLE_QUARTERS: &[&str] = &["◴", "◷", "◶", "◵"];
    pub const CIRCLE_HALVES: &[&str] = &["◐", "◓", "◑", "◒"];
    pub const SQUISH: &[&str] = &["╫", "╪"];
    pub const TOGGLE: &[&str] = &["⊶", "⊷"];
    // ----
    pub const TOGGLE2: &[&str] = &["▫", "▪", "▫"];
    pub const TOGGLE3: &[&str] = &["□", "■", "□"];
    pub const TOGGLE4: &[&str] = &["■", "□", "▪", "▫", "■"];
    pub const TOGGLE5: &[&str] = &["▮", "▯", "▮"];
    pub const TOGGLE6: &[&str] = &["ဝ", "၀", "ဝ"];
    pub const TOGGLE7: &[&str] = &["⦾", "⦿", "⦾"];
    pub const TOGGLE8: &[&str] = &["◍", "◌", "◍"];
    pub const TOGGLE9: &[&str] = &["◉", "◎", "◉"];
    pub const TOGGLE10: &[&str] = &["㊂", "㊀", "㊁", "㊂"];
    pub const TOGGLE11: &[&str] = &["⧇", "⧆", "⧇"];
    pub const TOGGLE12: &[&str] = &["☗", "☖", "☗"];
    pub const TOGGLE13: &[&str] = &["=", "*", "-", "="];
    pub const ARROW: &[&str] = &["←", "↖", "↑", "↗", "→", "↘", "↓", "↙", "←"];
    pub const ARROW2: &[&str] = &["⬆️ ", "↗️ ", "➡️ ", "↘️ ", "⬇️ ", "↙️ ", "⬅️ ", "↖️ ", "⬆️ "];
    pub const ARROW3: &[&str] = &[
        "▹▹▹▹▹",
        "▸▹▹▹▹",
        "▹▸▹▹▹",
        "▹▹▸▹▹",
        "▹▹▹▸▹",
        "▹▹▹▹▸",
        "▸▸▸▸▸",
    ];
    pub const BOUNCING_BAR: &[&str] = &[
        "[    ]", "[=   ]", "[==  ]", "[=== ]", "[ ===]", "[  ==]", "[   =]", "[    ]", "[   =]",
        "[  ==]", "[ ===]", "[====]", "[=== ]", "[==  ]", "[=   ]",
    ];
    pub const BOUNCING_BALL: &[&str] = &[
        "( ●    )",
        "(  ●   )",
        "(   ●  )",
        "(    ● )",
        "(     ●)",
        "(    ● )",
        "(   ●  )",
        "(  ●   )",
        "( ●    )",
        "(●     )",
    ];
    pub const SMILEY: &[&str] = &["😄 ", "😝 ", "😄 "];
    pub const MONKEY: &[&str] = &["🙈 ", "🙈 ", "🙉 ", "🙉 ", "🙊 ", "🙊 "];
    pub const HEARTS: &[&str] = &["💛 ", "💙 ", "💜 ", "💚 ", "❤️ ", "💛 "];
    pub const CLOCK: &[&str] = &[
        "🕛 ", "🕐 ", "🕑 ", "🕒 ", "🕓 ", "🕔 ", "🕕 ", "🕖 ", "🕗 ", "🕘 ", "🕙 ", "🕚 ", "🕛 ",
    ];
    pub const EARTH: &[&str] = &["🌍 ", "🌍 ", "🌎 ", "🌎 ", "🌏 ", "🌏 ", "🌏 "];
    pub const MOON: &[&str] = &[
        "🌑 ", "🌒 ", "🌓 ", "🌔 ", "🌕 ", "🌖 ", "🌗 ", "🌘 ", "🌑 ",
    ];
    pub const RUNNER: &[&str] = &[
        "🚶 ", "🚶 ", "🏃 ", "🏃 ", "🚶 ", "🚶 ", "🏃 ", "🏃 ", "🚶 ",
    ];
    pub const PONG: &[&str] = &[
        "▐⠂       ▌",
        "▐⠈       ▌",
        "▐ ⠂      ▌",
        "▐ ⠠      ▌",
        "▐  ⡀     ▌",
        "▐  ⠠     ▌",
        "▐   ⠂    ▌",
        "▐   ⠈    ▌",
        "▐    ⠂   ▌",
        "▐    ⠠   ▌",
        "▐     ⡀  ▌",
        "▐     ⠠  ▌",
        "▐      ⠂ ▌",
        "▐      ⠈ ▌",
        "▐       ⠂▌",
        "▐       ⠠▌",
        "▐       ⡀▌",
        "▐      ⠠ ▌",
        "▐      ⠂ ▌",
        "▐     ⠈  ▌",
        "▐     ⠂  ▌",
        "▐    ⠠   ▌",
        "▐    ⡀   ▌",
        "▐   ⠠    ▌",
        "▐   ⠂    ▌",
        "▐  ⠈     ▌",
        "▐  ⠂     ▌",
        "▐ ⠠      ▌",
        "▐ ⡀      ▌",
        "▐⠠       ▌",
        "▐⠂       ▌",
    ];
    pub const SHARK: &[&str] = &[
        "▐|\\____________▌",
        "▐_|\\___________▌",
        "▐__|\\__________▌",
        "▐___|\\_________▌",
        "▐____|\\________▌",
        "▐_____|\\_______▌",
        "▐______|\\______▌",
        "▐_______|\\_____▌",
        "▐________|\\____▌",
        "▐_________|\\___▌",
        "▐__________|\\__▌",
        "▐___________|\\_▌",
        "▐____________|\\▌",
        "▐____________/|▌",
        "▐___________/|_▌",
        "▐__________/|__▌",
        "▐_________/|___▌",
        "▐_______/|____▌",
        "▐______/|______▌",
        "▐_____/|_______▌",
        "▐____/|________▌",
        "▐___/|_________▌",
        "▐__/|__________▌",
        "▐_/|___________▌",
        "▐/|____________▌",
    ];
    pub const DQPB: &[&str] = &["d", "q", "p", "b"];
    pub const WEATHER: &[&str] = &[
        "☀️ ", "☀️ ", "☀️ ", "🌤 ", "⛅️ ", "🌥 ", "☁️ ", "🌧 ", "🌨 ", "🌧 ", "🌨 ", "🌧 ", "🌨 ", "⛈ ", "🌨 ",
        "🌧 ", "🌨 ", "☁️ ", "🌥 ", "⛅️ ", "🌤 ", "☀️ ", "☀️ ",
    ];
    pub const CHRISTMAS: &[&str] = &["🌲", "🎄", "🎄"];
    pub const GRENADE: &[&str] = &[
        "،   ", "′   ", " ´ ", " ‾ ", "  ⸌", "  ⸊", "  |", "  ⁎", "  ⁕", " ෴ ", "  ⁓", "   ",
        "   ", "   ",
    ];
    pub const POINT: &[&str] = &["∙∙∙", "●∙∙", "∙●∙", "∙∙●", "∙∙∙", "∙∙∙"];
    pub const LAYER: &[&str] = &["-", "=", "≡", "≡"];
    pub const STAR3: &[&str] = &["⭐", "⭐", "🌟", "🌟", "🌟"];
    pub const RAINBOW_CIRCLE: &[&str] = &["🔴", "🟠", "🟡", "🟢", "🔵", "🟣", "🔴"];
    pub const RAINBOW_BOXES: &[&str] = &["🟥", "🟧", "🟨", "🟩", "🟦", "🟪"];
    // 🟥🟧🟨🟩🟦🟪
}

pub mod bars {
    /*! https://changaco.oy.lc/unicode-progress-bars/ */
    pub const BAR1: &str = "█▁";
    pub const BAR2: &str = "⣿⣀";
    pub const BAR3: &str = "⬤○";
    pub const BAR4: &str = "■□";
    pub const BAR5: &str = "█░";
    pub const BAR6: &str = "▰▱";
    pub const BAR7: &str = "◼▭";
    pub const BAR8: &str = "▮▯";
    pub const BAR9: &str = "⚫⚪";
    pub const BAR10: &str = "■▢";
    pub const BAR11: &str = "▰═";
    pub const BAR12: &str = "▰╍";
}

// -------------------------------------------------------------------------------------------------
/*
pub fn in_progress(prefix:&'static str) -> ProgressBarBuilder {
    ProgressBarBuilder {
        prefix,
        template: None,
        length:None,
        spinner: spinners::ARROW3,
        progress: bars::BAR6,
        bar_length: 40,
        done_color: "green",
        todo_color: "blue"
    }
}

use tracing_indicatif::span_ext::IndicatifSpanExt;

pub struct ProgressBarBuilder {
    prefix:&'static str,
    length:Option<u64>,
    template:Option<String>,
    spinner: &'static[&'static str],
    progress:&'static str,
    bar_length:u16,
    done_color:&'static str,
    todo_color:&'static str,
}
impl ProgressBarBuilder {
    pub fn with_length(mut self,length:u64) -> Self {
        self.length = Some(length);
        self
    }
    pub fn set(self) -> ProgressBar {
        use indicatif::ProgressStyle;
        let s = match self.template {
            Some(t) => ProgressStyle::with_template(t.as_str()).unwrap(),
            None => {
                ProgressStyle::with_template(&format!(
                    "{{spinner}} {} [{{pos}}/{{len}}] {{bar:{}.{}/{}}} {{msg}}\n  (ca. {{eta}} remaining)",
                    self.prefix,
                    self.bar_length,
                    self.done_color,
                    self.todo_color
                )).unwrap()
            }
        }.tick_strings(self.spinner).progress_chars(self.progress);
        let span = tracing::Span::current();
        span.pb_set_style(&s);
        if let Some(l) = self.length {
            span.pb_set_length(l);
        }
        ProgressBar { span }
    }
}

#[derive(Clone)]
pub struct ProgressBar {
    span: tracing::Span,
}
impl ProgressBar {
    pub fn set_message(&self,msg:&str) {
        self.span.pb_set_message(msg);
    }
    pub fn tick(&self) {
        self.span.pb_inc(1);
    }
    pub fn advance(&self,by:u64) {
        self.span.pb_inc(by);
    }
}

 */

// -------------------------------------------------------------------------------------------------

pub struct ProgressBarManager {
    bars: parking_lot::RwLock<(usize, BTreeMap<usize, ProgressBarState>)>,
}
pub static PROGRESS_BARS: ProgressBarManager = ProgressBarManager {
    bars: parking_lot::RwLock::new((0, BTreeMap::new())),
};

impl ProgressBarManager {
    fn register(&self, bar: ProgressBarState) -> usize {
        let mut lock = self.bars.write();
        let len = lock.0;
        lock.0 += 1;
        lock.1.insert(len, bar);
        len
    }

    fn tick(&self, idx: usize) {
        let mut lock = self.bars.write();
        if let Some(pb) = lock.1.get_mut(&idx) {
            pb.current += 1;
            if pb.current == pb.length {
                lock.1.remove(&idx);
            } else {
                pb.ns_per_tick = ((pb.ns_per_tick * (pb.current - 1) as u128)
                    + pb.last_tick.elapsed().as_micros())
                    / (pb.current as u128);
                pb.last_tick = std::time::Instant::now();
            }
        }
    }
    fn advance(&self, index: usize, by: usize) {
        let mut lock = self.bars.write();
        if let Some(pb) = lock.1.get_mut(&index) {
            let old = pb.current as u128;
            pb.current += by;
            if pb.current >= pb.length {
                lock.1.remove(&index);
            } else {
                pb.ns_per_tick = ((pb.ns_per_tick * old) + pb.last_tick.elapsed().as_micros())
                    / (pb.current as u128);
                pb.last_tick = std::time::Instant::now();
            }
        }
    }

    fn set_msg<S: Into<FinalStr>>(&self, index: usize, msg: S) {
        let mut lock = self.bars.write();
        if let Some(pb) = lock.1.get_mut(&index) {
            pb.curr_label = msg.into();
        }
    }

    fn close(&self, index: usize) {
        let mut lock = self.bars.write();
        lock.1.remove(&index);
    }

    fn done(&self, idx: usize) -> bool {
        let lock = self.bars.read();
        lock.1.get(&idx).is_some()
    }
    pub fn with<R>(&self, f: impl FnOnce(&BTreeMap<usize, ProgressBarState>) -> R) -> R {
        let lock = self.bars.read();
        f(&lock.1)
    }
}

pub struct ProgressBarState {
    pub prefix: &'static str,
    pub curr_label: FinalStr,
    pub length: usize,
    pub current: usize,
    pub percentage: bool,
    last_tick: std::time::Instant,
    pub ns_per_tick: u128,
    pub parent: Option<usize>,
}

#[derive(Clone, Copy)]
pub struct ProgressBar(usize);

impl ProgressBar {
    pub fn tick(&self) {
        PROGRESS_BARS.tick(self.0);
    }
    pub fn done(&self) -> bool {
        PROGRESS_BARS.done(self.0)
    }
    pub fn set_message<S: Into<FinalStr>>(&self, msg: S) {
        PROGRESS_BARS.set_msg(self.0, msg);
    }
    pub fn advance(&self, by: usize) {
        PROGRESS_BARS.advance(self.0, by);
    }
}

pub fn in_progress<S: Into<FinalStr>>(
    prefix: &'static str,
    length: usize,
    percentage: bool,
    label: S,
) -> Option<ProgressBar> {
    //let i = PROGRESS_BARS.register(pbi);
    let mut label = label.into();
    let lbl = &mut label;
    let span: tracing::Span = tracing::Span::current();
    let r = tracing::Span::with_subscriber(
        &span,
        move |(id, sub): (&tracing::Id, &tracing::Dispatch)| {
            if let Some(ctx) = sub.downcast_ref::<WithProgressSpanContext>() {
                let mut r = None;
                let mut rr = &mut r;
                (ctx.0)(sub, id, &mut move |pbo: &mut ProgressSpanContext| {
                    let mut pbi = ProgressBarState {
                        prefix,
                        curr_label: std::mem::take(lbl),
                        length,
                        current: 0,
                        percentage,
                        last_tick: std::time::Instant::now(),
                        ns_per_tick: 0,
                        parent: None,
                    };
                    if let Some((_, p)) = pbo.parent {
                        pbi.parent = Some(p)
                    }
                    let i = PROGRESS_BARS.register(pbi);
                    pbo.index = Some(i);
                    *rr = Some(i)
                });
                r
            } else {
                None
            }
        },
    )
    .flatten();
    r.map(ProgressBar)
}

use immt_api::FinalStr;
use tracing::span::Attributes;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;

// -------------------------------------------------------------------------------------------------

struct ProgressSpanContext {
    index: Option<usize>,
    parent: Option<(tracing::Id, usize)>,
}

struct WithProgressSpanContext(
    fn(&tracing::Dispatch, &tracing::span::Id, f: &mut dyn FnMut(&mut ProgressSpanContext)),
);

pub struct ProgressLayer<S: tracing::Subscriber> {
    phantom: PhantomData<S>,
    get: WithProgressSpanContext,
}
impl<S> Default for ProgressLayer<S>
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
    fn default() -> Self {
        Self {
            phantom: PhantomData,
            get: WithProgressSpanContext(Self::with_context),
        }
    }
}
impl<S> ProgressLayer<S>
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
    fn with_context(
        dispatch: &tracing::Dispatch,
        id: &tracing::span::Id,
        f: &mut dyn FnMut(&mut ProgressSpanContext),
    ) {
        let subscriber: &S = dispatch
            .downcast_ref::<S>()
            .expect("subscriber should downcast to expected type; this is a bug!");
        let span = subscriber
            .span(id)
            .expect("Span not found in context, this is a bug");

        let mut ext = span.extensions_mut();

        if let Some(ctx) = ext.get_mut::<ProgressSpanContext>() {
            f(ctx);
        }
    }
}

impl<S> tracing_subscriber::layer::Layer<S> for ProgressLayer<S>
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, _attrs: &Attributes<'_>, id: &tracing::Id, ctx: Context<'_, S>) {
        let span = ctx
            .span(id)
            .expect("Span not found in context, this is a bug");
        let mut ext = span.extensions_mut();
        let parent_span = ctx.span_scope(id).and_then(|scope| {
            scope.skip(1).find_map(|span| {
                let ext = span.extensions();
                ext.get::<ProgressSpanContext>()
                    .and_then(|c| c.index.map(|i| (span.id().clone(), i)))
            })
        });

        ext.insert::<ProgressSpanContext>(ProgressSpanContext {
            index: None,
            parent: parent_span,
        });
    }

    fn on_close(&self, id: Id, ctx: Context<'_, S>) {
        let span = ctx
            .span(&id)
            .expect("Span not found in context, this is a bug");
        let mut ext = span.extensions_mut();
        if let Some(i) = ext.get_mut::<ProgressSpanContext>().and_then(|f| f.index) {
            PROGRESS_BARS.close(i)
        }
    }

    unsafe fn downcast_raw(&self, id: TypeId) -> Option<*const ()> {
        match id {
            id if id == TypeId::of::<Self>() => Some(self as *const _ as *const ()),
            id if id == TypeId::of::<WithProgressSpanContext>() => {
                Some(&self.get as *const _ as *const ())
            }
            _ => None,
        }
    }
}
