use std::path::Path;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::palette::tailwind;
use ratatui::widgets::{Block, Borders, Padding, Paragraph, WidgetRef,Widget};
use ratatui::widgets::block::Title;
use immt_system::controller::Controller;
use immt_ui::components::progress::ProgressBar;
use immt_ui::components::UITab;
use immt_ui::ui::Ui;


//#[tokio::main]
fn main() {
    let mut ui = Ui::default();
    let controller = {
        let mut builder = Controller::builder(Path::new("/home/jazzpirate/work/MathHub"));
        immt_stex::register(&mut builder);
        builder.build()
    };
    //ui.add_tab("📋 Log", TestTab::new("Logs", "These are the logs".to_string()),Some(1));
    //👷
    //ui.add_tab("👷 Building", TestTab::new("Build server", "This is the build server".to_string()),Some(1));
    // 📕, 📖, 🕮
    //ui.add_tab("📚 Library", TestTab::new("Library", "This is tab 3".to_string()),Some(1));
    /*ui.add_tab("Tab 4", TestTab::new("Fourth Tab", "This is tab 4".to_string()));
    ui.add_tab("Tab 5", TestTab::new("Fifth Tab", "This is tab 5".to_string()));
    ui.add_tab("Tab 6", TestTab::new("Sixth Tab", "This is tab 6".to_string()));
    ui.add_tab("Tab 7", TestTab::new("Seventh Tab", "This is tab 7".to_string()));
    ui.add_tab("Tab 8", TestTab::new("Eighth Tab", "This is tab 8".to_string()));
    ui.add_tab("Tab 9", TestTab::new("Ninth Tab", "This is tab 9".to_string()));

     */
    //ui.add_tab("⚙ Settings", TestTab::new("Settings", "These are the settings".to_string()),None);
    ui.add_progress(ProgressBar::dummy("First Dummy",50));
    ui.add_progress(ProgressBar::dummy_pctg("Second Dummy",150));
    ui.add_progress(ProgressBar::dummy("Third Dummy - Foo Bar Baz",250));
    ui.add_progress(ProgressBar::dummy_pctg("Fourth Dummy - Foo Bar Baz Blubb",350));
    ui.add_progress(ProgressBar::dummy("Fifth Dummy - Foo Bar Baz",550));
    ui.add_progress(ProgressBar::dummy_pctg("ZOOOOOOOOORT",750));

    ui.run(controller).unwrap();
}

struct TestTab {
    title: &'static str,
    content: String,
}
impl TestTab {
    fn new(title: &'static str, content: String) -> Self {
        Self { title, content }
    }
    fn block(&self) -> Block<'static> {
        Block::default()
            .borders(Borders::ALL)
            //.border_set(symbols::border::PROPORTIONAL_TALL)
            .padding(Padding::horizontal(1))
            //.border_style(tailwind::INDIGO.c700)
            .title(Title::from(self.title).alignment(Alignment::Center))
            .title_style(tailwind::LIME.c400)
    }
}
impl UITab for TestTab {
    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        Paragraph::new::<&str>(self.content.as_ref())
            .block(self.block())
            .render(area, buf);
    }

}