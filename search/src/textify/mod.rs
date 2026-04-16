mod ever;
mod parser;

pub fn textify(s: &str, inline: bool) -> String {
    parser::HtmlParser::run(s, inline).ok().unwrap_or_default()
}
