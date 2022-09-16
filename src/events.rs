use chrono::{DateTime, Utc};
use crate::event_calendar::CalendarSchedulableEvent;
//use serde::ser::{Serialize, SerializeStruct, Serializer};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{Error, Visitor};
use serde::ser::SerializeStruct;
use crate::schedule_show::Episode;
use crate::search_client::MediaType;

#[derive(Clone, Serialize, Deserialize)]
pub struct ScheduledEvent {
    // Store it all as Utc. Let the UI apply Timezone Offset
    pub scheduled_date: DateTime<Utc>,
    pub media_type: MediaType,
    // Scheduled Events must implement the CalendarSchedulableEvent trait use by the Event Calendar.
    pub episode: Option<Episode>,
    pub movie: Option<String>,
}

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

