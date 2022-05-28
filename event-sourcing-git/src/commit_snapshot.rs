use std::path::Path;

use geeks_git::{commit, get_head_commit, get_status, CommitReader, GitResult, StatusType};
use git2::{IndexAddOption, Oid, Repository};

pub const SNAPSHOT_MSG: &str = "[snapshot]";

pub fn commit_snapshot<P>(repo_path: P) -> GitResult<Option<Oid>>
where
  P: AsRef<Path>,
{
  let repo = Repository::open(&repo_path)?;
  let is_head_snapshot = get_head_commit(&repo)?
    .message
    .subject
    .contains(SNAPSHOT_MSG);
  let is_working_dir_clean = get_status(&repo_path, StatusType::WorkingDir)?.is_empty();

  if is_head_snapshot || is_working_dir_clean {
    return Ok(None);
  }

  let mut index = repo.index()?;
  index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
  index.write()?;
  let oid = commit(&repo_path, SNAPSHOT_MSG)?;

  Ok(Some(oid))
}

#[cfg(test)]
mod tests {
  use geeks_git::{get_status, CommitInfo, StatusType};
  use git2::Repository;

  use geeks_git_testing::FixtureRepository;

  use super::*;

  #[test]
  fn should_write_all_files_with_snapshot_commit() {
    let fixture = FixtureRepository::setup_with_script(
      r#"
    git commit --allow-empty -m "initial"
    echo "A" > a.txt
    echo "B" > b.txt
    mkdir foo/
    echo "foo/bar" > foo/bar.txt
    "#,
    );
    let oid = commit_snapshot(&fixture.path).unwrap().unwrap();
    let repo = Repository::open(&fixture.path).unwrap();
    let commit = repo.find_commit(oid).unwrap();
    assert_eq!(CommitInfo::from(commit).message, SNAPSHOT_MSG.into());

    let status = get_status(&fixture.path, StatusType::Both).unwrap();
    assert!(status.is_empty());
  }

  #[test]
  fn should_not_create_snapshot_commit_when_workdir_clean() {
    let fixture = FixtureRepository::setup_with_script(
      r#"
    git commit --allow-empty -m "initial"
    echo "A" > a.txt
    git add a.txt
    git commit -m "secondary"
    "#,
    );
    let result = commit_snapshot(&fixture.path).unwrap();
    assert!(result.is_none());
  }

  #[test]
  fn should_not_create_snapshot_commit_when_head_is_snapshot_commit() {
    let fixture = FixtureRepository::setup_with_script(
      r#"
    git commit --allow-empty -m "initial"
    echo "A" > a.txt
    "#,
    );
    let result = commit_snapshot(&fixture.path).unwrap();
    assert!(result.is_some());
    let result = commit_snapshot(&fixture.path).unwrap();
    assert!(result.is_none());
  }
}
