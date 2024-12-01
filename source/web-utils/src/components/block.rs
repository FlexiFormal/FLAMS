use leptos::prelude::*;
use crate::inject_css;

use super::Header;

#[slot]
pub struct Footer { children:Children }
#[slot]
pub struct HeaderRight { children:Children }
#[slot]
pub struct HeaderLeft { children:Children }
#[slot]
pub struct Separator { children:Children }

#[component]
pub fn Block(
    #[prop(optional)] header:Option<Header>,
    #[prop(optional)] header_right:Option<HeaderRight>,
    #[prop(optional)] header_left:Option<HeaderLeft>,
    #[prop(optional)] footer:Option<Footer>,
    #[prop(optional)] separator:Option<Separator>,
    #[prop(optional)] show_separator:Option<bool>,
    children:Children
) -> impl IntoView {
    use thaw::{Card,CardHeader,CardHeaderProps,CardHeaderAction,CardHeaderDescription,Divider,CardPreview,CardFooter};
    inject_css("immt-block",include_str!("block.css"));
    let has_header = header.is_some() || header_right.is_some() || header_left.is_some();
    let has_separator = separator.is_some() || show_separator == Some(true) || (show_separator.is_none() && has_header);
    view!{
        <Card class="immt-block-card">
            {if has_header {
                Some(CardHeader(CardHeaderProps{
                    class:Option::<String>::None.into(),
                    card_header_action:header_right.map(|c| CardHeaderAction{children:c.children}),
                    card_header_description:header_left.map(|c| CardHeaderDescription{children:c.children}),
                    children:header.map_or_else(
                      || Box::new(|| view!(<span/>).into_any()) as Children,
                      |c| c.children
                    )
                }))
            } else {None}}
            {if has_separator {
                Some(separator.map_or_else(
                  || view!(<div style="margin:5px;"><Divider/></div>),
                  |c| view!(<div style="margin:5px;"><Divider>{(c.children)()}</Divider></div>)
                ))
            } else {None}}
            <CardPreview class="immt-block-card-inner">
              {children()}
            </CardPreview>
            {footer.map(|h| view!{
                <CardFooter>{(h.children)()}</CardFooter>
            })}
        </Card>
    }
}