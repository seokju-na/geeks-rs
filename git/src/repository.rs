use git2::{Error, ErrorCode, Oid, Repository, Signature};

use crate::{CommitInfo, GitError, GitResult};

pub fn get_head(repo: &Repository) -> GitResult<Oid> {
  let head = repo.head()?.target();
  match head {
    Some(x) => Ok(x),
    None => Err(GitError::NoHead),
  }
}

pub fn get_head_commit(repo: &Repository) -> GitResult<CommitInfo> {
  let head = get_head(repo)?;
  let commit = repo.find_commit(head).map(CommitInfo::from)?;

  Ok(commit)
}

pub(crate) fn get_signature(repo: &Repository) -> Result<Signature<'_>, Error> {
  let sig = repo.signature();

  if let Err(e) = &sig {
    if e.code() == ErrorCode::NotFound {
      let config = repo.config()?;

      if let (Err(_), Ok(email_entry)) = (
        config.get_entry("user.name"),
        config.get_entry("user.email"),
      ) {
        if let Some(email) = email_entry.value() {
          return Signature::now("unknown", email);
        }
      };
    }
  }

  sig
}

#[cfg(test)]
mod tests {
  use geeks_git_testing::FixtureRepository;
  use git2::Repository;

  use super::*;

  #[test]
  fn should_get_head_commit() {
    let fixture = FixtureRepository::setup_with_script(
      r#"
    git commit --allow-empty -m "initial"
    "#,
    );
    let repo = Repository::open(&fixture.path).unwrap();
    let head_commit = get_head_commit(&repo).unwrap();

    assert_eq!(head_commit.message, "initial".into());
  }
}
