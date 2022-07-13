use std::ops::{Add};
use std::str::FromStr;
use chrono::{DateTime, Duration, Local, Utc};

use gloo::storage::{LocalStorage, Storage};
use wasm_bindgen::__rt::IntoJsResult;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

use web_sys::{Element, HtmlElement, HtmlInputElement, Node, NodeList};

use weblog::{console_error, console_log};
use yew::prelude::*;

use crate::search_client::{MediaType, TMDB};
use crate::show_card::Show;
use crate::site_config::ByngerStore;
use crate::episodes_picker::EpisodePicker;
use crate::event_calendar::CalendarSchedulableEvent;
use crate::ui_helpers::UiHelpers;

#[wasm_bindgen(module="/js/helpers.js")]
extern "C" {
    #[wasm_bindgen(js_name = bc_attach_shim)]
    fn calendar_attach( selector: &str, options: &str) -> Vec<JsValue>;

    #[wasm_bindgen(js_name = bc_value_shim)]
    fn calendar_range_value(cal: &JsValue) -> String;
}

// FIXME: These structs should probably get moved in to a search_client as generic output types.
// TODO: This is a bit redundant, but eventually I would like to have a more generic search client.
#[derive(Clone, PartialEq)]
pub struct Episode {
    pub air_date: String,
    pub episode_number: usize,
    pub name: String,
    pub id: usize,
    pub season_number: usize,
    pub still_path: Option<String>
}

