use maud::{html, Markup};

pub fn layout(content: Markup) -> Markup {
    html! {
        (maud::DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                link rel="stylesheet" href="/public/css/style.css";
                title { "Taskbane" }
            }
            body class="h-dvh w-dvw" {
                header {
                    h1 { "Taskbane" }
                }
                main {
                    (content)
                }
                footer.w-full {
                    p { "Footer content goes here." }
                }
                script src="/public/js/hotreload.js" {}
            }
        }
    }
}
