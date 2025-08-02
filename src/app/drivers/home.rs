use crate::infra::axum::route;
use axum::routing::get;
use maud::html;

pub fn home_routes() -> axum::Router {
    route("/", get(get_home))
}

async fn get_home() -> maud::Markup {
    html! {
        (maud::DOCTYPE)
        html {
            head {
                title { "Home Page" }
            }
            body {
                h1 { "Welcome to the Home Page!" }
                p { "This is a simple home page served by Axum." }
            }
        }
    }
}
