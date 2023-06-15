use chrono::{DateTime, Utc};
use crate::events::ScheduledEvent;
use crate::search_client::{MediaType, TMDB};
use crate::site_config::ByngerStore;
use gloo::storage::{LocalStorage, Storage};
use uuid::Uuid;

use yew::prelude::*;
use yew::{html, Callback, Component, Context, Html};
use crate::datetime_picker::DateTimePicker;

#[derive(Clone, PartialEq, Properties)]
pub struct EventDetailsProps {
    pub scheduled_event: ScheduledEvent,
    pub onclosed: Callback<bool>,
    pub onremove: Callback<Uuid>,
    pub onwatched: Callback<Uuid>,
    pub onreschedule: Callback<(Uuid, DateTime<Utc>)>
}

#[derive(Debug, Clone, PartialEq)]
pub struct Details {
    pub image: String,
    pub title: String,
    pub subtitle: String,
    pub overview: String,
    pub air_date: String,
    pub runtime: usize,
    pub rating: String,
}

pub struct EventDetails {
    details: Option<Details>,
}

pub enum EventDetailsMsg {
    Loading,
    DetailsLoaded(Details),
}

impl Component for EventDetails {
    type Message = EventDetailsMsg;
    type Properties = EventDetailsProps;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(EventDetailsMsg::Loading);
        EventDetails { details: None }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            EventDetailsMsg::Loading => {
                let _ev = ctx.props().scheduled_event.clone();
                ctx.link().send_future(async move {
                    let sc = TMDB::new(
                        LocalStorage::get(ByngerStore::TmdbApiKey.to_string())
                            .expect("Missing API Key"),
                    );

                    match _ev.media_type {
                        MediaType::tv => {
                            let _ep = _ev.episode.as_ref().unwrap().clone();
                            let res = sc
                                .get_tv_season_episode(
                                    &_ep.show_id.to_string(),
                                    _ep.season_number,
                                    _ep.episode_number,
                                )
                                .await;
                            match res {
                                None => EventDetailsMsg::Loading,
                                Some(e) => {
                                    let det = Details {
                                        image: TMDB::poster_path(e.still_path)
                                            .unwrap_or("".to_string()),
                                        title: e.name.to_string(),
                                        subtitle: format!(
                                            "{} | Season: {} Episode: {}",
                                            _ep.show_name, e.season_number, e.episode_number                                        ),
                                        overview: e.overview.unwrap(),
                                        air_date: e.air_date.unwrap(),
                                        runtime: e.runtime.unwrap(),
                                        rating: "".to_string(),
                                    };

                                    EventDetailsMsg::DetailsLoaded(det)
                                }
                            }
                        }
                        MediaType::movie => {
                            let _mv = _ev.movie.as_ref().unwrap().clone();
                            let res = sc.get_movie(&_mv.movie_id.to_string()).await;
                            match res {
                                Err(_e) => EventDetailsMsg::Loading,
                                Ok(m) => {
                                    let det = Details {
                                        image: TMDB::poster_path(m.poster_path)
                                            .unwrap_or("".to_string()),
                                        title: m.title.expect("Missing Title"),
                                        subtitle: format!(
                                            "Released: {} - Runtime: {}",
                                            m.release_date.unwrap(),
                                            m.runtime.unwrap()
                                        ),
                                        overview: m.overview.unwrap(),
                                        air_date: "".to_string(),
                                        runtime: m.runtime.unwrap(),
                                        rating: "".to_string(),
                                    };

                                    EventDetailsMsg::DetailsLoaded(det)
                                }
                            }
                        }
                        _ => EventDetailsMsg::Loading,
                        // MediaType::actor => {}
                        // MediaType::person => {}
                        // MediaType::unknown => {}
                    }
                });

                false
            }
            EventDetailsMsg::DetailsLoaded(det) => {
                // console_log!(format!("{det:?}"));
                self.details = Some(det);

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let event = ctx.props().scheduled_event.clone();
        // Feels like there should be a better way, but this works.
        let oce = ctx.props().onclosed.clone();
        let ore = ctx.props().onremove.clone();
        let owe = ctx.props().onwatched.clone();
        let ors = ctx.props().onreschedule.clone();
        let onclose = Callback::from(move |_| oce.emit(true));
        let onremove = Callback::from(move |_| ore.emit(event.uuid));
        let onwatched = Callback::from(move |_| owe.emit(event.uuid));
        let onreschedule = Callback::from(move |dt:DateTime<Utc>| ors.emit((event.uuid, dt.clone())));


        let mut watched_class = classes!("button", "is-info");
        if event.watched {
            watched_class.push(classes!("event-watched", "is-outlined"));
        }

        html! {
            if let Some(det) = &self.details {
                <div class="modal is-active">
                    <div class="modal-background"></div>
                    <div class="modal-card">
                        <header class="modal-card-head pb-1 pt-1 pl-1 pr-1">
                            <div class="modal-card-title mt-0 mb-0 p-0">
                                <h1 class="title">{&det.title.to_string()}</h1>
                                <h2 class="subtitle">{&det.subtitle.to_string()}</h2>
                            </div>
                            <button class="delete is-large pl-1" aria-label="close" onclick={onclose}></button>
                        </header>
                         <section class="modal-card-body">
                            <div class="columns">
                                <div class="column is-three-fifths">
                                    <p>{&det.overview.to_string()}</p>
                                </div>
                                <div class="column">
                                    <figure class="image">
                                        <img src={det.image.to_string()} alt="Placeholder image" />
                                    </figure>
                                </div>
                            </div>
                            <div class="box">
                                <div class="field has-addons has-addons-centered">
                                    <p class="control">
                                        <DateTimePicker label="RESCHEDULE" onclick={onreschedule}/>
                                    </p>
                                </div>
                                <div class="field is-grouped is-grouped-centered">
                                    <p class="control">
                                        <button class={watched_class} aria-label="watched" onclick={onwatched}>
                                            {"WATCHED"}
                                        </button>
                                    </p>
                                    <p class="control">
                                        <button class="button is-danger" aria-label="remove" onclick={onremove}>
                                            {"REMOVE"}
                                        </button>
                                    </p>
                                </div>
                            </div>
                         </section>
                    </div>
                </div>
            }

        }
    }
}
