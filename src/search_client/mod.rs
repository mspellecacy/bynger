use std::fmt::{Display, Formatter};

use std::rc::Rc;

use futures::future::try_join_all;

use reqwasm::http::Request;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use weblog::{console_error, console_log};
use yew::Context;

use crate::FindShow;

// Linter really doesn't like lowercase enum variants, but it matches the TMDB return values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Hash, Eq)]
pub enum MediaType {
    tv,
    movie,
    actor,
    person,
    unknown,
}

impl Default for MediaType {
    fn default() -> Self {
        Self::unknown
    }
}

impl From<MediaType> for String {
    fn from(mt: MediaType) -> Self {
        match mt {
            MediaType::tv => "tv",
            MediaType::movie => "movie",
            MediaType::actor => "actor",
            MediaType::person => "person",
            MediaType::unknown => "unknown",
        }
        .to_string()
    }
}

impl From<MediaType> for &str {
    fn from(mt: MediaType) -> Self {
        match mt {
            MediaType::tv => "tv",
            MediaType::movie => "movie",
            MediaType::actor => "actor",
            MediaType::person => "person",
            MediaType::unknown => "unknown",
        }
    }
}

impl Display for MediaType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mt = match self {
            MediaType::tv => "tv",
            MediaType::movie => "movie",
            MediaType::actor => "actor",
            MediaType::person => "person",
            MediaType::unknown => "unknown",
        };
        write!(f, "{mt}")
    }
}

impl FromStr for MediaType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mt = match s.to_lowercase().as_str() {
            "movie" => Self::movie,
            "tv" => Self::tv,
            "actor" => Self::actor,
            "person" => Self::person,
            _ => Self::unknown,
        };

        Ok(mt)
    }
}
impl From<String> for MediaType {
    fn from(s: String) -> MediaType {
        MediaType::from_str(s.as_str()).unwrap()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub media_type: MediaType,
    pub year: String,
}

pub trait SearchClient {
    fn by_title(&self, title: String, page: Option<usize>) -> Result<Vec<SearchResult>, String>;
    fn by_id(&self, id: String, ctx: &&Context<FindShow>);
}

#[derive(Clone, PartialEq, Eq)]
pub struct TMDB {
    pub api_key: Rc<String>,
}

#[derive(Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TMDBTVObj {
    pub id: usize,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub original_name: Option<String>,
    #[serde(default)]
    pub overview: Option<String>,
    #[serde(default)]
    pub first_air_date: Option<String>,
    #[serde(default)]
    pub last_air_date: Option<String>,
    #[serde(default)]
    pub number_of_seasons: usize,
    #[serde(default)]
    pub number_of_episodes: usize,
    #[serde(default)]
    pub episode_run_time: Vec<usize>,
    #[serde(default)]
    pub poster_path: Option<String>, // Can be null
    #[serde(default)]
    pub backdrop_path: Option<String>, // Can be null
    #[serde(default)]
    pub in_production: bool,
    #[serde(default)]
    pub tagline: Option<String>,

    #[serde(default)]
    pub seasons: Vec<TMDBSeasonObj>,
}

#[derive(Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TMDBMovieObj {
    pub id: usize,
    #[serde(default)]
    pub imdb_id: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub original_title: Option<String>,
    #[serde(default)]
    pub overview: Option<String>,
    #[serde(default)]
    pub release_date: Option<String>,
    #[serde(default)]
    pub poster_path: Option<String>, // Can be null
    #[serde(default)]
    pub backdrop_path: Option<String>, // Can be null
    #[serde(default)]
    pub tagline: Option<String>,
    #[serde(default)]
    pub runtime: Option<usize>,
    #[serde(default)]
    pub status: Option<String>,
}

// Hacky
#[derive(Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TMDBResult {
    // -- Shared by TV&Movie, Actor also has a limited subset I dont really care about.
    id: usize,
    #[serde(default)]
    media_type: MediaType,
    #[serde(default)]
    genre_ids: Vec<usize>,
    #[serde(default)]
    overview: String,
    #[serde(default)]
    original_language: String,

    // -- TV
    #[serde(default)]
    name: String,
    #[serde(default)]
    original_name: String,
    #[serde(default)]
    first_air_date: String,

    // -- Movie
    #[serde(default)]
    release_date: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    original_title: String,
}

#[derive(Default, Clone, PartialEq, Eq)]
pub struct SearchResponse {
    pub page: usize,
    pub results: Vec<SearchResult>,
    pub total_pages: usize,
    pub total_results: usize,
}

#[derive(Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TMDBSearchResponse {
    page: usize,
    results: Vec<TMDBResult>,
    total_pages: usize,
    total_results: usize,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TMDBEpisodeObj {
    #[serde(default)]
    pub air_date: Option<String>,
    #[serde(default)]
    pub episode_number: usize,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub id: usize,
    #[serde(default)]
    pub season_number: usize,
    #[serde(default)]
    pub still_path: Option<String>,
    #[serde(default)]
    pub overview: Option<String>,
    // #[serde(default)]
    // pub vote_average: Option<String>, // actually return a float I believe.
    // #[serde(default)]
    // pub vote_count: Option<usize>,
    #[serde(default)]
    pub runtime: Option<usize>,
}

#[derive(Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TMDBSeasonObj {
    #[serde(default)]
    pub air_date: Option<String>,
    #[serde(default)]
    pub id: usize,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub overview: Option<String>,
    #[serde(default)]
    pub poster_path: Option<String>,
    #[serde(default)]
    pub season_number: usize,
    #[serde(default)]
    pub episodes: Vec<TMDBEpisodeObj>,
}

