use std::fmt::Error;
use std::ops::Deref;
use futures::TryFutureExt;
use gloo::storage::{LocalStorage, Storage};
use gloo::storage::errors::StorageError;
use js_sys::Atomics::store;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use web_sys::RequestCache::Default;

use weblog::console_log;
use crate::ByngerStore;
use crate::ByngerStore::ScheduledEvents;
use crate::event_calendar::CalendarSchedulableEvent;
use crate::events::ScheduledEvent;

pub struct EventManager {
    storage: Box<String>,
    pub events: Box<Vec<ScheduledEvent>>
}

impl EventManager {
    pub(crate) fn create() -> Self {
        let storage = Box::new(format!("{}", ByngerStore::ScheduledEvents));
        let events = Box::new(
            LocalStorage::get(&*storage).unwrap_or(Vec::<ScheduledEvent>::new())
        );

        EventManager {
            storage,
            events
        }
    }

    fn add_event(&mut self, scheduled_event: ScheduledEvent) {
        // console_log!(format!("{}", &scheduled_event.event.id()));
        self.events.push(scheduled_event)
    }

    fn store(&self) -> Result<(), StorageError> {
        LocalStorage::set(self.storage.as_ref(), self.events.to_vec().clone())
    }

    fn purge_events(&mut self) -> Result<(), StorageError> {
        self.events.clear();

        self.store()
    }

    pub fn add_events(&mut self, mut events: Vec<ScheduledEvent>) -> Result<(), StorageError> {
        self.events.append(&mut events);

        self.store() // commit new schedule to LocalStorage
    }

}

