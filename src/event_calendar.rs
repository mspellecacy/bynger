use std::ops::{Sub};
use chrono::{Datelike, DateTime, Duration, NaiveDate, NaiveTime, TimeZone, Utc};
use yew::prelude::*;

use crate::event_calendar::EventCalendarMsg::ChangeDate;
use crate::search_client::MediaType;
use crate::ui_helpers::UiHelpers;


pub trait CalendarSchedulableEvent {
    fn id(&self) -> String;
    fn name(&self) -> String;
    fn media_type(&self) -> MediaType;
    fn description(&self) -> String;
}

pub struct EventCalendar {
    current_date: DateTime<Utc>
}

pub enum EventCalendarMsg {
    ChangeDate(DateTime<Utc>)
}

#[derive(Clone, PartialEq, Properties)]
pub struct EventCalendarProperties {
    #[prop_or(Utc::now())]
    pub date: DateTime<Utc>,
    #[prop_or_default]
    pub events: Vec<String>,
}

fn get_calendar_cells(date: &DateTime<Utc>) -> Vec<Option<NaiveDate>> {
    let mut cells: Vec<Option<NaiveDate>> = vec![];
    let year = date.year();
    let month = date.month();
    let month_start = NaiveDate::from_ymd(year, month, 1);
    let month_start_from_monday = month_start.weekday().number_from_monday();
    let month_end = match month+1 == 13 {
        true => NaiveDate::from_ymd(year, 12, 31),
        false => NaiveDate::from_ymd(year, month+1, 1).sub(Duration::days(1))
    };

    // start cell padding.
    for _ in 1..month_start_from_monday { cells.push(None); }
    // insert days...
    for day_number in 1..=month_end.day() { cells.push(Some(NaiveDate::from_ymd(year, month, day_number))); }
    // end cell padding
    while cells.len()%7 != 0 { cells.push(None); }

    cells
}

fn formatted_calendar_cell(date: &Option<NaiveDate>) -> Html {
    match date {
        None => html!{<td></td>},
        Some(d) => {
            let day = format!{"{}", d.day()};
            html!{
                <td>
                    <div>
                        <p>{day}</p>
                        <p class="is-size-7">{"// Event 1"}</p>
                        <p class="is-size-7">{"// Event 2"}</p>
                        <p class="is-size-7">{"// Event 3"}</p>
                        <p class="is-size-7">{"// Event 4"}</p>
                    </div>
                </td>
            }
        }
    }
}

// TODO: Pretty sure the TimeZone management on this component is wonk.
// TODO: Implement a configuration setting for what TimeZone to use by default for display.
impl Component for EventCalendar {
    type Message = EventCalendarMsg;
    type Properties = EventCalendarProperties;

    fn create(ctx: &Context<Self>) -> Self {
        let current_date = ctx.props().date;
        Self {
            current_date
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ChangeDate(new_date) => {
                self.current_date = new_date;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let date = self.current_date;

        let chevron_click = ctx.link().callback(move |me:MouseEvent| {
            let mut out_date = date;
            if let Some(elem_id) = UiHelpers::get_id_from_event_elem(Event::from(me)) {
                if let Some(direction) = elem_id.strip_prefix("cal_month_") {
                    let mut year = date.year();
                    let mut month = date.month();
                    match direction {
                        "next" => {
                            if month+1 == 13 {
                                month = 1;
                                year += 1;
                            } else {
                                month += 1;
                            }
                            let next_month = NaiveDate::from_ymd(year, month, 1);
                            out_date = DateTime::from_utc(next_month.and_time(NaiveTime::from_hms(0, 0, 1)), Utc);
                        },
                        "prev" => {
                            let prev_month = NaiveDate::from_ymd(year, month, 1).sub(Duration::days(1));
                            out_date = DateTime::from_utc(prev_month.and_time(NaiveTime::from_hms(0, 0, 1)), Utc);
                        },
                        "curr" => {
                            out_date = Utc::now();
                        }
                        _ => unreachable!()
                    }
                }
            }
            ChangeDate(out_date)
        });

        let cells = get_calendar_cells(&date);
        let weeks= cells.chunks(7).map(|week| {
            let _day_val = move |val: Option<NaiveDate>| {
              match val {
                  None => String::from(""),
                  Some(val) => format!{"{}", val.day()}
              }
            };

            html!{
                <tr>
                  {
                    week.iter().map(|&day| {
                      formatted_calendar_cell(&day)
                    }).collect::<Html>()
                  }
                </tr>
            }
        }).collect::<Html>();

        html! {
            <div class="box">
                <nav class="level">
                    <p class="level-left" onclick={&chevron_click}>
                        <button class="button">
                            <span class="icon is-small">
                                <i class="gg-chevron-left" id="cal_month_prev"></i>
                            </span>
                        </button>
                    </p>
                    <p class="level-item" onclick={&chevron_click}>
                        <a class="button" id="cal_month_curr">{"today"}</a>
                    </p>
                    <p class="level-item has-text-centered">
                        <div>
                            <p class="heading">{&date.format("%Y").to_string()}</p>
                            <p class="title">{&date.format("%B").to_string()}</p>
                        </div>
                    </p>
                    <p class="level-item" onclick={&chevron_click}>
                        <a class="button" id="cal_add_show">{"add show"}</a>
                    </p>
                    <p class="level-right" onclick={&chevron_click}>
                        <button class="button">
                            <span class="icon is-small">
                                <i class="gg-chevron-right" id="cal_month_next"></i>
                            </span>
                        </button>
                    </p>
                </nav>
                <table id="bynger_cal" class="table is-striped is-fullwidth">
                    <thead>
                        <tr>
                            <th>{"Monday"}</th>
                            <th>{"Tuesday"}</th>
                            <th>{"Wednesday"}</th>
                            <th>{"Thursday"}</th>
                            <th>{"Friday"}</th>
                            <th>{"Saturday"}</th>
                            <th>{"Sunday"}</th>
                        </tr>
                    </thead>
                    <tbody>
                        {weeks}
                    </tbody>
                </table>
            </div>
        }
    }
}