#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub mod components;
pub mod mathml;

use std::borrow::Cow;

use immt_utils::{hashstr, CSS};

pub fn do_css(css: CSS) {
    match css {
        CSS::Inline(s) => {
            let id = hashstr("id_", &s);
            //#[cfg(feature="ssr")]
            let s = String::from(s);
            do_inject_css(id.into(), s.into());
        }
        CSS::Link(s) => {
            let id = hashstr("id_", &s);
            #[cfg(feature = "ssr")]
            {
                use leptos_meta::Stylesheet;
                let _ = leptos::view! {
                    <Stylesheet id=id href=s.to_string()/>
                };
            }
            #[cfg(all(any(feature = "hydrate", feature = "csr"), not(feature = "ssr")))]
            {
                use leptos::prelude::document;
                let Some(head) = document().head() else {
                    leptos::logging::log!("ERROR: head does not exist");
                    return;
                };
                match head.query_selector(&format!("link#{id}")) {
                    Ok(Some(_)) => return,
                    Err(e) => {
                        leptos::logging::log!("ERROR: query link element error: {e:?}");
                        return;
                    }
                    Ok(None) => (),
                };
                let Ok(style) = document().create_element("link") else {
                    leptos::logging::log!("ERROR: error creating style element");
                    return;
                };
                _ = style.set_attribute("id", &id);
                _ = style.set_attribute("rel", "stylesheet");
                _ = style.set_attribute("href", &s);
                _ = head.prepend_with_node_1(&style);
            }
        }
    }
}

#[inline]
pub fn inject_css(id: &'static str, content: &'static str) {
    do_inject_css(Cow::Borrowed(id), Cow::Borrowed(content));
}

#[allow(clippy::missing_const_for_fn)]
#[allow(clippy::needless_pass_by_value)]
fn do_inject_css(id: Cow<'static, str>, content: Cow<'static, str>) {
    #[cfg(feature = "ssr")]
    {
        use leptos_meta::Style;

        let _ = leptos::view! {
            <Style id=id>
                {content}
            </Style>
        };
    }
    #[cfg(not(feature = "ssr"))]
    {
        use leptos::prelude::document;
        let Some(head) = document().head() else {
            leptos::logging::log!("ERROR: head does not exist");
            return;
        };
        let Ok(style) = head.query_selector(&format!("style#{id}")) else {
            leptos::logging::log!("ERROR: query style element error");
            return;
        };
        if style.is_some() {
            return;
        }

        let Ok(style) = document().create_element("style") else {
            leptos::logging::log!("ERROR: error creating style element");
            return;
        };
        _ = style.set_attribute("id", &id);
        style.set_text_content(Some(&content));
        _ = head.prepend_with_node_1(&style);
    }
}
