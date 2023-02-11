use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveTime, TimeZone, Utc};
use std::ops::Sub;
use wasm_bindgen::prelude::wasm_bindgen;

use yew::prelude::*;

use crate::event_calendar::EventCalendarMsg::{ChangeDate, ChangeDay};
use crate::event_manager::{CsvType, EventManager};
use crate::events::ScheduledEvent;
use crate::search_client::MediaType;
use crate::ui_helpers::UiHelpers;

#[wasm_bindgen(module = "/js/helpers.js")]
extern "C" {
    #[wasm_bindgen(js_name = export_file)]
    fn export_file(filename: &str, data: &str, data_type: &str);
}

pub trait CalendarSchedulableEvent {
    fn id(&self) -> String;
    fn name(&self) -> String;
    fn media_type(&self) -> MediaType;
    fn description(&self) -> String;
    fn duration(&self) -> usize; // in minutes
}

pub struct EventCalendar {
    active_day: DateTime<Utc>,
    active_month: DateTime<Utc>,
}

pub enum EventCalendarMsg {
    ChangeDate(DateTime<Utc>),
    ChangeDay(DateTime<Utc>),
    ExportCsv,
}

#[derive(Clone, PartialEq, Eq, Properties)]
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
    let month_start = NaiveDate::from_ymd_opt(year, month, 1).expect("Bad Date");
    let month_start_from_monday = month_start.weekday().number_from_monday();
    let month_end = match NaiveDate::from_ymd_opt(year, month + 1, 1) {
        None => NaiveDate::from_ymd_opt(year, 12, 31).unwrap(),
        Some(end) => end.sub(Duration::days(1)),
    };

    // start cell padding.
    for _ in 1..month_start_from_monday {
        cells.push(None);
    }
    // insert days...
    for day_number in 1..=month_end.day() {
        cells.push(NaiveDate::from_ymd_opt(year, month, day_number));
    }
    // end cell padding
    while cells.len() % 7 != 0 {
        cells.push(None);
    }

    cells
}

fn formatted_event_line(se: &ScheduledEvent) -> Html {
    let (text, icon) = match se.media_type {
        MediaType::tv => {
            // [ICON] 16:30 | The Office - The Dundies
            let t = format! {" {} | {} - {}",
            se.scheduled_date.format("%R"),
            se.episode.as_ref().unwrap().show_name,
            se.episode.as_ref().unwrap().name };
            let i = "gg-tv".to_string();

            (t, i)
        }
        MediaType::movie => {
            // [ICON] 16:30 | Ghostbusters
            let t = format! {" {} | {}",
            se.scheduled_date.format("%R"),
            se.movie.as_ref().unwrap().show_name };
            let i = "gg-film".to_string();

            (t, i)
        }
        MediaType::actor => (String::from("Unknown Actor"), String::from("gg-boy")),
        MediaType::person => (String::from("Unknown Person"), String::from("gg-boy")),
        MediaType::unknown => (String::from("Unknown"), String::from("gg-danger")),
    };

    html! {
        <a class="panel-block schedule-item">
            <span class="panel-icon">
                <i class={icon} aria-hidden="true"></i>
            </span>
            {text}
        </a>
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
            active_month: current_date,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ChangeDate(new_date) => {
                self.active_month = new_date;
                // Today should pull up the current day if you're in the same month already.
                if self.active_month.month() == new_date.month() {
                    self.active_day = new_date;
                }

                true
            }
            ChangeDay(day) => {
                self.active_day = day;

                true
            }
            EventCalendarMsg::ExportCsv => {
                let mut em = EventManager::create();
                if let Ok(csv) = em.events_as_csv(CsvType::GCAL) {
                    // Push our CSV to the client as it's own file.
                    export_file("bynger_event_export.csv", &csv, "text/csv")
                }

                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let em = EventManager::create();
        let day = self.active_day;
        let date = self.active_month;
        let dn = day.date_naive();
        let onexport = ctx.link().callback(|_| EventCalendarMsg::ExportCsv);
        let chevron_click = ctx.link().callback(move |me: MouseEvent| {
            let mut out_date = date;
            if let Some(elem_id) = UiHelpers::get_id_from_event_elem(Event::from(me)) {
                if let Some(direction) = elem_id.strip_prefix("cal_month_") {
                    let mut year = date.year();
                    let mut month = date.month();
                    match direction {
                        "next" => {
                            if month + 1 == 13 {
                                month = 1;
                                year += 1;
                            } else {
                                month += 1;
                            }
                            let next_month = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
                            out_date = DateTime::from_utc(
                                next_month.and_time(NaiveTime::from_hms_opt(0, 0, 1).unwrap()),
                                Utc,
                            );
                        }
                        "prev" => {
                            let prev_month = NaiveDate::from_ymd_opt(year, month, 1)
                                .unwrap()
                                .sub(Duration::days(1));
                            out_date = DateTime::from_utc(
                                prev_month.and_time(NaiveTime::from_hms_opt(0, 0, 1).unwrap()),
                                Utc,
                            );
                        }
                        "curr" => {
                            out_date = Utc::now();
                        }
                        _ => unreachable!(),
                    }
                }
            }
            ChangeDate(out_date)
        });

        // Must be mutable to sort after collection.
        let mut day_events: Vec<&ScheduledEvent> = em
            .events
            .iter()
            .filter(|se| se.scheduled_date.date_naive() == dn)
            .collect();
        // Sort our day's events by time...
        day_events.sort_by(|a, b| a.scheduled_date.cmp(&b.scheduled_date));

        let cells = get_calendar_cells(&date);
        let cell_id_format = "%Y_%m_%d";
        let day_click = ctx.link().callback(move |me: MouseEvent| {
            let mut out = day;
            if let Some(elem_id) = UiHelpers::get_id_from_event_elem(Event::from(me)) {
                let id_split: Vec<&str> = elem_id.split('_').collect();
                out = Utc
                    .with_ymd_and_hms(
                        id_split[0].parse::<i32>().unwrap(),
                        id_split[1].parse::<u32>().unwrap(),
                        id_split[2].parse::<u32>().unwrap(),
                        0,
                        0,
                        1,
                    )
                    .unwrap();
            }

            ChangeDay(out)
        });

        let day_val = |d: Option<NaiveDate>| {
            match d {
                None => html! {<td></td>},
                Some(d) => {
                    let day_id = d.format(cell_id_format).to_string();
                    let events: Vec<&ScheduledEvent> = em
                        .events
                        .iter()
                        .filter(|se| se.scheduled_date.date_naive() == d)
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
        let weeks = cells
            .chunks(7)
            .map(|week| {
                html! {
                    <tr>
                      {
                        week.iter().map(|&d| {
                          day_val(d)
                        }).collect::<Html>()
                      }
                    </tr>
                }
            })
            .collect::<Html>();

        html! {
            <div class="is-centered box calendar-container">
                <div class="columns">
                    <div class="column calendar-left">
                        // This should probably be broken in to it's own component
                        // <CalendarCard date={date} event-manager={&em} />
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
                                            formatted_event_line(ev)
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
                            <p class="level-item" onclick={&onexport}>
                                <a class="button" id="cal_export_events">{"export"}</a>
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
