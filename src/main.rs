extern crate core;

use gloo::storage::errors::StorageError;
use gloo::storage::{LocalStorage, Storage};

use yew::prelude::*;
use yew::virtual_dom::VNode;
use yew_router::prelude::*;

mod episodes_picker;
mod event_calendar;
mod event_manager;
mod events;
mod find_show;
mod schedule_show;
mod search_client;
mod show_card;
mod site_config;
mod tv_card;
mod ui_helpers;

use crate::event_calendar::EventCalendar;
use crate::find_show::FindShow;
use crate::site_config::{ByngerStore, SiteConfig};

#[derive(Routable, PartialEq, Eq, Clone, Debug)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/schedule")]
    Schedule,
    #[at("/config")]
    Config,
}

pub struct Bynger {
    nav_open: bool,
    api_key: Result<String, StorageError>,
}
pub enum ByngerMsg {
    ToggleNav,
}

impl Component for Bynger {
    type Message = ByngerMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let api_key: Result<String, StorageError> =
            LocalStorage::get(ByngerStore::TmdbApiKey.to_string());
        Self {
            nav_open: false,
            api_key,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ByngerMsg::ToggleNav => {
                self.nav_open = !self.nav_open;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let is_active = if self.nav_open { "is-active" } else { "" };
        let toggle_nav = ctx.link().callback(|_| ByngerMsg::ToggleNav);
        html! {
            <BrowserRouter>
                <nav class="navbar" role="navigation" aria-label="main navigation">
                    <div class="navbar-brand">
                        <a class="navbar-item" href="/">
                            <span class="icon">
                                <i class="gg-bot"></i>
                            </span>
                        </a>
                        <a role="button" class={classes!("navbar-burger", is_active)}
                            aria-label="menu"
                            aria-expanded="false"
                            onclick={&toggle_nav}>
                          <span aria-hidden="true"></span>
                          <span aria-hidden="true"></span>
                          <span aria-hidden="true"></span>
                        </a>
                    </div>
                    <div class={classes!("navbar-menu", is_active)}>
                        <div class="navbar-start">
                            <div onclick={&toggle_nav} class="navbar-item">
                                <Link<Route> to={Route::Home}>{ "Home" }</Link<Route>>
                            </div>
                            <div onclick={&toggle_nav} class="navbar-item">
                                <Link<Route> to={Route::Schedule}>{ "Schedule" }</Link<Route>>
                            </div>
                            <div onclick={&toggle_nav} class="navbar-item">
                                <Link<Route> to={Route::Config}>{ "Config" }</Link<Route>>
                            </div>
                        </div>
                    </div>
                </nav>
                <div class="container">
                    <main class="columns is-centered">
                        <Switch<Route> render={Switch::render(switch)} />
                    </main>
                </div>
            </BrowserRouter>
        }
    }
}

fn switch(routes: &Route) -> Html {
    let api_key: Result<String, StorageError> =
        LocalStorage::get(ByngerStore::TmdbApiKey.to_string());

    // Redirect to config if TMDB API Key doesn't exist or is empty.
    // Dont redirect if we're already going to config (otherwise infinite redirect)
    let main = if (api_key.is_err() || api_key.expect("").is_empty()) && routes != &Route::Config {
        html! { <Redirect<Route> to={Route::Config}/> }
    } else {
        match routes {
            Route::Home => {
                html! { <EventCalendar /> }
            }
            Route::Schedule => {
                html! { <FindShow /> }
            }
            Route::Config => {
                html! { <SiteConfig /> }
            }
        }
    };

    // Stops the linter from complaining about returning () instead of Html
    main as VNode
}

fn main() {
    yew::start_app::<Bynger>();
}
