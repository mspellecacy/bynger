use gloo::storage::{LocalStorage, Storage};



use weblog::{console_error};
use yew::prelude::*;

use crate::search_client::{MediaType, TMDB};
use crate::show_card::Show;
use crate::site_config::ConfigOptions;

// FIXME: These structs should probably get moved in to a search_client as generic output types.
// TODO: This is a bit redundant, but eventually I would like to have a more generic search client.
#[derive(Clone)]
pub struct Episode {
    pub air_date: String,
    pub episode_number: usize,
    pub name: String,
    pub id: usize,
    pub season_number: usize,
    pub still_path: Option<String>
}

#[derive(Clone)]
pub struct Season {
    pub id: usize,
    pub air_date: Option<String>,
    pub name: Option<String>,
    pub overview: Option<String>,
    pub poster_path: Option<String>,
    pub season_number: usize,
    pub episodes: Option<Vec<Episode>>
}

#[derive(Clone)]
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
    schedule_show_state: ScheduleShowState,
    search_client: TMDB,
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
    SeasonsResult(Vec<Season>)
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
        let api_key: String =  LocalStorage::get(ConfigOptions::TmdbApiKey.to_string()).expect("Missing API Key");
        Self {
            show: None,
            seasons: None,
            schedule_show_state: ScheduleShowState::default(),
            search_client: TMDB::new(api_key),
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
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props().clone();
        let _media_type = props.media_type;
        let on_cancel = move |e| { props.on_cancel.emit(e) };
        let mut title = "Loading...".to_string();
        let mut subtitle = "Sub title...".to_string();
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
                    let overview_max_len = 350;
                    let _season_id = s.id;
                    let season_number = s.season_number;
                    let season_episodes = s.episodes.unwrap();
                    let episode_count = &season_episodes.len();
                    let poster_fragment = get_thumbnail(s.poster_path);
                    let overview = s.overview.unwrap_or(String::from("No Overview"));
                    let overview_long = || overview.len() > overview_max_len;
                    let air_date = s.air_date.unwrap_or(String::from("Missing Air Date"));

                    let _season_header = format!("{} - {}", &air_date, &overview);
                    let _season_episodes = season_episodes.into_iter()
                        .fold(Vec::<Html>::new(), |mut episodes, e| {
                            let li_entry = format!("{} - {} - {}", e.episode_number, e.air_date, e.name);
                            episodes.push(html!{<li>{li_entry}</li>});

                            episodes
                        });

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
                                        //
                                        <div class="tabs is-centered is-boxed">
                                            <ul class="mt-0 ml-0">
                                                <li class="is-active">
                                                    <a><span>{"All"}</span></a>
                                                </li>
                                                <li>
                                                    <a><span>{"Some"}</span></a>
                                                </li>
                                                <li>
                                                    <a><span>{"None"}</span></a>
                                                </li>
                                            </ul>
                                        </div>
                                        // <div class="block columns">
                                        //     <div class="column">
                                        //         <p class="bd-notification is-info">{"First column"}</p>
                                        //         <div class="columns is-mobile">
                                        //             <div class="column">
                                        //                 <p class="bd-notification is-info">{"First nested column"}</p>
                                        //             </div>
                                        //             <div class="column">
                                        //                 <p class="bd-notification is-info">{"Second nested column"}</p>
                                        //             </div>
                                        //         </div>
                                        //     </div>
                                        //     <div class="column">
                                        //         <p class="bd-notification is-danger">{"Second column"}</p>
                                        //         <div class="columns is-mobile">
                                        //             <div class="column is-half">
                                        //                 <p class="bd-notification is-danger">{"50%"}</p>
                                        //             </div>
                                        //             <div class="column">
                                        //                 <p class="bd-notification is-danger">{"Auto"}</p>
                                        //             </div>
                                        //             <div class="column">
                                        //                 <p class="bd-notification is-danger">{"Auto"}</p>
                                        //             </div>
                                        //         </div>
                                        //     </div>
                                        // </div>
                                        //
                                </div>
                            </div>
                        </div>
                    });

                    acc  // Fold
                });

                html!{
                    {for seasons}
                }
            }
            ScheduleShowState::EpisodeScheduler => {
                html!{
                    <p>{"Scheduler goes here ..."}</p>
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
                        <button class="delete is-large" aria-label="close" onclick={on_cancel}></button>
                    </header>
                     <section class="modal-card-body pb-1 pt-1">
                        {card_body}
                     </section>
                    <footer class="modal-card-foot pb-1 pt-1">
                        //<button class="button control" onclick={on_cancel}>{"Cancel"}</button>
                    </footer>
                </div>
            </div>
        }
    }
}