impl TMDB {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key: Rc::new(api_key),
        }
    }

    pub fn poster_path(path: Option<String>) -> Option<String> {
        // FIXME: At some point involve the actual config options from TMDB.
        let base = "https://image.tmdb.org/t/p/w500/";
        path.map(|p| format!("{base}{p}"))
    }

    pub async fn search_title(
        &self,
        title: &String,
        page: &Option<usize>,
    ) -> Result<SearchResponse, String> {
        let url = format!("https://api.themoviedb.org/3/search/multi?api_key={}&query={}&page={}&include_adult=false", &self.api_key, title, page.unwrap());
        let mut out = vec![];
        let res = Request::get(&url).send().await;

        match res {
            Ok(r) => {
                match r.json::<TMDBSearchResponse>().await {
                    Ok(tsr) => {
                        tsr.results
                            .into_iter()
                            // Should eventually support "actor"
                            .filter(|p| matches!(p.media_type, MediaType::tv | MediaType::movie))
                            .map(|r| SearchResult {
                                id: r.id.to_string(),
                                title: Some(r.title).get_or_insert(r.name).to_string(),
                                media_type: r.media_type,
                                year: r.release_date,
                            })
                            .for_each(|s| out.push(s));

                        Ok(SearchResponse {
                            results: out,
                            page: tsr.page,
                            total_pages: tsr.total_pages,
                            total_results: tsr.total_results,
                        })
                    }
                    Err(e) => {
                        // FIXME: Return actual Errors.
                        console_log!("Json Parsing Error.");
                        console_error!("Error:", e.to_string());

                        Err(e.to_string())
                    }
                }
            }
            Err(e) => Err(e.to_string()),
        }
    }

    pub async fn get_movie(&self, id: &String) -> Result<TMDBMovieObj, String> {
        let key = &self.api_key;
        let url =
            format!("https://api.themoviedb.org/3/movie/{id}?api_key={key}&include_adult=false");
        let res = Request::get(&url).send().await;

        match res {
            Ok(r) => match r.json::<TMDBMovieObj>().await {
                Ok(m) => Ok(m),
                Err(e) => {
                    console_error!(e.to_string());
                    Err(e.to_string())
                }
            },
            Err(e) => {
                console_error!(e.to_string());
                Err(e.to_string())
            }
        }
    }

    pub async fn get_tv(&self, id: &String) -> Result<TMDBTVObj, String> {
        let key = &self.api_key;
        let url = format!("https://api.themoviedb.org/3/tv/{id}?api_key={key}&include_adult=false");
        let res = Request::get(&url).send().await;

        match res {
            Ok(r) => {
                match r.json::<TMDBTVObj>().await {
                    Ok(r) => Ok(r),
                    Err(e) => {
                        // Bad deserialization of json
                        console_error!(e.to_string());
                        Err(e.to_string())
                    }
                }
            }
            Err(e) => {
                // Bad response from TMDB...
                console_error!(e.to_string());
                Err(e.to_string())
            }
        }
    }

    pub async fn get_tv_season(&self, id: &String, season: usize) -> Option<TMDBSeasonObj> {
        let key = self.api_key.clone();
        let base = format!("https://api.themoviedb.org/3/tv/{id}/season/{season}");
        let postfix = format!("?api_key={key}");
        let url = format!("{base}{postfix}");

        match Request::get(&url).send().await {
            Ok(res) => match res.json::<TMDBSeasonObj>().await {
                Ok(season) => Some(season),
                Err(e) => {
                    console_log!(format!("Failed parsing JSON - {e}"));
                    None
                }
            },
            Err(e) => {
                console_log!(format!("Failed Season Request - {e}"));
                None
            }
        }
    }

    pub async fn get_tv_season_episode(
        &self,
        id: &String,
        season: usize,
        episode: usize,
    ) -> Option<TMDBEpisodeObj> {
        let key = self.api_key.clone();
        let base =
            format!("https://api.themoviedb.org/3/tv/{id}/season/{season}/episode/{episode}");
        let postfix = format!("?api_key={key}");
        let url = format!("{base}{postfix}");

        match Request::get(&url).send().await {
            Ok(res) => match res.json::<TMDBEpisodeObj>().await {
                Ok(episode) => Some(episode),
                Err(e) => {
                    console_log!(format!("Failed parsing JSON - {e}"));
                    None
                }
            },
            Err(e) => {
                console_log!(format!("Failed Season Episode Request - {e}"));
                None
            }
        }
    }

    pub async fn get_seasons_episodes(&self, id: &String) -> Option<Vec<TMDBSeasonObj>> {
        match self.get_tv(id).await {
            Ok(show) => {
                let seasons: Vec<_> = (1..=show.number_of_seasons)
                    .into_iter()
                    .map(|sn| async move {
                        let season = async { self.get_tv_season(id, sn).await };
                        match season.await {
                            None => Err("Bad Request"),
                            Some(res) => Ok(res),
                        }
                    })
                    .collect();

                match try_join_all(seasons).await {
                    Ok(seasons) => Some(seasons),
                    Err(e) => {
                        console_log!(format!("Failed TV Season Fetch Join - {e}"));
                        None
                    }
                }
            }
            Err(e) => {
                console_log!(format!("Failed TV Season Fetch - {e}"));
                None
            }
        }
    }
}
