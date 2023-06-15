use chrono::{DateTime, Utc, Local, NaiveDate, NaiveTime};
use uuid::Uuid;
use yew::{Callback, Classes, classes, Component, Context, Html, html, Properties};
use crate::ui_helpers::UiHelpers;

#[derive(Debug, PartialEq, Properties)]
pub struct DateTimePickerProps {
    #[prop_or_default]
    pub class: Classes,
    #[prop_or("SAVE".to_string())]
    pub label: String,
    #[prop_or(DateTimePickerType::DateTime)]
    pub picker: DateTimePickerType,
    #[prop_or(Utc::now())]
    pub start_datetime: DateTime<Utc>,
    #[prop_or(Utc::now())]
    pub end_datetime: DateTime<Utc>,

    pub onclick: Callback<DateTime<Utc>>,

}

#[derive(Debug, PartialEq)]
pub enum DateTimePickerType {
    DateTime,
    DateTimeRange, // TODO: Implement DateTimeRange and replace schedule_show implementation.
}

pub struct DateTimePicker;

impl Component for DateTimePicker {
    type Message = ();
    type Properties = DateTimePickerProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let date_format = "%F"; // YYYY-MM-DD
        let time_format = "%R"; // HH:
        let start_date_string = ctx.props().start_datetime.format(date_format).to_string();
        let start_time_string = ctx.props().end_datetime.format(time_format).to_string();
        let oce = ctx.props().onclick.clone();
        let onclick = Callback::from(move |dt| {
            let raw_start_date = UiHelpers::get_value_from_input_by_id("#pickerDateStart")
                .expect("Missing Start Date?");
            let raw_start_time = UiHelpers::get_value_from_input_by_id("#pickerTimeStart")
                .expect("Missing Start Time?");

            let (mut new_date, mut new_time) = (
                NaiveDate::parse_from_str(&raw_start_date, "%Y-%m-%d")
                    .expect("Bad start date format."),
                NaiveTime::parse_from_str(&raw_start_time, "%H:%M")
                    .expect("Bad start time format."),
            );

            oce.emit(new_date.and_time(new_time).and_utc())
        });


        html!{
            <div class={ctx.props().class.clone()}>
                <form id="dateTimePickerForm">
                    <div class="field has-addons">
                        <p class="control">
                            <input class="input" id="pickerDateStart" type="date" value={start_date_string} />
                        </p>
                        <p class="control">
                            <input class="input" id="pickerTimeStart" type="time" value={start_time_string} />
                        </p>
                        <p class="control">
                            <button class="button is-primary" aria-label="watched" {onclick}>
                                 {ctx.props().label.clone()}
                            </button>
                        </p>
                    </div>
                </form>
            </div>
        }
    }
}