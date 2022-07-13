use std::collections::HashMap;
use std::str::FromStr;
use futures::StreamExt;
use gloo::storage::{LocalStorage, Storage};
use itertools::Itertools;
use wasm_bindgen::JsCast;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::{HtmlInputElement};
use web_sys::InputEvent;
use web_sys::{Event};
use weblog::{console_info, console_log};
use yew::prelude::*;


use crate::search_client::{MediaType, SearchResponse, TMDB};
use crate::show_card::{Show, ShowCard};
use crate::site_config::ByngerStore;
use crate::schedule_show::ScheduleShow;
use crate::ui_helpers::UiHelpers;

pub struct FindShow {
    modal_state: FindShowModalState,
    search_client: TMDB,
    search_value: String,
    search_results: HashMap<usize, SearchResponse>,
    show_cache: HashMap<(String, MediaType), Show>,
    current_page: Option<usize>,
    max_page: Option<usize>,
    show_selected: Option<(String, MediaType)>,
}

pub enum FindShowModalState {
    Closed = 0,
    Searching = 1,
    Scheduling = 2,
}

impl Default for FindShowModalState {
    fn default() -> Self {
        FindShowModalState::Closed
    }
}

#[derive(Clone, PartialEq)]
pub enum FindShowMsg {
    Click,
    CloseModal,
    InputChange(String),
    Search,
    Working,
    SearchComplete(SearchResponse),
    ShowResult(Show),
    GetShow((String, MediaType)),
    ShowSelected((String, MediaType)),
    PageRequest(usize),
    ScrollHandler,
}

impl FromStr for FindShowMsg {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Search" => Ok(Self::Search),
            &_ => Err(()),
        }
    }
}

