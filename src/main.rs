extern crate core;

use gloo::storage::{LocalStorage, Storage};
use gloo::storage::errors::StorageError;

use yew::prelude::*;
use yew_router::prelude::*;

mod find_show;
mod search_client;
mod show_card;
mod tv_card;
mod schedule_show;
mod site_config;
mod episodes_picker;
mod event_calendar;
mod ui_helpers;
mod events;
mod event_manager;

use crate::find_show::FindShow;
use crate::site_config::{ByngerStore, SiteConfig};
use crate::event_calendar::EventCalendar;

#[derive(Routable, PartialEq, Clone, Debug)]
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
    ToggleNav
}

impl Component for Bynger {
    type Message = ByngerMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let api_key: Result<String, StorageError> = LocalStorage::get(ByngerStore::TmdbApiKey.to_string());
        Self {
            nav_open: false,
            api_key
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
    let api_key: Result<String, StorageError> = LocalStorage::get(ByngerStore::TmdbApiKey.to_string());

    // Dont redirect if we're already going to config (otherwise infinite redirect)
    let main = if api_key.is_err() && routes != &Route::Config {
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

    main
}

fn main() {
    yew::start_app::<Bynger>();
}
