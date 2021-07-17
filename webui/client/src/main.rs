#![recursion_limit = "1024"]

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use console_error_panic_hook::set_once as set_panic_hook;
use wasm_bindgen::prelude::*;
use yew::prelude::*;
#[cfg(feature = "demo-abc")]
use yew::services::ConsoleService;
use yew_router::{route::Route, switch::Permissive};
mod switch;
use switch::{AppAnchor, AppRoute, AppRouter, PublicUrlSwitch};

mod components;
use components::summary_graph::SummaryGraph;

pub enum Msg {
    ToggleNavbar,
}

pub struct App {
    link: ComponentLink<Self>,
    navbar_active: bool,
}

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            navbar_active: false,
        }
    }

    fn update(&mut self, _: Self::Message) -> bool {
        false
    }

    fn change(&mut self, _: Self::Properties) -> bool {
        false
    }

    fn view(&self) -> Html {
        html! {
            <>
                {self.view_nav()}
                <main>
                    <AppRouter
                        render=AppRouter::render(Self::switch)
                        redirect=AppRouter::redirect(|route: Route| {
                            AppRoute::PageNotFound(Permissive(Some(route.route))).into_public()
                        })
                    />
                </main>
            </>
        }
    }
}

impl App {
    fn view_nav(&self) -> Html {
        let navbar_active = self.navbar_active;
        let active_class = if navbar_active { "is-active" } else { "" };
        html! {
            <nav class="navbar is-primary" role="navigation" aria-label="main navigation">
                <div class="navbar-brand">
                    <h1 class="navbar-item is-size-3">{ "Wakatime Summary" }</h1>

                    <a role="button"
                        class=classes!("navbar-burger", "burger", active_class)
                        aria-label="menu" aria-expanded="false"
                    >
                        <span aria-hidden="true"></span>
                        <span aria-hidden="true"></span>
                        <span aria-hidden="true"></span>
                    </a>
                </div>
                <div class=classes!("navbar-menu", active_class)>
                    <div class="navbar-start">
                        <AppAnchor classes="navbar-item" route=AppRoute::Home>
                            { "ホーム" }
                        </AppAnchor>
                        <AppAnchor classes="navbar-item" route=AppRoute::PostList>
                            { "投稿" }
                        </AppAnchor>

                        <div class="navbar-item has-dropdown is-hoverable">
                            <a class="navbar-link">
                                { "もっと" }
                            </a>
                            <div class="navbar-dropdown">
                                <a class="navbar-item">
                                    <AppAnchor classes="navbar-item" route=AppRoute::AuthorList>
                                    { "制作者に合う" }
                                    </AppAnchor>
                                </a>
                            </div>
                        </div>
                    </div>
                </div>
            </nav>
        }
    }
    fn switch(switch: PublicUrlSwitch) -> Html {
        match switch.route() {
            AppRoute::Post(id) => {
                html! { <SummaryGraph id=id /> }
            }
            AppRoute::PostListPage(_) => todo!(),
            AppRoute::PostList => todo!(),
            AppRoute::Author(_) => todo!(),
            AppRoute::AuthorList => todo!(),
            AppRoute::PageNotFound(_) => {
                html! { <div>{ "ページがみつかりませーん" }</div> }
            }
            AppRoute::Home => {
                html! {
                    <>
                    <div class="box">
                        { "This text is <em>not</em> within a <strong>block</strong>." }
                    </div>
                    <div class="card">
                        <form class="card-content">
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
                        </form>
                    </div>
                    </>
                }
            }
        }
    }
}

#[wasm_bindgen(inline_js = "export function snippetTest() { console.log('Hello from JS FFI!'); }")]
extern "C" {
    fn snippetTest();
}

fn main() {
    set_panic_hook();
    unsafe {
        snippetTest();
    }

    // Show off some feature flag enabling patterns.
    #[cfg(feature = "demo-abc")]
    {
        ConsoleService::log("feature `demo-abc` enabled");
    }
    #[cfg(feature = "demo-xyz")]
    {
        ConsoleService::log("feature `demo-xyz` enabled");
    }

    yew::start_app::<App>();
}
