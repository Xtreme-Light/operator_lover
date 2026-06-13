mod api;
mod components;
mod pages;

use leptos::*;
use leptos_router::{Route, Router, Routes};

use crate::components::Nav;
use crate::pages::{DbPage, ExcelPage, HomePage};

fn main() {
    console_error_panic_hook::set_once();
    _ = console_log::init_with_level(log::Level::Info);

    mount_to_body(|| view! { <App/> });
}

#[component]
fn App() -> impl IntoView {
    view! {
        <div class="app">
            <Router>
                <Nav/>
                <div class="container">
                    <Routes>
                        <Route path="/" view=HomePage/>
                        <Route path="/db" view=DbPage/>
                        <Route path="/excel" view=ExcelPage/>
                    </Routes>
                </div>
            </Router>
        </div>
    }
}
