use maud::{html, Markup};

pub fn layout(content: Markup) -> Markup {
    html! {
        (maud::DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                title { "Taskbane" }
            }
            body {
                header {
                    h1 { "Taskbane" }
                }
                main {
                    (content)
                }
                footer.w-full {
                    p { "Footer content goes here." }
                }
            }
        }
    }
}