impl CalendarSchedulableEvent for Episode {
    fn id(&self) -> String {
        self.id.to_string()
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn media_type(&self) -> MediaType {
        MediaType::Tv
    }

    fn description(&self) -> String {
        "// Not implemented".to_string()
    }
}

#[derive(Clone, PartialEq)]
pub struct Season {
    pub id: usize,
    pub air_date: Option<String>,
    pub name: Option<String>,
    pub overview: Option<String>,
    pub poster_path: Option<String>,
    pub season_number: usize,
    pub episodes: Option<Vec<Episode>>
}

#[derive(Clone, PartialEq)]
pub enum ScheduleShowState {
    Loading,
    EpisodePicker,
    EpisodeScheduler
}

impl Default for ScheduleShowState {
    fn default() -> Self { ScheduleShowState::Loading }
}
#[derive(Clone)]
pub struct ScheduleShow {
    show: Option<Show>,
    seasons: Option<Vec<Season>>,
    episodes_to_schedule: Vec<Episode>,
    node_ref: NodeRef,
    schedule_show_state: ScheduleShowState,
    search_client: TMDB,
    range_picker: Option<JsValue>
}

#[derive(Clone, PartialEq, Properties)]
pub struct ScheduleProps {
    pub show_id: String,
    pub media_type: MediaType,
    pub on_cancel: Callback<MouseEvent>
}

pub enum ScheduleShowMsg {
    Error(String),
    Working,
    FetchShow,
    FetchSeasons,
    ShowResult(Show),
    SeasonsResult(Vec<Season>),
    ScheduleEpisodes(Vec<Episode>)
}

fn get_thumbnail(path: Option<String>) -> Html {
    match TMDB::poster_path(path) {
        None => html! {},
        Some(s) =>
            html! {
                <figure class="image">
                    <div class="has-ratio" style="width:128px;">
                        <img src={s} alt="Placeholder image" />
                    </div>
                </figure>
            }
    }
}

impl Component for ScheduleShow {
    type Message = ScheduleShowMsg;
    type Properties = ScheduleProps;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(ScheduleShowMsg::FetchShow );
        let api_key: String =  LocalStorage::get(ByngerStore::TmdbApiKey.to_string()).expect("Missing API Key");
        Self {
            show: None,
            seasons: None,
            episodes_to_schedule: vec![],
            node_ref: NodeRef::default(),
            schedule_show_state: ScheduleShowState::default(),
            search_client: TMDB::new(api_key),
            range_picker: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ScheduleShowMsg::Working => { false }
            ScheduleShowMsg::FetchShow => {
                let props = ctx.props().clone();
                let show_id = props.show_id;
                let media_type = props.media_type;
                let search_client = self.search_client.clone();
                ctx.link().send_future(async move {
                    match media_type {
                        MediaType::Tv => {
                            match search_client.get_tv(&show_id).await {
                                Ok(show) => {
                                    ScheduleShowMsg::ShowResult(Show::from(show))
                                }
                                Err(e) => {
                                    ScheduleShowMsg::Error(e)
                                }
                            }
                        }
                        MediaType::Movie => {
                            ScheduleShowMsg::Working
                        }
                        MediaType::Actor => {
                            ScheduleShowMsg::Working
                        }
                        MediaType::Unknown => {
                            ScheduleShowMsg::Working
                        }
                    }
                });
                false
            }
            ScheduleShowMsg::FetchSeasons => {
                let search_client = self.search_client.clone();
                match self.show.clone() {
                    None => {}
                    Some(show) => {
                        ctx.link().send_future(async move {
                            let seasons = search_client.get_seasons_episodes(&show.id).await;
                            match seasons {
                                None => { ScheduleShowMsg::Error("Unknown".to_string()) }
                                Some(s) => {
                                    let seasons = s.into_iter().fold(Vec::<Season>::new(), |mut seasons, so| {
                                        let eps = so.episodes.into_iter().fold(Vec::<Episode>::new(), |mut eps, ep| {
                                            eps.push(Episode {
                                                air_date: ep.air_date,
                                                episode_number: ep.episode_number,
                                                name: ep.name,
                                                id: ep.id,
                                                season_number: ep.season_number,
                                                still_path: ep.still_path
                                            });

                                            eps
                                        });

                                        seasons.push(Season {
                                            id: so.id,
                                            air_date: so.air_date,
                                            name: so.name,
                                            overview: so.overview,
                                            poster_path: so.poster_path,
                                            season_number: so.season_number,
                                            episodes: Some(eps)
                                        });

                                        seasons
                                    });

                                    ScheduleShowMsg::SeasonsResult(seasons)
                                }
                            }
                        });
                    }
                }

                false
            }
            ScheduleShowMsg::ShowResult(show) => {
                self.show = Some(show);
                ctx.link().send_future(async move {
                    ScheduleShowMsg::FetchSeasons
                });

                true
            }
            ScheduleShowMsg::SeasonsResult(seasons) => {
                self.seasons = Some(seasons);
                self.schedule_show_state = ScheduleShowState::EpisodePicker;
                true
            }
            ScheduleShowMsg::Error(e) => {
                console_error!(e);
                false
            }
            ScheduleShowMsg::ScheduleEpisodes(eps) => {
                self.episodes_to_schedule = eps;
                self.schedule_show_state = ScheduleShowState::EpisodeScheduler;

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let doc = gloo_utils::document();
        let seasons = self.seasons.clone();
        let props = ctx.props().clone();
        let _media_type = props.media_type;
        let on_cancel = move |e| { props.on_cancel.emit(e) };
        let on_distribute = ctx.link().callback(move |_| {
            // Bit of a brute force solution. I doubt its terribly performant, but I also doubt
            // anyone would actually notice given the scale we're working in.
            // Bucket-of-Eps
            let mut episodes_to_schedule: Vec<Episode> = vec![];

            // Parse through All
            if let Ok(res) = doc.query_selector_all("li.is-active [id$='tab_all']") {
                (0..=res.length()).into_iter().for_each(|r| {
                    if let Some(node) = res.get(r) {
                        // Feels like there should be a better way...what am I not understanding?
                        let tab = (HtmlElement::from(JsValue::from(node))).id();
                        let season_num: Vec<&str> = tab.split('_').collect();
                        if let Some(s) = season_num.get(1) {
                            if let Ok(season_as_usize) = usize::from_str(s) {
                                if let Some(seas) = &seasons {
                                    if let Some(found_season) = seas.iter().find(|&p| p.season_number == season_as_usize) {
                                        episodes_to_schedule.append(&mut found_season.episodes.as_ref().unwrap().clone());
                                    }
                                }
                            }
                        }
                    }
                });
            }

            // Parse through the individually selected episodes
            if let Ok(res) = doc.query_selector_all(".episode_checkbox:checked") {
                (0..=res.length()).into_iter().for_each(|r| {
                    if let Some(node) = res.get(r) {
                        let ep_id = (HtmlElement::from(JsValue::from(node))).id();
                        let split_id: Vec<&str> = ep_id.split('_').collect(); // format: season_07_episode_14
                        if let (Some(sea_num), Some(ep_num)) = (split_id.get(1), split_id.get(3)) {
                            if let (Ok(snum), Ok(enum_)) = (usize::from_str(sea_num), usize::from_str(ep_num)) {
                                if let Some(seas) = &seasons {
                                    if let Some(s) = seas.iter().find(|s| s.season_number == snum) {
                                        if let Some(eps) = &s.episodes {
                                            if let Some(ep) = eps.iter().find(|e| e.episode_number == enum_) {
                                                episodes_to_schedule.push(ep.clone());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                });
            }

            ScheduleShowMsg::ScheduleEpisodes(episodes_to_schedule)
        });
        let on_schedule = ctx.link().callback( move |_| {
            let picker_selector = "#pickerDateTimeStart".to_string();
            if let Some(mut current_dt) = UiHelpers::get_value_from_input_by_id(picker_selector) {
                let mut events_in_day = 0;

                console_log!(format!("Starting From: {:?}", current_dt));
            }

            ScheduleShowMsg::Working
        });

        let mut title = "Loading...".to_string();
        let mut subtitle = "".to_string();
        let card_body = match self.schedule_show_state {
            ScheduleShowState::Loading => {
                html!{<p>{"Loading..."}</p>}
            }
            ScheduleShowState::EpisodePicker => {
                let show = self.show.clone().unwrap();
                let seas = self.seasons.clone().expect("Missing Seasons");
                title = show.title.unwrap_or_else(|| "Loading...".to_string());
                subtitle = format!("First Aired: {} | Seasons: {}", show.first_air_date.unwrap(), show.number_of_seasons.unwrap());

                let seasons = seas.into_iter().fold(Vec::<Html>::new(), |mut acc, s| {
                    // FIXME: Feels like there should be a better way, but I dont know.
                    let overview_max_len = 300;
                    let _season_id = s.id;
                    let season_number = s.season_number;
                    let episode_count = s.episodes.clone().unwrap().len();
                    let poster_fragment = get_thumbnail(s.poster_path.clone());
                    let overview = s.overview.clone().unwrap_or_else(|| String::from("No Overview"));
                    let overview_long = || overview.len() > overview_max_len;
                    let air_date = s.air_date.clone().unwrap_or_else(|| String::from("Missing Air Date"));

                    acc.push(html!{
                        <div class="card card-season mb-3">
                            <div class="card-content pb-0 pt-1 pr-0 pl-0">
                                <div class="media mb-1">
                                    <div class="media-left pl-2">
                                        <div class="box pl-1 pr-1 pt-1 pb-1">{poster_fragment}</div>
                                    </div>
                                    <div class="media-content mb-0 pb-0">
                                        <container class="container box pt-1 pb-1 pl-1 pr-1 mr-2">
                                            <h1 class="title is-4">{format!{"Season {}", &season_number}}</h1>
                                            <h3 class="subtitle is-6 mb-1">
                                                <div>{format!{"Episodes: {}", &episode_count}}</div>
                                                <div>{format!{"First Aired: {}", &air_date}}</div>
                                            </h3>
                                            <p class="card-season-overview">{&overview}</p>
                                            {if overview_long() {
                                            // TODO: Implement description reveal.
                                            html!{
                                            <p class="card-season-overview-more is-primary">
                                                <span class="icon is-pulled-right mr-1">
                                                    <i class="gg-chevron-down-o is-large is-info"></i>
                                                </span>
                                            </p>}
                                            } else { html!{} }}
                                        </container>
                                    </div>
                                </div>
                                <div class="content mt-0 pt-0">
                                    <EpisodePicker season={Some(s)} />
                                </div>
                            </div>
                        </div>
                    });

                    acc  // Fold in season
                }); // EpisodePicker

                html!{
                    {for seasons}
                }
            }
            ScheduleShowState::EpisodeScheduler => {
                title = format!("{} Episodes to Distribute", self.episodes_to_schedule.len());
                let days_of_week = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
                let range_start = Local::now();
                let range_end = range_start.add(Duration::weeks(4));
                let date_format = "%F"; // YYYY-MM-DD
                let time_format = "%R"; // HH:
                let start_date_string = range_start.format(date_format).to_string();
                let start_time_string = range_start.format(time_format).to_string();
                let end_date_string = range_end.format(date_format).to_string();
                let end_time_string = range_end.format(time_format).to_string();

                html!{
                    <div>
                    <form id="schedulerForm">
                    <div class="box">
                        <div>
                            <h1 class="is-size-4">{"Distribution Date and Time Limits"}</h1>
                            // <h3 class="subtitle is-6">{"Leave end date empty for opened ended scheduling."}</h3>
                        </div>
                        <div class="field mb-0 is-horizontal">
                            <div class="field-label is-normal">
                                <label class="label">{"Start Date"}</label>
                            </div>
                            <div class="field-body">
                                <div class="field">
                                    <div class="control">
                                        <input id="pickerDateStart" type="date" value={start_date_string} />
                                    </div>
                                </div>
                            </div>
                        </div>
                        <div class="field mb-0 is-horizontal">
                            <div class="field-label is-normal">
                                <label class="label">{"Start Time"}</label>
                            </div>
                            <div class="field-body">
                                <div class="field">
                                    <div class="control">
                                        <input id="pickerTimeStart" type="time" value={start_time_string} />
                                    </div>
                                </div>
                            </div>
                        </div>
                        <div class="field mb-0 is-horizontal">
                            <div class="field-label is-normal">
                                <label class="label">{"End Time"}</label>
                            </div>
                            <div class="field-body">
                                <div class="field">
                                    <div class="control">
                                        <input id="pickerTimeStart" type="time" value={end_time_string} />
                                    </div>
                                </div>
                            </div>
                        </div>
                        <div class="field mb-0 is-horizontal">
                            <div class="field-label is-normal">
                                <label class="label">{"End Date"}</label>
                            </div>
                            <div class="field-body">
                                <div class="field">
                                    <div class="control">
                                        <input id="pickerDateEnd" type="date" value={end_date_string} />
                                        {"\u{00a0}\u{00a0}\u{00a0}"} // a few &nbsp;
                                        <input class="is-checkradio is-success"
                                                id={"checkbox_use_end_date"}
                                                type="checkbox"
                                                checked=true
                                                // TODO: Checkbox should disable End Date input
                                                // onchange={}
                                        />
                                        <label for={format!("checkbox_use_end_date")}>
                                            {"Use End Date"}
                                        </label>
                                    </div>
                                </div>
                            </div>
                        </div>
                        </div>
                        // <input class="input"
                        //     type="datetime"
                        //     id="pickerDateTimeRange"
                        //     data-is-range="true"
                        //     data-display-mode="dialog"
                        //     data-show-header="false"
                        //     data-validate-label="Save"
                        //     data-minute-steps="1"
                        //     data-label-from="Start Date/Time"
                        //     data-label-to="End Date/Time"
                        //     data-start-date={format!("{}", range_start.format(date_format))}
                        //     data-start-time={format!("{}", range_start.format(time_format))}
                        //     data-end-date={format!("{}", range_end.format(date_format))}
                        //     data-end-time={format!("{}", range_end.format(time_format))}
                        // />
                        <div class="box">
                            <h1 class="is-size-4">{"Scheduling Options"}</h1>
                            <div class="columns is-variable">
                                <div class="column">
                                    <div>
                                        <p>{"Days of Week"}</p>
                                        <div class="columns is-multiline is-gapless">
                                            {
                                                days_of_week.into_iter().enumerate().map(|(idx, day)| {
                                                    html!{
                                                        <div class="column is-half">
                                                            <div class="field">
                                                                <input class="is-checkradio is-success"
                                                                        id={format!("checkbox_{day}")}
                                                                        type="checkbox"
                                                                        checked={matches!(idx, 0..=4)}
                                                                />
                                                                <label for={format!("checkbox_{day}")}>
                                                                    {day}
                                                                </label>
                                                            </div>
                                                        </div>
                                                    }
                                                }).collect::<Html>()
                                            }
                                        </div>
                                    </div>
                                </div>
                                <div class="column">
                                    <div class="mb-1">
                                        <p>{"Episodes per Day"}</p>
                                        <div>
                                            <div class="select">
                                                <select id="episodesPerDay">
                                                    // 6 Eps a day? Might as well fill...
                                                    {(1..=6).into_iter().map(|idx|
                                                        html!{ <option value={idx.to_string()} selected={idx==1}>{idx}</option> }
                                                    ).collect::<Html>()}
                                                    <option value="0">{"Fill"}</option>
                                                </select>
                                            </div>
                                        </div>
                                    </div>
                                    <br />
                                    <div>
                                        <div class="field">
                                            <input class="is-checkradio is-success"
                                                    id="noEndDatetime"
                                                    type="checkbox"
                                                    checked=false
                                            />
                                            <label for="ignoreEndDate">
                                                {"Ignore End Date"}
                                            </label>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </form>
                    </div>
                }
            }
        };

        html! {
            <div class="modal is-active">
                <div class="modal-background"></div>
                <div class="modal-card">
                    <header class="modal-card-head pb-1 pt-1 pl-1 pr-1">
                        <div class="modal-card-title mt-0 mb-0 p-0">
                            <h1 class="title">{title}</h1>
                            <h2 class="subtitle">{subtitle}</h2>
                        </div>
                        <button class="delete is-large pl-1" aria-label="close" onclick={on_cancel}></button>
                    </header>
                     <section class="modal-card-body pb-1 pt-1">
                        {card_body}
                     </section>
                    <footer class="modal-card-foot pb-1 pt-1">
                        if self.schedule_show_state == ScheduleShowState::EpisodePicker {
                            <button class="button" onclick={&on_distribute}>{"Distribute Episodes"}</button>
                            // <button class="button control" onclick={on_cancel}>{"Cancel"}</button>
                        } if self.schedule_show_state == ScheduleShowState::EpisodeScheduler {
                            <button class="button" onclick={&on_schedule}>{"Schedule"}</button>
                        }
                    </footer>
                </div>
            </div>
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        match self.schedule_show_state {
            ScheduleShowState::Loading => {}
            ScheduleShowState::EpisodePicker => {}
            ScheduleShowState::EpisodeScheduler => {
                // if self.range_picker.is_none() {
                //     // attach always returns an array, we just want the first (and only) picker.
                //     let cal: JsValue = calendar_attach("[id*='picker']", "")[0].clone();
                //     self.range_picker = Some(cal);
                // }
            }
        }
    }
}