use chrono::{DateTime, Utc};

//use serde::ser::{Serialize, SerializeStruct, Serializer};
use crate::schedule_show::{Episode, Movie};
use crate::search_client::MediaType;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScheduledEvent {
    pub uuid: Uuid,  // Lazy PK, but it works.
    // Store it all as Utc. Let the UI apply Timezone Offset
    pub scheduled_date: DateTime<Utc>,
    pub media_type: MediaType,
    // Scheduled Events must implement the CalendarSchedulableEvent trait use by the Event Calendar.
    pub episode: Option<Episode>,
    pub movie: Option<Movie>,
}

// pub trait ShowEvent {
//     fn show_name(&self) -> Option<String>;
//     fn show_length(&self) -> Option<String>;
//     fn show_description(&self) -> Option<String>;
// }
//
//
// impl ShowEvent for ScheduledEvent {
//     fn show_name(&self) -> &str {
//         match &self.media_type {
//             MediaType::tv => {
//                 match &self.episode {
//                     None => {"Unknown"}
//                     Some(ep) => { &ep.show_name }
//                 }
//             }
//             MediaType::movie => {
//                 match &self.movie {
//                     None => { "Unknown" }
//                     Some(mo) => { &mo.show_name }
//                 }
//             }
//             _ => {"Unknown"}
//         }
//     }
//
//     fn show_length(&self) -> &str {
//         match &self.media_type {
//             MediaType::tv => {
//                 match &self.episode {
//                     None => { "Unknown" }
//                     Some(ep) => { "Ep Name" }
//                 }
//             }
//             MediaType::movie => {
//                 match &self.movie {
//                     None => { "Unknown" }
//                     Some(mo) => { "Movie doesnt have runtime??" }
//                 }
//             }
//             _ => { "Unknown" }
//         }
//     }
//
//     fn show_description(&self) -> &str {
//         match self.media_type {
//             MediaType::tv => {
//                 match &self.episode {
//                     None => { "Unknown" }
//                     Some(ep) => { format!("{}", ep.show_id.clone()).as_str() }
//                 }
//             }
//             MediaType::movie => {
//                 match &self.movie {
//                     None => { "Unknown" }
//                     Some(mo) => { format!("{}", mo.movie_id).as_str() }
//                 }
//             }
//             _ => { "Unknown" }
//         }
//     }
// }

// impl Serialize for ScheduledEvent<'_> {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
//         let mut state = serializer.serialize_struct("ScheduledEvent", 6)?;
//         state.serialize_field("scheduled_date", &self.scheduled_date.to_string())?;
//         state.serialize_field("event.id", &self.event.id())?;
//         state.serialize_field("event.name", &self.event.name())?;
//         state.serialize_field("event.media_type", &self.event.media_type())?;
//         state.serialize_field("event.description", &self.event.description())?;
//         state.serialize_field("event.duration", &self.event.duration())?;
//
//         state.end()
//     }
// }
