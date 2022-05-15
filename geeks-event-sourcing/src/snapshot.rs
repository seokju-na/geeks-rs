use crate::{Aggregate, AggregateRoot};
use async_trait::async_trait;

#[async_trait]
pub trait Snapshot<T>
where
  T: Aggregate,
{
  type Error: Send + Sync;

  async fn load(&self) -> Result<AggregateRoot<T>, Self::Error>;

  async fn save(&self, root: AggregateRoot<T>) -> Result<(), Self::Error>;
}

#[cfg(test)]
mod tests {
  use crate::testing::{Todo, TodoSnapshot, TodoStatus};
  use crate::{AggregateRoot, Snapshot};
  use chrono::Utc;
  use geeks_git_testing::FixtureRepository;
  use std::collections::HashMap;

  #[tokio::test]
  async fn should_load_snapshot_from_fs() {
    let todo = Todo {
      id: "todo1".to_string(),
      title: "Eat pizza".to_string(),
      status: TodoStatus::Done,
      created_at: Utc::now().timestamp(),
      updated_at: Utc::now().timestamp(),
    };
    let root = AggregateRoot::<Todo>::new(
      HashMap::from([("todo1".to_string(), todo)]),
      HashMap::from([("todo1".to_string(), 1)]),
    );
    let fixture = FixtureRepository::setup();
    let snapshot = TodoSnapshot::new(&fixture.path);
    snapshot.save(root).await.unwrap();
    let aggregate = snapshot.load().await.unwrap();

    assert!(aggregate.get_state("todo1").is_some());
  }
}
