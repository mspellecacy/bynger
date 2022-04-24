use yew::prelude::*;

struct TvShow {
    tmdb_id: String,
    imdb_id: String,
    overview: String,
}

pub struct TvCard {
    tv_show: Option<TvShow>,
}

pub enum TvCardMsg {
    Loading,
    Loaded,
}

impl Component for TvCard {
    type Message = TvCardMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self { tv_show: None }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {}
    }
}
