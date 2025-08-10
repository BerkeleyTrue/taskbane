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
            div {
                button
                    class="text-white bg-rose-400 hover:bg-rose-700 font-bold py-2 px-4 rounded focus:outline-none focus:shadow-outline"
                    onclick="initRegister()" {
                        "Register"
                }
            }
        }
        script src="/public/js/passkey.js" {}
    })
}
