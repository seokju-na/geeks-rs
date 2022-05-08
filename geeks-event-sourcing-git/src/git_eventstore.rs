use std::marker::PhantomData;
use std::path::PathBuf;

use async_trait::async_trait;
use git2::Repository;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{from_str, to_string};

use geeks_event_sourcing::{Event, Eventstore, PersistedEvent, VersionSelect};
use geeks_git::{commit, CommitInfo, CommitMessage, CommitReader, GitError};

pub struct GitEventstore<T>
where
  T: Event,
{
  pub name: String,
  repo_path: PathBuf,
  _event: PhantomData<T>,
}

impl<T> GitEventstore<T>
where
  T: Event + Clone + Serialize + DeserializeOwned,
{
  pub fn new(name: &str, repo_path: PathBuf) -> Self {
    Self {
      name: String::from(name),
      repo_path,
      _event: PhantomData::default(),
    }
  }

  fn event_to_commit_message(persisted: PersistedEvent<T>) -> CommitMessage {
    CommitMessage {
      subject: format!("[event] {}", persisted.event.name()),
      body: to_string(&persisted).unwrap(),
    }
  }

  fn commit_to_event(commit: CommitInfo) -> PersistedEvent<T> {
    from_str(&commit.message.body).unwrap()
  }
}

#[async_trait]
impl<T> Eventstore for GitEventstore<T>
where
  T: Event + Clone + Serialize + DeserializeOwned,
{
  type Event = T;
  type Error = GitError;

  async fn read(
    &self,
    aggregate_id: String,
    select: VersionSelect,
  ) -> Result<Vec<PersistedEvent<Self::Event>>, Self::Error> {
    let repo = Repository::open(&self.repo_path)?;
    let reader = CommitReader::new(&repo)?.start_on_head();
    let events: Vec<_> = reader
      .flat_map(|x| x.map(GitEventstore::commit_to_event))
      .filter(|event| event.aggregate_id == aggregate_id)
      .filter(|event| match select {
        VersionSelect::All => true,
        VersionSelect::From(v) => event.version >= v,
      })
      .collect();

    Ok(events)
  }

  async fn append(&self, events: Vec<PersistedEvent<Self::Event>>) -> Result<(), Self::Error> {
    let commit_messages = events
      .into_iter()
      .map(GitEventstore::event_to_commit_message);

    for message in commit_messages {
      commit(&self.repo_path, message)?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use crate::git_eventstore::GitEventstore;
  use geeks_event_sourcing::testing::{TodoEvent, TodoStatus};
  use geeks_event_sourcing::{Event, Eventstore, PersistedEvent, VersionSelect};
  use geeks_git_testing::FixtureRepository;

  #[tokio::test]
  async fn should_read_events() {
    let event1 = TodoEvent::TodoCreated {
      id: "todo1".to_string(),
      title: "Drink coffee".to_string(),
      status: TodoStatus::InProgress,
    };
    let event2 = TodoEvent::TodoTitleUpdated {
      title: "Eat pizza".to_string(),
    };

    let fixture = FixtureRepository::setup_with_script(
      r#"
    git config --local user.email "test@test.com"
    "#,
    );
    let eventstore = GitEventstore::new("test", fixture.path.clone());
    eventstore
      .append(vec![
        PersistedEvent {
          aggregate_id: "todo1".to_string(),
          version: 1,
          event: event1,
        },
        PersistedEvent {
          aggregate_id: "todo1".to_string(),
          version: 2,
          event: event2,
        },
      ])
      .await
      .unwrap();

    let events = eventstore
      .read("todo1".to_string(), VersionSelect::All)
      .await
      .unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].event.name(), "TodoTitleUpdated");
    assert_eq!(events[1].event.name(), "TodoCreated");
  }
}
