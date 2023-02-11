use std::error::Error;

use chrono::Duration;
use gloo::storage::errors::StorageError;
use gloo::storage::{LocalStorage, Storage};
use std::ops::Add;

use crate::events::ScheduledEvent;
use crate::search_client::MediaType;
use crate::ByngerStore;

pub enum CsvType {
    GCAL, // Google Cal
    // --- GCAL Format ---
    // Subject: String
    // Start Date: D/MM/YYYY (2/20/2023)
    // Start Time: H:MM AMPM (7:10 PM)
    // End Date: D/MM/YYYY (5/15/2023)
    // End Time: H:MM AMPM (8:30 PM)
    // All Day Event: BOOL (FALSE)
    // Description: String
    // Location: String
    // Private: BOOL (TRUE)
    ICAL, // Not really a CSV format, actually.
}

pub struct EventManager {
    storage: String,
    pub events: Vec<ScheduledEvent>,
}

impl EventManager {
    pub(crate) fn create() -> Self {
        let storage = format!("{}", ByngerStore::ScheduledEvents);
        let events = LocalStorage::get(&*storage).unwrap_or_default();

        EventManager { storage, events }
    }

    // fn add_event(&mut self, scheduled_event: ScheduledEvent) {
    //     // console_log!(format!("{}", &scheduled_event.event.id()));
    //     self.events.push(scheduled_event)
    // }

    fn store(&self) -> Result<(), StorageError> {
        LocalStorage::set(&self.storage, self.events.to_vec())
    }

    fn purge_events(&mut self) -> Result<(), StorageError> {
        self.events.clear();

        self.store()
    }

    pub fn add_events(&mut self, mut events: Vec<ScheduledEvent>) -> Result<(), StorageError> {
        self.events.append(&mut events);
        self.events
            .sort_unstable_by(|a, b| a.scheduled_date.cmp(&b.scheduled_date));
        self.store() // commit new schedule to LocalStorage
    }

    pub fn events_as_csv(&mut self, csv_type: CsvType) -> Result<String, Box<dyn Error>> {
        // A naive CSV export implementation.
        let mut csv_string = String::new();
        match csv_type {
            CsvType::GCAL => {
                let date_fmt = "%D"; // Month-day-year format. Same as %m/%d/%y
                let time_fmt = "%I:%M %p";
                let tv_subject = |event: &ScheduledEvent| {
                    let ep = event.episode.clone().expect("Missing Episode");
                    format!(
                        "{} | s{}e{}",
                        &ep.show_name, &ep.season_number, &ep.episode_number
                    )
                };
                let mv_subject = |event: &ScheduledEvent| {
                    let mv = event.movie.clone().expect("Missing Movie");
                    format!("{} | Runtime: {}", mv.show_name, mv.runtime)
                };

                // Could we just use serde?
                // Write the header.
                csv_string.push_str(
                        "Subject, Start Date, Start Time, End Date, End Time, All Day Event, Description, Location, Private \n",
                    );

                self.events
                    .sort_unstable_by(|a, b| a.scheduled_date.cmp(&b.scheduled_date));
                for event in &self.events {
                    match event.media_type {
                        MediaType::tv => {
                            let ep = &event.episode.clone().unwrap();

                            csv_string.push_str(&format!(
                                "{},{},{},{},{},{},{},{},{}\n",
                                tv_subject(event),
                                event.scheduled_date.format(date_fmt),
                                event.scheduled_date.format(time_fmt),
                                &event.scheduled_date.format(date_fmt), // Eps end same day.
                                event
                                    .scheduled_date
                                    .add(Duration::minutes(ep.episode_run_time as i64))
                                    .format(time_fmt),
                                "FALSE",
                                ep.name,
                                "",
                                "TRUE"
                            ));
                        }
                        MediaType::movie => {
                            let mv = &event.movie.clone().unwrap();

                            csv_string.push_str(&format!(
                                "{},{},{},{},{},{},{},{},{}\n",
                                mv_subject(event),
                                event.scheduled_date.format(date_fmt),
                                event.scheduled_date.format(time_fmt),
                                &event.scheduled_date.format(date_fmt), // Eps end same day.
                                event
                                    .scheduled_date
                                    .add(Duration::minutes(mv.runtime as i64))
                                    .format(time_fmt),
                                "FALSE",
                                mv.show_name,
                                "",
                                "TRUE"
                            ));
                        }
                        _ => {}
                    }
                }
            }
            CsvType::ICAL => {
                todo!();
            }
        };

        Ok(csv_string)
    }
}
