use indicatif::ProgressStyle;
use tracing::Level;

pub mod spinners {
    /*! // https://jsbin.com/lezohatoho/edit?js,output */
    pub const DOTS: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    pub const DOTS2: &[&str] = &["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];
    pub const DOTS3: &[&str] = &["⠋", "⠙", "⠚", "⠞", "⠖", "⠦", "⠴", "⠲", "⠳", "⠓"];
    pub const DOTS4: &[&str] = &["⠄", "⠆", "⠇", "⠋", "⠙", "⠸", "⠰", "⠠", "⠰", "⠸", "⠙", "⠋", "⠇", "⠆"];
    pub const DOTS5: &[&str] = &["⠋", "⠙", "⠚", "⠒", "⠂", "⠂", "⠒", "⠲", "⠴", "⠦", "⠖", "⠒", "⠐", "⠐", "⠒", "⠓", "⠋"];
    pub const DOTS6: &[&str] = &["⠁", "⠉", "⠙", "⠚", "⠒", "⠂", "⠂", "⠒", "⠲", "⠴", "⠤", "⠄", "⠄", "⠤", "⠴", "⠲", "⠒", "⠂", "⠂", "⠒", "⠚", "⠙", "⠉", "⠁"];
    pub const DOTS7: &[&str] = &["⠈", "⠉", "⠋", "⠓", "⠒", "⠐", "⠐", "⠒", "⠖", "⠦", "⠤", "⠠", "⠠", "⠤", "⠦", "⠖", "⠒", "⠐", "⠐", "⠒", "⠓", "⠋", "⠉", "⠈"];
    pub const DOTS8: &[&str] = &["⠁", "⠁", "⠉", "⠙", "⠚", "⠒", "⠂", "⠂", "⠒", "⠲", "⠴", "⠤", "⠄", "⠄", "⠤", "⠠", "⠠", "⠤", "⠦", "⠖", "⠒", "⠐", "⠐", "⠒", "⠓", "⠋", "⠉", "⠈", "⠈"];
    pub const DOTS9: &[&str] = &["⢹", "⢺", "⢼", "⣸", "⣇", "⡧", "⡗", "⡏"];
    pub const DOTS10: &[&str] = &["⢄", "⢂", "⢁", "⡁", "⡈", "⡐", "⡠"];
    pub const DOTS11: &[&str] = &["⠁", "⠂", "⠄", "⡀", "⢀", "⠠", "⠐", "⠈"];
    pub const DOTS12: &[&str] = &["⢀⠀", "⡀⠀", "⠄⠀", "⢂⠀", "⡂⠀", "⠅⠀", "⢃⠀", "⡃⠀", "⠍⠀", "⢋⠀", "⡋⠀", "⠍⠁", "⢋⠁", "⡋⠁", "⠍⠉", "⠋⠉", "⠋⠉", "⠉⠙", "⠉⠙", "⠉⠩", "⠈⢙", "⠈⡙", "⢈⠩", "⡀⢙", "⠄⡙", "⢂⠩", "⡂⢘", "⠅⡘", "⢃⠨", "⡃⢐", "⠍⡐", "⢋⠠", "⡋⢀", "⠍⡁", "⢋⠁", "⡋⠁", "⠍⠉", "⠋⠉", "⠋⠉", "⠉⠙", "⠉⠙", "⠉⠩", "⠈⢙", "⠈⡙", "⠈⠩", "⠀⢙", "⠀⡙", "⠀⠩", "⠀⢘", "⠀⡘", "⠀⠨", "⠀⢐", "⠀⡐", "⠀⠠", "⠀⢀", "⠀⡀"];
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
    pub const GROW_HORIZONTAL: &[&str] = &["▏", "▎", "▍", "▌", "▋", "▊", "▉", "▊", "▋", "▌", "▍", "▎"];
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
    pub const TOGGLE2: &[&str] = &["▫", "▪","▫"];
    pub const TOGGLE3: &[&str] = &["□", "■","□"];
    pub const TOGGLE4: &[&str] = &["■", "□", "▪", "▫","■"];
    pub const TOGGLE5: &[&str] = &["▮", "▯","▮"];
    pub const TOGGLE6: &[&str] = &["ဝ", "၀","ဝ"];
    pub const TOGGLE7: &[&str] = &["⦾", "⦿","⦾"];
    pub const TOGGLE8: &[&str] = &["◍", "◌","◍"];
    pub const TOGGLE9: &[&str] = &["◉", "◎","◉"];
    pub const TOGGLE10: &[&str] = &["㊂", "㊀", "㊁","㊂"];
    pub const TOGGLE11: &[&str] = &["⧇", "⧆","⧇"];
    pub const TOGGLE12: &[&str] = &["☗", "☖","☗"];
    pub const TOGGLE13: &[&str] = &["=", "*", "-","="];
    pub const ARROW: &[&str] = &["←", "↖", "↑", "↗", "→", "↘", "↓", "↙","←"];
    pub const ARROW2: &[&str] = &["⬆️ ", "↗️ ", "➡️ ", "↘️ ", "⬇️ ", "↙️ ", "⬅️ ", "↖️ ","⬆️ "];
    pub const ARROW3: &[&str] = &["▹▹▹▹▹", "▸▹▹▹▹", "▹▸▹▹▹", "▹▹▸▹▹", "▹▹▹▸▹", "▹▹▹▹▸","▸▸▸▸▸"];
    pub const BOUNCING_BAR: &[&str] = &["[    ]", "[=   ]", "[==  ]", "[=== ]", "[ ===]", "[  ==]", "[   =]", "[    ]", "[   =]", "[  ==]", "[ ===]", "[====]", "[=== ]", "[==  ]", "[=   ]"];
    pub const BOUNCING_BALL: &[&str] = &["( ●    )", "(  ●   )", "(   ●  )", "(    ● )", "(     ●)", "(    ● )", "(   ●  )", "(  ●   )", "( ●    )", "(●     )"];
    pub const SMILEY: &[&str] = &["😄 ", "😝 ","😄 "];
    pub const MONKEY: &[&str] = &["🙈 ", "🙈 ", "🙉 ", "🙉 ", "🙊 ", "🙊 "];
    pub const HEARTS: &[&str] = &["💛 ", "💙 ", "💜 ", "💚 ", "❤️ ","💛 "];
    pub const CLOCK: &[&str] = &["🕛 ", "🕐 ", "🕑 ", "🕒 ", "🕓 ", "🕔 ", "🕕 ", "🕖 ", "🕗 ", "🕘 ", "🕙 ", "🕚 ","🕛 "];
    pub const EARTH: &[&str] = &["🌍 ", "🌍 ","🌎 ","🌎 ", "🌏 ", "🌏 ","🌏 "];
    pub const MOON: &[&str] = &["🌑 ", "🌒 ", "🌓 ", "🌔 ", "🌕 ", "🌖 ", "🌗 ", "🌘 ","🌑 "];
    pub const RUNNER: &[&str] = &["🚶 ", "🚶 ","🏃 ","🏃 ","🚶 ","🚶 ", "🏃 ","🏃 ","🚶 "];
    pub const PONG: &[&str] = &["▐⠂       ▌", "▐⠈       ▌", "▐ ⠂      ▌", "▐ ⠠      ▌", "▐  ⡀     ▌", "▐  ⠠     ▌", "▐   ⠂    ▌", "▐   ⠈    ▌", "▐    ⠂   ▌", "▐    ⠠   ▌", "▐     ⡀  ▌", "▐     ⠠  ▌", "▐      ⠂ ▌", "▐      ⠈ ▌", "▐       ⠂▌", "▐       ⠠▌", "▐       ⡀▌", "▐      ⠠ ▌", "▐      ⠂ ▌", "▐     ⠈  ▌", "▐     ⠂  ▌", "▐    ⠠   ▌", "▐    ⡀   ▌", "▐   ⠠    ▌", "▐   ⠂    ▌", "▐  ⠈     ▌", "▐  ⠂     ▌", "▐ ⠠      ▌", "▐ ⡀      ▌", "▐⠠       ▌","▐⠂       ▌"];
    pub const SHARK: &[&str] = &["▐|\\____________▌", "▐_|\\___________▌", "▐__|\\__________▌", "▐___|\\_________▌", "▐____|\\________▌", "▐_____|\\_______▌", "▐______|\\______▌", "▐_______|\\_____▌", "▐________|\\____▌", "▐_________|\\___▌", "▐__________|\\__▌", "▐___________|\\_▌", "▐____________|\\▌", "▐____________/|▌", "▐___________/|_▌", "▐__________/|__▌", "▐_________/|___▌", "▐_______/|____▌", "▐______/|______▌", "▐_____/|_______▌", "▐____/|________▌", "▐___/|_________▌", "▐__/|__________▌", "▐_/|___________▌", "▐/|____________▌"];
    pub const DQPB: &[&str] = &["d", "q", "p", "b"];
    pub const WEATHER: &[&str] = &["☀️ ", "☀️ ", "☀️ ", "🌤 ", "⛅️ ", "🌥 ", "☁️ ", "🌧 ", "🌨 ", "🌧 ", "🌨 ", "🌧 ", "🌨 ", "⛈ ", "🌨 ", "🌧 ", "🌨 ", "☁️ ", "🌥 ", "⛅️ ", "🌤 ", "☀️ ", "☀️ "];
    pub const CHRISTMAS: &[&str] = &["🌲", "🎄", "🎄"];
    pub const GRENADE: &[&str] = &["،   ", "′   ", " ´ ", " ‾ ", "  ⸌", "  ⸊", "  |", "  ⁎", "  ⁕", " ෴ ", "  ⁓", "   ", "   ", "   "];
    pub const POINT: &[&str] = &["∙∙∙", "●∙∙", "∙●∙", "∙∙●", "∙∙∙", "∙∙∙"];
    pub const LAYER: &[&str] = &["-", "=", "≡", "≡"];
    pub const STAR3: &[&str] = &["⭐","⭐","🌟","🌟","🌟"];
    pub const RAINBOW_CIRCLE: &[&str] = &["🔴","🟠","🟡","🟢","🔵","🟣","🔴"];
    pub const RAINBOW_BOXES: &[&str] = &["🟥","🟧","🟨","🟩","🟦","🟪"];
    // 🟥🟧🟨🟩🟦🟪
}

pub mod bars {
    /*! https://changaco.oy.lc/unicode-progress-bars/ */
    pub const BAR1 : &str = "█▁";
    pub const BAR2 : &str = "⣿⣀";
    pub const BAR3 : &str = "⬤○";
    pub const BAR4 : &str = "■□";
    pub const BAR5 : &str = "█░";
    pub const BAR6 : &str = "▰▱";
    pub const BAR7 : &str = "◼▭";
    pub const BAR8 : &str = "▮▯";
    pub const BAR9 : &str = "⚫⚪";
    pub const BAR10 : &str = "■▢";
    pub const BAR11 : &str = "▰═";
    pub const BAR12 : &str = "▰╍";
}

pub fn in_progress(prefix:&'static str) -> ProgressBarBuilder {
    ProgressBarBuilder {
        prefix,
        initial_message: "".to_string(),
        level: Level::INFO,
        template: None,
        spinner: spinners::ARROW3,
        progress: bars::BAR6,
        bar_length: 40,
        done_color: "green",
        todo_color: "blue"
    }
}

pub struct ProgressBarBuilder {
    prefix:&'static str,
    initial_message:String,
    level: Level,
    template:Option<String>,
    spinner: &'static[&'static str],
    progress:&'static str,
    bar_length:u16,
    done_color:&'static str,
    todo_color:&'static str,
}
impl ProgressBarBuilder {
    pub fn build(self) -> ProgressStyle {
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
        };
        s.tick_strings(self.spinner).progress_chars(self.progress)
    }
}