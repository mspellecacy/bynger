use std::ops::{Sub};
use chrono::{Datelike, DateTime, Duration, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono::format::Fixed::TimezoneOffset;
use gloo::console::console;
use itertools::Itertools;

use serde::{Deserialize, Deserializer};
use weblog::console_log;
use yew::prelude::*;

use crate::event_calendar::EventCalendarMsg::{ChangeDate, ChangeDay};
use crate::event_manager::EventManager;
use crate::events::ScheduledEvent;
use crate::search_client::MediaType;
use crate::ui_helpers::UiHelpers;

pub trait CalendarSchedulableEvent {
    fn id(&self) -> String;
    fn name(&self) -> String;
    fn media_type(&self) -> MediaType;
    fn description(&self) -> String;
    fn duration(&self) -> usize; // in minutes
}

// impl Deserialize for dyn CalendarSchedulableEvent {
//     fn deserialize<'a, D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'a> {
//         todo!()
//     }
// }

pub struct EventCalendar {
    active_day: DateTime<Utc>,
    active_month: DateTime<Utc>,
}

pub enum EventCalendarMsg {
    ChangeDate(DateTime<Utc>),
    ChangeDay(DateTime<Utc>)
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

fn formatted_event_line(se: &ScheduledEvent) -> Html {
    // <span class="panel-icon">
    //     <i class="gg-tv" aria-hidden="true"></i>
    //     </span>
    //     {&ev.scheduled_date.format("%R")}
    // {" | "}
    // {&ev.episode.as_ref().unwrap().show_name}
    // {" - "}
    // {&ev.episode.as_ref().unwrap().name}
    //
    let text_type = match se.media_type {
        MediaType::Tv => {
            let text = format!{" {} | {} - {}",
                               se.scheduled_date.format("%R"),
                               se.episode.as_ref().unwrap().show_name,
                               se.episode.as_ref().unwrap().name };
            let type_ = "gg-tv".to_string();

            (text, type_)
        }
        MediaType::Movie => {
            let text = format!{" {} | {}",
                               se.scheduled_date.format("%R"),
                               se.movie.as_ref().unwrap().show_name };
            let type_ = "gg-film".to_string();

            (text, type_)
        }
        MediaType::Actor => {
            (String::from("Unknown Actor"), String::from("gg-boy"))
        }
        MediaType::Unknown => {
            (String::from("Unknown Actor"), String::from("gg-danger"))
        }
    };



    html!{
        <>
            <span class="panel-icon">
                <i class={text_type.1} aria-hidden="true"></i>
            </span>
            {text_type.0}
        </>
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
            active_day: current_date,
            active_month: current_date
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ChangeDate(new_date) => {
                self.active_month = new_date;
                true
            }
            ChangeDay(day) => {
                self.active_day = day;

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let day = self.active_day;
        let date = self.active_month;
        let dn = day.date_naive();

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

        let em = EventManager::create();
        // Must be mutable to sort after collection.
        let mut day_events: Vec<&ScheduledEvent> = em.events.iter()
            .filter(|se| se.scheduled_date.date_naive() == dn )
            .collect();
        // Sort our day's events by time...
        day_events.sort_by(|a, b| a.scheduled_date.cmp(&b.scheduled_date));

        let cells = get_calendar_cells(&date);
        let cell_id_format = "%Y_%m_%d";
        let day_click = ctx.link().callback(move |me:MouseEvent| {
            let mut out = day;
            if let Some(elem_id) = UiHelpers::get_id_from_event_elem(Event::from(me)) {
                let id_split: Vec<&str> = elem_id.split("_").collect();
                out = Utc.ymd(
                    id_split[0].parse::<i32>().unwrap(),
                    id_split[1].parse::<u32>().unwrap(),
                    id_split[2].parse::<u32>().unwrap())
                    .and_hms(0, 0, 1);
            }

            ChangeDay(out)
        });

        let day_val = |d: Option<NaiveDate>| {
            match d {
                None => html! {<td></td>},
                Some(d) => {
                    let day_id = d.format(cell_id_format).to_string();
                    let events: Vec<&ScheduledEvent> = em.events.iter()
                        .filter(|se| se.scheduled_date.date_naive() == d )
                        .collect();
                    html! {
                        // Even though the onclick is on the TD, nested elements trigger it and fail
                        // to pick up the ID properly. Hackily adding the ID to all the elements
                        // resolves this. Not ideal, but works without breaking anything.
                        <td class="day-link is-clickable"
                            id={day_id.clone()} onclick={&day_click}
                            title={format!("{} Events Scheduled", events.len())}>
                            <div id={day_id.clone()} class="is-inline-block">
                                {d.format("%d")}
                                if !events.is_empty() {
                                    <img id={day_id.clone()} class="events-tag" />
                                }
                            </div>
                        </td>
                    }
                }
            }
        };
        let weeks= cells.chunks(7).map(|week| {
            html!{
                <tr>
                  {
                    week.iter().map(|&d| {
                      day_val(d.clone())
                    }).collect::<Html>()
                  }
                </tr>
            }
        }).collect::<Html>();

        html! {
            <div class="is-centered box calendar-container">
                <div class="columns">
                    <div class="column calendar-left">
                        // This should probably be broken in to it's own component
                        // <CalendarCard date={date} event-manager={em} />
                        <div class="card">
                            <div class="card-header-title is-centered pb-0">
                                <div class="date-num-name-box">
                                    <div class="date-month-name">
                                        {&day.format("%B")}
                                    </div>
                                    <div class="date-day-num">
                                        {&day.format("%d")}
                                    </div>
                                    <div class="date-day-name">
                                        {&day.format("%A")}
                                    </div>
                                </div>
                            </div>
                            <div class="card-content pt-0">
                                <div class="content">
                                    <p class="subtitle">{"Schedule"}</p>
                                    {
                                        day_events.iter().map(|&ev| {
                                            html!{
                                                <a class="panel-block schedule-item">
                                                    {formatted_event_line(ev)}
                                                </a>
                                            }
                                        }).collect::<Html>()
                                    }
                                </div>
                            </div>
                            <footer class="card-footer">

                            </footer>
                        </div>
                    </div>
                    <div class="column is-three-fifths calendar-base">
                        <nav class="level">
                            // Spamming the ID so the onclick works, hacky.
                            <p class="level-left" onclick={&chevron_click} id="cal_month_prev">
                                <button class="button" id="cal_month_prev">
                                    <span class="icon is-small" id="cal_month_prev">
                                        <i class="gg-chevron-left" id="cal_month_prev"></i>
                                    </span>
                                </button>
                            </p>
                            <p class="level-item" onclick={&chevron_click}>
                                <a class="button" id="cal_month_curr">{"today"}</a>
                            </p>
                            <p class="level-item has-text-centered">
                                <div>
                                    <p class="heading">{&date.format("%Y")}</p>
                                    <p class="title">{&date.format("%B")}</p>
                                </div>
                            </p>
                            <p class="level-item" onclick={&chevron_click}>
                                <a class="button" id="cal_add_show">{"add show"}</a>
                            </p>
                            <p class="level-right" onclick={&chevron_click} id="cal_month_next">
                                <button class="button" id="cal_month_next">
                                    <span class="icon is-small" id="cal_month_next">
                                        <i class="gg-chevron-right" id="cal_month_next"></i>
                                    </span>
                                </button>
                            </p>
                        </nav>
                        <table id="bynger_cal" class="table is-fullwidth is-striped">
                            <thead>
                                <tr class="">
                                    <th>{"MON"}</th>
                                    <th>{"TUE"}</th>
                                    <th>{"WED"}</th>
                                    <th>{"THU"}</th>
                                    <th>{"FRI"}</th>
                                    <th>{"SAT"}</th>
                                    <th>{"SUN"}</th>
                                </tr>
                            </thead>
                            <tbody>
                                {weeks}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        }
    }
}