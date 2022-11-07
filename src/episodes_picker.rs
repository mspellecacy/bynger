use std::ops::Not;

use itertools::Itertools;
use web_sys::{Event, HtmlElement};

use crate::schedule_show::Season;
use crate::search_client::TMDB;
use wasm_bindgen::JsCast;
use wasm_bindgen::UnwrapThrowExt;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub enum EpisodePickerTab {
    // This will kinda do double-duty as state for this component.
    All,
    Some,
    None,
}

pub struct EpisodePicker {
    episode_picker_tab: EpisodePickerTab,
    season: Option<Season>,
}

pub enum EpisodePickerMsg {
    Working,
    PickerTabChange(Event),
}

#[derive(Clone, PartialEq, Properties)]
pub struct EpisodePickerProperties {
    pub season: Option<Season>,
}

// FIXME: Need a basic utility struct instead of duping code.
fn get_thumbnail(path: Option<String>) -> Html {
    match TMDB::poster_path(path) {
        None => html! {},
        Some(s) => html! {
            <figure class="image">
                <div class="has-ratio" style="width:128px;">
                    <img src={s} alt="Placeholder image" />
                </div>
            </figure>
        },
    }
}

impl Component for EpisodePicker {
    type Message = EpisodePickerMsg;
    type Properties = EpisodePickerProperties;

    fn create(ctx: &Context<Self>) -> Self {
        let episode_picker_tab = EpisodePickerTab::Some;
        let season = ctx.props().season.clone();

        Self {
            episode_picker_tab,
            season,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            EpisodePickerMsg::PickerTabChange(e) => {
                let id = e
                    .target()
                    .unwrap()
                    .dyn_into::<HtmlElement>()
                    .unwrap_throw()
                    .id();
                let tab = id.split('_').collect::<Vec<&str>>().pop();
                match tab {
                    Some(tab_name) => {
                        match tab_name.to_uppercase().as_str() {
                            "ALL" => self.episode_picker_tab = EpisodePickerTab::All,
                            "SOME" => self.episode_picker_tab = EpisodePickerTab::Some,
                            "NONE" => self.episode_picker_tab = EpisodePickerTab::None,
                            _ => {}
                        }
                        true
                    }
                    _ => {
                        // Ignore click
                        false
                    }
                }
            }
            EpisodePickerMsg::Working => false,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let season = self.season.clone().unwrap();
        // Supports <=999 seasons? mmk.
        let season_number = format!("{:002}", season.season_number);
        let current_tab = &self.episode_picker_tab;
        let season_id_base = format!("season_{season_number}");
        let tab_id_base = format!("{season_id_base}_tab");
        let tab_click = ctx
            .link()
            .callback(move |me: MouseEvent| EpisodePickerMsg::PickerTabChange(Event::from(me)));

        match current_tab {
            EpisodePickerTab::Some => {
                let episodes = season.episodes.unwrap();
                // I want 3 columns of eps, no matter how many episodes there are per column.
                // This wont result in the most perfectly even columns, but it works well enough.
                let chunk_size = (episodes.len() as f32 / 3_f32).ceil();

                let season_chunks: Vec<Html> = (0..=episodes.len())
                    .chunks(chunk_size as usize)
                    .into_iter()
                    .map(|chunks| {
                        let eps = chunks.into_iter()
                            .fold(Vec::new(), |mut acc, r| {
                                if let Some(e) = episodes.get(r) {
                                        let checkbox_id = format!("{season_id_base}_episode_{:002}", e.episode_number);
                                        acc.push(html!{
                                            <div class="field">
                                            <label class="checkbox">
                                                <input class="episode_checkbox" id={checkbox_id.to_string()} type="checkbox" checked=true />
                                                {format!("\u{00A0} {} | {}", e.episode_number, e.name)}
                                            </label>
                                            </div>
                                         });
                                }

                                acc
                            });

                        html!{ <div class="column is-one-third">{for eps }</div> }
                    }).collect_vec();

                html! {
                    <>
                    <div class="tabs is-centered mb-1">
                        <ul class="mt-0 ml-0 mb-1">
                            <li onclick={&tab_click}>
                                <a><span id={format!("{tab_id_base}_all")}>{"All"}</span></a>
                            </li>
                            <li onclick={&tab_click} class="is-active">
                                <a><span id={format!("{tab_id_base}_some")}>{"Some"}</span></a>
                            </li>
                            <li onclick={&tab_click}>
                                <a><span id={format!("{tab_id_base}_none")}>{"None"}</span></a>
                            </li>
                        </ul>
                    </div>

                    <div class="box episode-box">
                        <div class="columns is-multiline">
                            {for season_chunks}
                        </div>
                    </div>
                    </>
                }
            }
            all_or_none => {
                let is_all = all_or_none == &EpisodePickerTab::All;
                html! {
                    <div class="tabs is-centered">
                        <ul class="mt-0 ml-0">
                            <li onclick={&tab_click}
                                class={classes!(is_all.then(|| Some("is-active")))}>
                                <a><span id={format!("{tab_id_base}_all")}>{"All"}</span></a>
                            </li>
                            <li onclick={&tab_click}>
                                <a><span id={format!("{tab_id_base}_some")}>{"Some"}</span></a>
                            </li>
                            <li onclick={&tab_click}
                                class={classes!(is_all.not().then(|| Some("is-active")))}>
                                <a><span id={format!("{tab_id_base}_none")}>{"None"}</span></a>
                            </li>
                        </ul>
                    </div>
                }
            }
        }
    }
}
