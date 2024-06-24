use leptos::*;
use crate::utils::if_logged_in_client;

#[island]
pub fn Queue() -> impl IntoView {
    move || if_logged_in_client(
        || template!{
            <div>"Queue"</div>
        },
        || template!{
            <div>"Please log in to view the queue"</div>
        }
    )
}