extern crate core;

pub use crate::aggregate::{Aggregate, AggregateRoot};
pub use crate::command::Command;
pub use crate::event::{Event, PersistedEvent};
pub use crate::eventstore::*;
pub use crate::snapshot::*;

mod aggregate;
mod command;
mod event;
mod eventstore;
mod snapshot;
pub mod testing;

pub type Version = u64;
pub type Timestamp = i64;

#[derive(thiserror::Error, Debug)]
pub enum Error<E, EE, SE> {
  #[error("aggregate error: {0}")]
  AggregateError(#[source] E),

  #[error("eventstore error: {0}")]
  EventstoreError(#[source] EE),

  #[error("snapshot error: {0}")]
  SnapshotError(#[source] SE),
}

pub async fn get_unsaved_events<T, E>(
  root: &AggregateRoot<T>,
  eventstore: &E,
) -> Result<Vec<PersistedEvent<T::Event>>, E::Error>
where
  T: Aggregate,
  E: Eventstore<Event = T::Event>,
{
  let versions = root.versions.clone();
  let mut unsaved_events = Vec::new();
  let read_events = versions.iter().map(|(id, version)| async {
    eventstore
      .read(id.to_owned(), VersionSelect::From(*version + 1))
      .await
  });

  for events in read_events {
    let events = events.await?;
    unsaved_events.append(&mut events.clone());
  }

  Ok(unsaved_events)
}

pub async fn load_aggregate<T, E, S>(
  eventstore: &E,
  snapshot: &S,
) -> Result<AggregateRoot<T>, Error<T::Error, E::Error, S::Error>>
where
  T: Aggregate,
  E: Eventstore<Event = T::Event>,
  S: Snapshot<T>,
{
  let mut root = snapshot.load().await.map_err(Error::SnapshotError)?;
  let unsaved_events = get_unsaved_events(&root, eventstore)
    .await
    .map_err(Error::EventstoreError)?;

  root
    .save_events(unsaved_events)
    .map_err(Error::AggregateError)?;

  snapshot
    .save(root.clone())
    .await
    .map_err(Error::SnapshotError)?;

  Ok(root)
}
