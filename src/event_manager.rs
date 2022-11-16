use gloo::storage::errors::StorageError;
use gloo::storage::{LocalStorage, Storage};

use crate::events::ScheduledEvent;
use crate::ByngerStore;

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

    fn add_event(&mut self, scheduled_event: ScheduledEvent) {
        // console_log!(format!("{}", &scheduled_event.event.id()));
        self.events.push(scheduled_event)
    }

    fn store(&self) -> Result<(), StorageError> {
        LocalStorage::set(&self.storage, self.events.to_vec())
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
