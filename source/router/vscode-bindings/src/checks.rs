use flams_router_base::maybe_lazy;
use flams_router_content::checks::ResultExt;
use ftml_dom::TermTrackedViews;
use leptos::prelude::*;

maybe_lazy!(
    Checks = {
        // make sure this runs client side rather than server side because of hydration errors
        // I don't understand.
        let sig = RwSignal::new(false);
        Effect::new(move || {
            #[cfg(feature = "hydrate")]
            {
                sig.set(true);
            }
        });
        let inner = move || {
            let url = leptos_router::hooks::use_query_map()
                .with_untracked(|q| q.get("url"))
                .and_then(|s| s.parse().ok());
            url.map(checks)
        };

        (move || if sig.get() { Some(inner()) } else { None }).into_any()
    }
);

fn checks(url: url::Url) -> impl IntoView {
    use flams_web_utils::components::Spinner;
    let check = Resource::new(move || url.clone(), get_check);
    view! {<Suspense fallback = || view!(<Spinner/>)>{move ||
        match check.get().map(|s| s.map(|s| s.map(|s| ftml_solver_trace::results::DocumentCheckResult::from_json(&s)))) {
            Some(Ok(Some(Ok(v)))) => Some(flams_router_content::Views::top(move || v.render())),
            _ => None
        }
    }</Suspense>}
}

#[allow(clippy::unused_async)]
#[server(prefix = "/api", endpoint = "checks")]
pub async fn get_check(url: url::Url) -> Result<Option<String>, ServerFnError<String>> {
    let Some(state) = flams_lsp::STDIOLSPServer::global_state() else {
        return Ok(None);
    };
    let Some(doc) = state.get(&url.into()) else {
        return Ok(None);
    };
    let ret = doc.annotations.lock().check.as_ref().map(|j| j.to_json());
    Ok(ret)
}
