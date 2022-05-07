use std::marker::PhantomData;
use std::path::PathBuf;

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;

use geeks_event_sourcing::{Event, Eventstore, PersistedEvent, VersionSelect};
use geeks_git::GitError;

pub struct GitEventstore<T>
where
  T: Event,
{
  repo_path: PathBuf,
  _event: PhantomData<T>,
}

impl<T> GitEventstore<T>
where
  T: Event,
{
  pub fn new(repo_path: PathBuf) -> Self {
    Self {
      repo_path,
      _event: PhantomData::default(),
    }
  }
}

#[async_trait]
impl<T> Eventstore for GitEventstore<T>
where
  T: Event + Clone + Serialize + DeserializeOwned,
{
  type Event = T;
  type Error = GitError;

  fn read(
    &self,
    id: &str,
    select: VersionSelect,
  ) -> Result<Vec<PersistedEvent<Self::Event>>, Self::Error> {
    todo!()
  }

  async fn append(
    &self,
    id: String,
    events: Vec<PersistedEvent<Self::Event>>,
  ) -> Result<(), Self::Error> {
    todo!()
  }
}
