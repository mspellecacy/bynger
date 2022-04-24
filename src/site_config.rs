use std::fmt::{Display, Formatter};

use gloo::storage::{LocalStorage, Storage};

use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlInputElement};

use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub enum ConfigOptions {
    TmdbApiKey = 0,
    ScheduleEntries = 1,
}

impl Display for ConfigOptions {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let prefix = "BYNGER";
        let name = match self {
            ConfigOptions::TmdbApiKey => { "TMDB_API_KEY" }
            ConfigOptions::ScheduleEntries => { "SCHEDULE_ENTRIES" }
        };
        write!(f,"{prefix}_{name}")
    }
}

#[derive(Clone, PartialEq)]
pub struct SiteConfig {
    tmdb_api_key: Option<String>,
    schedule_entries: Option<Vec<String>>
}

pub enum SiteConfigMsg {
    Update(String),
    Save,
}

impl Component for SiteConfig {
    type Message = SiteConfigMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let tmdb_api_key = LocalStorage::get(ConfigOptions::TmdbApiKey.to_string()).unwrap_or_default();
        let schedule_entries = Some(LocalStorage::get(ConfigOptions::ScheduleEntries.to_string()).unwrap_or_else(|_| Vec::new()));

        Self {
            tmdb_api_key,
            schedule_entries
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            SiteConfigMsg::Update(key) => {
                self.tmdb_api_key = Some(key);
                false
            }
            SiteConfigMsg::Save => {
                LocalStorage::set(ConfigOptions::TmdbApiKey.to_string(), self.tmdb_api_key.clone());
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let api_key = self.tmdb_api_key.clone().unwrap_or_default();
        let onclick = ctx.link().callback(|me| SiteConfigMsg::Save);
        let onchange = ctx.link().batch_callback(|e: Event| {
            let target: Option<EventTarget> = e.target();
            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
            input.map(|input| SiteConfigMsg::Update(input.value()))
        });

        html! {
            <div class="box">
                <div class="field">
                  <label class="label">{"TMDB API Key"}</label>
                  <div class="control">
                    <input class="input" type="text" placeholder="TMDB API Key" value={api_key}
                        id={"tmdb_api_key"} {onchange} />
                  </div>
                </div>
                <div class="control">
                    <button class="button is-primary" {onclick}>{"Save"}</button>
                </div>
            </div>
        }
    }
}