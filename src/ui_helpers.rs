use crate::search_client::TMDB;
use wasm_bindgen::UnwrapThrowExt;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Event, HtmlElement, HtmlInputElement, InputEvent};
use yew::{html, Html};

pub struct UiHelpers;

impl UiHelpers {
    pub fn get_id_from_event_elem(e: Event) -> Option<String> {
        let et = e.target().unwrap();
        let t: HtmlElement = et.dyn_into().unwrap_throw();

        Some(t.id())
    }

    pub fn get_value_from_input_by_id(id: &str) -> Option<String> {
        let mut out_value = None;
        let doc = gloo_utils::document();
        if let Ok(input_elem) = doc.query_selector(id) {
            let val = (HtmlInputElement::from(JsValue::from(input_elem))).value();
            out_value = Some(val);
        }

        out_value
    }

    pub fn get_value_from_checkbox_by_id(id: &str) -> Option<bool> {
        let mut out_value = None;
        let doc = gloo_utils::document();
        if let Ok(input_elem) = doc.query_selector(id) {
            let val = (HtmlInputElement::from(JsValue::from(input_elem))).checked();

            out_value = Some(val);
        }

        out_value
    }

    pub fn get_value_from_input_event(e: InputEvent) -> String {
        let event: Event = e.dyn_into().unwrap_throw();
        let event_target = event.target().unwrap_throw();
        let target: HtmlInputElement = event_target.dyn_into().unwrap_throw();
        target.value()
    }

    pub fn get_thumbnail(path: Option<String>) -> Html {
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
}