impl From<()> for FindShowMsg {
    fn from(_val: ()) -> Self {
        FindShowMsg::Working
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct SearchProps {
    pub value: String,
    pub on_change: Callback<String>,
    pub search_request: Callback<bool>,
    node_ref: NodeRef,
}

#[function_component(SearchInput)]
pub fn search_input(props: &SearchProps) -> Html {
    let SearchProps {
        value,
        on_change,
        search_request,
        node_ref: _,
    } = props.clone();


    let oninput = Callback::from(move |ie: InputEvent| {
        let event: Event = ie.dyn_into().unwrap_throw();
        let event_target = event.target().unwrap_throw();
        let target: HtmlInputElement = event_target.dyn_into().unwrap_throw();

        // Emit our new input value
        on_change.emit(target.value())
    });

    let onkeydown = Callback::from(move |k: KeyboardEvent|
        // Pressing 'enter' in the search box should initiate the search.
        if k.key_code() == 13 { search_request.emit(true) }
    );

    use_effect_with_deps(move |node_ref| {
        if let Some(input) = node_ref.cast::<HtmlInputElement>() {
            input.focus();
        } || ()
    }, props.node_ref.clone());

    html! {
          <div class="control has-icons-right">
            <input ref={props.node_ref.clone()}
            class="input" type="text" placeholder="Show Title" {value} {oninput} {onkeydown} />
            <span class="icon is-small is-right">
                <i class="gg-search"></i>
            </span>
          </div>
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct SearchResultsProps {
    pub show_selected: Callback<(String, MediaType)>,   // returns id/type on item click.
    pub page_request: Callback<usize>,                  // Returns the number of the page requested.
    pub shows: HashMap<usize, Show>,                    // Collection of Shows
    #[prop_or(3)]                                       // Default to 3-wide
    pub columns: usize,                                 // N-columns to group by.
}

#[function_component(SearchResults)]
pub fn search_results(props: &SearchResultsProps) -> Html {
    let emitter = props.show_selected.clone();
    let onclick = Callback::from(move |v| emitter.emit(v));
    let other = "pb-1 pl-1"; //"pl-1 pr-1 mr-1 ml-1";
    let col_is = format!("column {other} is-{}", 12 / props.columns);

    (0..props.shows.len())
        .chunks(props.columns)
        .into_iter()
        .map(|chunks| {
            let columns = chunks
                .into_iter()
                .fold(Vec::new(), |mut acc, r| {
                    if let Some(s) = props.shows.get(&r){
                        acc.push(html! {
                            <div key={s.id.clone()} class={col_is.to_owned()}>
                                <ShowCard show={s.to_owned()} onclick={&onclick} />
                            </div>
                        });
                    }

                    acc
            });

            html!{ <div class="columns"> {for columns } </div> }
       }).collect::<Html>()
}

impl Component for FindShow {
    type Message = FindShowMsg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let api_key: String =  LocalStorage::get(ByngerStore::TmdbApiKey.to_string()).expect("Missing API Key");
        Self {
            modal_state: Default::default(),
            search_client: TMDB::new(api_key),
            search_value: "".to_string(),
            search_results: HashMap::new(),
            show_cache: HashMap::new(),
            current_page: Some(1),
            max_page: Some(1),
            show_selected: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            FindShowMsg::Click => {
                self.modal_state = FindShowModalState::Searching;
                true
            }
            FindShowMsg::CloseModal => {
                self.show_selected = None;
                self.search_results.clear();
                self.search_value.clear();
                self.current_page = Some(1);
                self.modal_state = FindShowModalState::Closed;
                true
            }
            FindShowMsg::InputChange(input) => {
                self.search_value = input;
                self.search_results.clear();
                self.current_page = Some(1);
                false
            }
            FindShowMsg::Search => {
                let sv = self.search_value.clone();
                let cp = self.current_page;
                let sc = self.search_client.clone();
                ctx.link().send_future(async move {
                    let res =  sc.search_title(&sv, &cp).await;
                    match res {
                        Ok(sr) => {
                            FindShowMsg::SearchComplete(sr)
                        },
                        Err(_) => FindShowMsg::Working,
                    }
                });
                true
            }
            FindShowMsg::Working => false,
            FindShowMsg::SearchComplete(res) => {
                let cache = &mut self.show_cache;
                res.clone().results.into_iter()
                    .map(|r| (r.id, r.media_type))
                    .filter(|p| !cache.contains_key(p))
                    .for_each(|r| {
                        ctx.link().send_future(async move { FindShowMsg::GetShow(r)})
                    });

                self.current_page = Some(res.page);
                self.max_page = Some(res.total_pages);
                self.search_results.insert(res.page, res);

                true
            }

            FindShowMsg::ShowSelected((i, m)) => {
                self.show_selected = Some((i, m));
                self.modal_state = FindShowModalState::Scheduling;
                true
            }
            FindShowMsg::ScrollHandler => {
                console_log!(String::from("You did a scroll?"));
                false
            }
            FindShowMsg::PageRequest(page) => {
                console_info!("Page Requested: ", page);
                self.current_page = Some(page);
                let _sv = self.search_value.clone();
                let sr = self.search_results.get(&page);
                match sr {
                    None => {
                        console_info!(format!(
                            "Fetching results for {} - Page: {}",
                            self.search_value,
                            self.current_page.unwrap()
                        ));

                        ctx.link().send_future(async { FindShowMsg::Search });
                    }
                    Some(sr) => {
                        console_info!(format!(
                            "Existing results for {} - Page: {}",
                            self.search_value,
                            self.current_page.unwrap()
                        ));
                        self.current_page = Some(sr.page);
                    }
                }
                true
            }
            FindShowMsg::ShowResult(show) => {
                let key = (show.id.clone(), show.media_type.clone());
                self.show_cache.insert(key, show);

                true
            }
            FindShowMsg::GetShow(req) => {
                let (id, mt) = req;
                let sc = self.search_client.clone();
                match mt {
                    MediaType::Tv => {
                        ctx.link().send_future( async move {
                            match sc.get_tv(&id).await {
                                Ok(res) => {
                                    let show = Show::from(res);
                                    FindShowMsg::ShowResult(show)
                                }
                                Err(e) => {
                                    console_log!(e);
                                    FindShowMsg::Working
                                }
                            }
                        });
                    }
                    MediaType::Movie => {
                        ctx.link().send_future( async move {
                            match sc.get_movie(&id).await {
                                Ok(res) => {
                                    let show = Show::from(res);
                                    FindShowMsg::ShowResult(show)
                                }
                                Err(e) => {
                                    console_log!(e);
                                    FindShowMsg::Working
                                }
                            }
                        });
                    }
                    unknown => {
                        console_log!(String::from(unknown));
                    }
                }

                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onclick = ctx.link().callback(|_| FindShowMsg::Click);
        let closemodal = ctx.link().callback(|_| FindShowMsg::CloseModal);
        let on_change = ctx.link().callback(FindShowMsg::InputChange);
        let onsearch = ctx.link().callback(|_| FindShowMsg::Search);
        let search_request = ctx.link().callback(|_| FindShowMsg::Search);
        let show_selected = ctx
            .link()
            .callback(|(id, media_type)| FindShowMsg::ShowSelected((id, media_type)));
        let page_request = ctx.link().callback(FindShowMsg::PageRequest);
        let find_show_fragment = html! {
                <div class="box">
                    <div class="control">
                        <button class="button" {onclick}>{ "Find Show" }</button>
                    </div>
                </div>
            };

        match self.modal_state {
            FindShowModalState::Closed => find_show_fragment,
            FindShowModalState::Searching => {
                let have_response = self.search_results.get(&self.current_page.unwrap());
                let search_node_ref = NodeRef::default();
                let search_nav = match have_response {
                    Some(_) => {
                        let crr = self.current_page.unwrap();
                        let max = self.max_page.unwrap();
                        let onclick = ctx.link().callback(move |me: MouseEvent| {
                            let id = UiHelpers::get_id_from_event_elem(Event::from(me));
                            match id {
                                None => FindShowMsg::Working,
                                Some(p) => {
                                    let page_num = p.strip_prefix("page_link_");
                                    let nav_link = p.strip_prefix("nav_");

                                    match (page_num, nav_link) {
                                        (Some(page), None) => {
                                            FindShowMsg::PageRequest(page.parse::<usize>().unwrap())
                                        }
                                        (None, Some(nav)) => {
                                            let next_page = if nav == "next" {
                                                if (crr + 1) > max {
                                                    max
                                                } else {
                                                    crr + 1
                                                }
                                            } else if nav == "prev" {
                                                if (crr - 1) > 0 {
                                                    crr - 1
                                                } else {
                                                    1
                                                }
                                            } else {
                                                crr
                                            };

                                            FindShowMsg::PageRequest(next_page)
                                        }
                                        _ => {
                                            console_log!("Hmmm...");
                                            FindShowMsg::Working
                                        }
                                    }
                                }
                            }
                        });
                        let page_links = (1..=max).into_iter().map(|i| {
                            let is_current = if i == crr { "is-current" } else { "" };
                            let display_value = i;
                            let id = format!("page_link_{i}");
                            let aria = format!("Go to Page {display_value}");
                            let class = format!("pagination-link {is_current}");

                            html! { <li key={i}><a onclick={&onclick} id={id} class={class} aria-label={aria}>{display_value}</a></li> }
                        }).collect::<Html>();

                        html! {
                            <div class="columns pb-1 pl-1 mb-1">
                                <div class="column is-12 pl-0 pb-1 ml-0">
                                    <nav class="pagination is-centered is-small"
                                         role="navigation"
                                         aria-label="pagination">
                                      <a class="pagination-previous" onclick={&onclick}>
                                        <span class="icon is-left">
                                            <i class="gg-chevron-left" id="nav_prev"></i>
                                        </span>
                                      </a>
                                      <a class="pagination-next" onclick={&onclick}>
                                          <span class="icon is-left">
                                            <i class="gg-chevron-right" id="nav_next"></i>
                                          </span>
                                      </a>
                                      <ul class="pagination-list">
                                        {page_links}
                                      </ul>
                                    </nav>
                                </div>
                            </div>
                        }
                    }
                    None => {
                        html! { /* no links */ }
                    }
                };

                let search_content = match have_response {
                    Some(r) => {

                        let shows = r.results.clone().into_iter()
                            .fold(HashMap::<usize, Show>::new(), |mut acc, r| {
                                let key = acc.len();
                                let show = self.show_cache.get(&(r.id, r.media_type));
                                match show {
                                    None => {}
                                    Some(s) => {
                                        acc.insert(key, s.clone());
                                    }
                                }

                                acc
                            });
                        html! { <SearchResults {show_selected} {page_request} columns=4 shows={shows} /> }
                    }
                    None => {
                        html! { <h1>{"No Results"}</h1> }
                    }
                };

                html! {
                    <>
                    {find_show_fragment}
                    <div class="modal is-active" id="search-modal">
                        <div class="modal-background"></div>
                        <div class="modal-card">
                            <header class="modal-card-head pb-1 pt-1 pl-1 pr-1">
                                <div class="modal-card-title mt-0 mb-0 pr-1">
                                    <div class="field has-addons">
                                         <div class="control">
                                            <a class="button" onclick={onsearch}>{"Search"}</a>
                                        </div>
                                        <div class="control is-expanded">
                                            <SearchInput value={self.search_value.to_owned()} {on_change} {search_request} node_ref={search_node_ref}/>
                                        </div>
                                     </div>
                                </div>
                                <button class="delete is-large" aria-label="close" onclick={&closemodal}></button>
                            </header>
                            <section class="modal-card-body pb-1 pt-1" id="search-results">
                                {search_nav}
                                {search_content}
                            </section>
                            <footer class="modal-card-foot pb-1 pt-1" >
                            //    <button class="button" onclick={&closemodal}>{"Cancel"}</button>
                            </footer>
                        </div>
                    </div>
                    </>
                }
            }
            FindShowModalState::Scheduling => {
                let (show_id, media_type) = self.show_selected.clone().unwrap();
                let on_cancel = &closemodal;

                html! {
                    <>
                    {find_show_fragment}
                    <ScheduleShow {show_id} {media_type} {on_cancel}/>
                    </>
                }
            }
        }
    }
}
