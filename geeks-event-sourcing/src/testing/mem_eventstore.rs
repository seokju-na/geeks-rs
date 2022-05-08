use std::collections::HashMap;
use std::{
  convert::Infallible,
  sync::{Arc, RwLock},
};

use async_trait::async_trait;

use crate::{Event, Eventstore, PersistedEvent, VersionSelect};

#[derive(Debug)]
struct InMemoryBackend<T>
where
  T: Event,
{
  events: HashMap<String, Vec<PersistedEvent<T>>>,
}

impl<T> Default for InMemoryBackend<T>
where
  T: Event,
{
  fn default() -> Self {
    Self {
      events: HashMap::default(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct InMemoryEventstore<T>
where
  T: Event,
{
  backend: Arc<RwLock<InMemoryBackend<T>>>,
}

impl<T> Default for InMemoryEventstore<T>
where
  T: Event,
{
  fn default() -> Self {
    Self {
      backend: Arc::default(),
    }
  }
}

#[async_trait]
impl<T> Eventstore for InMemoryEventstore<T>
where
  T: Event + Clone,
{
  type Event = T;
  type Error = Infallible;

  async fn read(
    &self,
    aggregate_id: String,
    select: VersionSelect,
  ) -> Result<Vec<PersistedEvent<Self::Event>>, Self::Error> {
    let backend = self.backend.read().expect("locked");
    let events: Vec<_> = backend
      .events
      .get(&aggregate_id)
      .cloned()
      .unwrap_or_default()
      .into_iter()
      .filter(|event| match select {
        VersionSelect::All => true,
        VersionSelect::From(v) => event.version >= v,
      })
      .collect();

    Ok(events)
  }

  async fn append(&self, events: Vec<PersistedEvent<Self::Event>>) -> Result<(), Self::Error> {
    let mut backend = self
      .backend
      .write()
      .expect("acquire write lock on event store backend");

    events.into_iter().for_each(|event| {
      backend
        .events
        .entry(event.aggregate_id.to_owned())
        .and_modify(|x| x.push(event.clone()))
        .or_insert_with(|| vec![event]);
    });

    Ok(())
  }
}
