use chrono::{DateTime, Utc};
use crate::event_calendar::CalendarSchedulableEvent;

pub struct ScheduledEvent {
    // Store it all as Utc. Let the UI apply Timezone Offset
    scheduled_date: DateTime<Utc>,
    // Scheduled Events must implement the CalendarSchedulableEvent trait use by the Event Calendar.
    event: dyn CalendarSchedulableEvent
}