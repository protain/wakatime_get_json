#![recursion_limit = "1024"]

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use console_error_panic_hook::set_once as set_panic_hook;
use wasm_bindgen::prelude::*;
use web_sys::Element;
use yew::{html::Scope, prelude::*};
use yew_router::prelude::*;
//use yew_router::{route::Route, switch::Permissive};
mod switch;
use switch::Route;

mod components;
use components::summary_graph::SummaryGraph;

pub enum Msg {
    ToggleNavbar,
}

pub struct App {
    navbar_active: bool,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        crate::hide_loading();
        Self {
            navbar_active: true,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <BrowserRouter>
                {self.view_nav(ctx.link())}
                <main>
                    <Switch<Route>
                        render={Switch::render(Self::switch)}
                    />
                </main>
            </BrowserRouter>
        }
    }
}

impl App {
    fn view_nav(&self, link: &Scope<Self>) -> Html {
        let navbar_active = self.navbar_active;
        let active_class = if navbar_active { "is-active" } else { "" };
        html! {
            <nav class="navbar is-primary" role="navigation" aria-label="main navigation">
                <div class="navbar-brand">
                    <h1 class="navbar-item is-size-3">{ "Wakatime Summary" }</h1>

                    <a role="button"
                        class={classes!("navbar-burger", "burger", active_class)}
                        aria-label="menu" aria-expanded="false"
                    >
                        <span aria-hidden="true"></span>
                        <span aria-hidden="true"></span>
                        <span aria-hidden="true"></span>
                    </a>
                </div>
                <div class={classes!("navbar-menu", active_class)}>
                    <div class="navbar-start">
                        <Link<Route> classes="navbar-item" to={Route::Home}>
                            { "ホーム" }
                        </Link<Route>>
                        <Link<Route> classes="navbar-item" to={Route::PostList}>
                            { "サマリー表示" }
                        </Link<Route>>

                        <div class="navbar-item has-dropdown is-hoverable">
                            <a class="navbar-link">
                                { "その他" }
                            </a>
                            <div class="navbar-dropdown">
                                <a class="navbar-item">
                                    <Link<Route> classes="navbar-item" to={Route::VersionInfo}>
                                    { "バージョン情報" }
                                    </Link<Route>>
                                </a>
                            </div>
                        </div>
                    </div>
                </div>
            </nav>
        }
    }
    fn switch(switch: &Route) -> Html {
        gloo::console::info!(&format!("Route::switch: {:?}", &switch));
        match switch.clone() {
            Route::Post { id } => {
                html! { <SummaryGraph id={id} /> }
            }
            Route::PostListPage { id } => todo!(),
            Route::PostList => {
                html! { <SummaryGraph id=1 /> }
            }
            Route::VersionInfo => {
                // リダイレクトする場合はweb-sys使う
                let window = web_sys::window().expect("no window find");
                let _ = window.location().set_href("https://www.google.co.jp");
                html! {}
            }
            Route::Home => {
                html! {
                    <div class="card">
                        <div class="card-content">
                        <div class="field">
                            <label class="label">{ "Email" }</label>
                            <div class="control">
                            <input class="input" type="email" placeholder="e.g. alex@example.com"/>
                            </div>
                        </div>

                        <div class="field">
                            <label class="label">{ "Password" }</label>
                            <div class="control">
                            <input class="input" type="password" placeholder="********"/>
                            </div>
                        </div>

                        <button class="button is-primary">{ "Sign in" }</button>
                        </div>
                    </div>
                }
            }
        }
    }
}

#[wasm_bindgen(inline_js = r#"
export function hide_loading() {
    const spinner = document.getElementById('loading');
    spinner.classList.add('loaded');
}
export function show_loading() {
    const spinner = document.getElementById('loading');
    spinner.classList.remove('loaded');
}
"#)]
extern "C" {
    pub fn show_loading();
    pub fn hide_loading();
}

fn main() {
    set_panic_hook();
    gloo::console::log!("hide loading start");
    hide_loading();
    gloo::console::log!("hide loading end");

    // Show off some feature flag enabling patterns.
    #[cfg(feature = "demo-abc")]
    {
        gloo::console::log!("feature `demo-abc` enabled");
    }
    #[cfg(feature = "demo-xyz")]
    {
        gloo::console::log!("feature `demo-xyz` enabled");
    }

    let element: Element = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .query_selector("#app")
        .expect("can't get body node for rendering")
        .expect("can't unwrap body node");
    yew::start_app_in_element::<App>(element);
}
