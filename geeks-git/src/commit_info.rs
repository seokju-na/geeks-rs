use git2::{Commit, Oid};

#[derive(Debug, PartialEq, Eq)]
pub struct CommitInfo {
  pub message: String,
  pub time: i64,
  pub author_name: String,
  pub author_email: String,
  pub id: Oid,
}

impl<'a> From<Commit<'a>> for CommitInfo {
  fn from(commit: Commit<'a>) -> Self {
    let message = commit.message().unwrap_or("").to_string();
    let author = commit.author();

    Self {
      message,
      time: commit.time().seconds(),
      author_name: author.name().unwrap_or("unknown").to_string(),
      author_email: author.email().unwrap_or("unknown").to_string(),
      id: commit.id(),
    }
  }
}
