extern crate core;

use gloo::storage::errors::StorageError;
use gloo::storage::{LocalStorage, Storage};

use yew::prelude::*;
use yew::{html, Component, Context, Html};
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

    fn create(_ctx: &Context<Self>) -> Self {
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
                    <div class="navbar-brand"></div>
                    <div class="navbar-menu">
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
                            <Link<Route> to={Route::Home} classes="navbar-item">{ "Home" }</Link<Route>>
                            <Link<Route> to={Route::Schedule} classes="navbar-item">{ "Schedule" }</Link<Route>>
                            <Link<Route> to={Route::Config} classes="navbar-item">{ "Config" }</Link<Route>>
                        </div>
                        <div class="navbar-end">
                            <a class="navbar-item" href="https://github.com/mspellecacy/bynger" target="_blank"> {"Github"} </a>
                        </div>
                    </div>
                </nav>
                <div class="container">
                    <main class="columns is-centered">
                        <Switch<Route> render={switch} />
                    </main>
                </div>
            </BrowserRouter>
        }
    }
}

fn switch(routes: Route) -> Html {
    // FIXME: We're reading this every page change.
    // I spent a couple hours trying to extract it and make it read only when necessary but created
    // a different problem, so I reverted the changes.
    let api_key: Result<String, StorageError> =
        LocalStorage::get(ByngerStore::TmdbApiKey.to_string());

    // Redirect to config if TMDB API Key doesn't exist or is empty.
    // Dont redirect if we're already going to config (otherwise infinite redirect)
    if (api_key.is_err() || api_key.expect("").is_empty()) && routes != Route::Config {
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
    }
}

fn main() {
    yew::Renderer::<Bynger>::new().render();
}
