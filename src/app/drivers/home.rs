use crate::{app::drivers::layout::layout, infra::axum::route};
use axum::routing::get;
use maud::html;

pub fn home_routes() -> axum::Router {
    route("/", get(get_home))
}

async fn get_home() -> maud::Markup {
    layout(html! {
        div {
            h1 { "Home Page!" }
            p { "This is a simple home page served by Axum." }
        }
    })
}
