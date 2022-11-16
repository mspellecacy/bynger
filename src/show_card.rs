use crate::search_client::{MediaType, TMDBMovieObj, TMDBTVObj, TMDB};
use std::fmt::Display;
use weblog::console_error;
use yew::prelude::*;
use yew::virtual_dom::VNode;

#[derive(Default, Clone, PartialEq, Eq, Properties)]
pub struct Show {
    pub id: String,                     // tmdb/tv: id
    pub title: Option<String>,          // tmdb/tv: name
    pub original_title: Option<String>, // tmdb/tv: original_name
    pub first_air_date: Option<String>,
    pub poster: Option<String>,
    pub media_type: MediaType,
    pub episode_run_time: Option<Vec<usize>>,
    pub last_air_date: Option<String>,
    pub number_of_episodes: Option<usize>,
    pub number_of_seasons: Option<usize>,
    pub overview: Option<String>,
    pub tagline: Option<String>,
    pub in_production: bool,
}

impl From<TMDBTVObj> for Show {
    fn from(t: TMDBTVObj) -> Self {
        let poster = match (t.poster_path, t.backdrop_path) {
            (Some(pp), None) => Some(pp),
            (None, Some(bp)) => Some(bp),
            (Some(pp), Some(_bp)) => Some(pp),
            _ => None,
        };

        Self {
            id: format!("{}", t.id),
            title: t.name,
            original_title: t.original_name,
            first_air_date: t.first_air_date,
            poster: TMDB::poster_path(poster),
            media_type: MediaType::tv,
            episode_run_time: Some(t.episode_run_time),
            last_air_date: t.last_air_date,
            number_of_episodes: Some(t.number_of_episodes),
            number_of_seasons: Some(t.number_of_seasons),
            overview: t.overview,
            tagline: t.tagline,
            in_production: t.in_production,
        }
    }
}

// Since I'm trying to use ShowCard somewhat generically we'll finagle a Movie in to a Show for now.
impl From<TMDBMovieObj> for Show {
    fn from(m: TMDBMovieObj) -> Self {
        // Not DRY and should probably be handled by TMDB class to give back the full and proper URL
        let poster = match (m.poster_path, m.backdrop_path) {
            (Some(pp), None) => Some(pp),
            (None, Some(bp)) => Some(bp),
            (Some(pp), Some(_bp)) => Some(pp),
            _ => None,
        };

        Self {
            id: format!("{}", m.id),
            title: m.title,
            original_title: m.original_title,
            first_air_date: m.release_date,
            poster: TMDB::poster_path(poster),
            media_type: MediaType::movie,
            episode_run_time: Some(vec![m.runtime.unwrap_or(0)]),
            last_air_date: None,
            in_production: false,
            number_of_episodes: None,
            number_of_seasons: None,
            overview: m.overview,
            tagline: m.tagline,
        }
    }
}

#[derive(Default, Clone)]
pub struct ShowCard {}

pub enum ShowCardMsg {
    Loading,
    LoadingError(String),
    ShowFound(Show),
    ShowClicked,
}
impl From<()> for ShowCardMsg {
    fn from(_val: ()) -> Self {
        Self::Loading
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct ShowCardProps {
    pub show: Show,
    pub onclick: Callback<(String, MediaType)>,
}

impl ShowCard {
    fn get_thumbnail(path: Option<String>) -> Html {
        let thumb = match path {
            None => html! {},
            Some(s) => html! {
                <figure class="image is-2by3">
                    <img class="is-radiusless" src={s} alt="Placeholder image" />
                </figure>
            },
        };

        thumb as VNode
    }

    fn value_into_pair<T: Display>(name: &str, item: &Option<T>) -> Html {
        let nbsp = '\u{00a0}'.to_string();
        let template = move |k, v| -> Html {
            let out = html! {
                <p class="level">
                    <span class="show-item has-text-weight-semibold">{k}</span>
                    <span class="show-item">{v}</span>
                </p>
            };

            out
        };

        match item {
            // None => html!{ <p class="level"><span class="show-item">{'\u{00a0}'}</span></p> },
            None => template(&nbsp, &nbsp),
            Some(val) => template(&name.to_string(), &val.to_string()),
        }
    }
}

impl Component for ShowCard {
    type Message = ShowCardMsg;
    type Properties = ShowCardProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ShowCardMsg::Loading => true,
            ShowCardMsg::ShowFound(show) => {
                // self.show = Some(show);
                true
            }
            ShowCardMsg::LoadingError(e) => {
                console_error!(e);
                false
            }
            ShowCardMsg::ShowClicked => {
                let _out = (
                    ctx.props().show.id.clone(),
                    ctx.props().show.media_type.clone(),
                );
                ctx.props().onclick.emit(_out);
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onclick = ctx
            .link()
            .callback(|_e: MouseEvent| ShowCardMsg::ShowClicked);

        let view = match Some(ctx.props().show.clone()) {
            None => {
                html! { /* do nothing */ }
            }
            Some(s) => {
                let title = &s.title.unwrap_or_default();
                let season_count = ShowCard::value_into_pair("Seasons", &s.number_of_seasons);
                let episode_count = ShowCard::value_into_pair("Episodes", &s.number_of_episodes);
                let air_date = ShowCard::value_into_pair("First Aired", &s.first_air_date);

                html! {
                    <div class="card show-card pl-0 pr-0" data-id={s.id}>
                         // Show Title Header ...
                        <header class="card-header is-shadowless">
                            <p class="card-header-title pl-1">{&title}</p>
                        </header>
                        // Card image ...
                        <div class="card-image">
                            { ShowCard::get_thumbnail(s.poster.clone()) }
                        </div>

                        // Basic show info
                        <div class="card-content">
                            <ul>
                                <li>{air_date}</li>
                                <li>{season_count}</li>
                                <li>{episode_count}</li>
                            </ul>
                        </div>
                        <footer class="card-footer">
                            <div class="card-footer-item pl-1 pr-1 pt-0 pb-0" onclick={onclick}>
                                <div class="is-clickable pl-0">{"Pick"}</div>
                            </div>
                        </footer>
                    </div>
                }
            }
        };

        view as VNode
    }
}
