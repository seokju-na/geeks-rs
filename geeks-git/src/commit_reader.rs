use git2::{Error, Oid, Repository, Revwalk};

use crate::commit_info::CommitInfo;
use crate::GitResult;

pub enum CommitReadStartOn {
  Head,
  Oid(Oid),
}

pub type CommitReaderEndWhen = fn(&CommitInfo) -> bool;

fn keep_reading(_: &CommitInfo) -> bool {
  false
}

pub struct CommitReader<'a> {
  repo: &'a Repository,
  revwalk: Revwalk<'a>,
  start_on: CommitReadStartOn,
  end_when: Option<CommitReaderEndWhen>,
  started: bool,
}

impl<'a> CommitReader<'a> {
  pub fn new(repo: &'a Repository) -> GitResult<Self> {
    let revwalk = repo.revwalk()?;

    Ok(Self {
      repo,
      revwalk,
      start_on: CommitReadStartOn::Head,
      end_when: None,
      started: false,
    })
  }

  #[must_use]
  pub fn start_on_head(self) -> Self {
    self.start_on(CommitReadStartOn::Head)
  }

  #[must_use]
  pub fn start_on_oid(self, oid: Oid) -> Self {
    self.start_on(CommitReadStartOn::Oid(oid))
  }

  #[must_use]
  pub fn start_on(self, start: CommitReadStartOn) -> Self {
    Self {
      start_on: start,
      ..self
    }
  }

  #[must_use]
  pub fn end_when(self, end_when: CommitReaderEndWhen) -> Self {
    Self {
      end_when: Some(end_when),
      ..self
    }
  }

  fn push_start(&mut self) -> Result<(), Error> {
    if self.started {
      return Ok(());
    }

    match self.start_on {
      CommitReadStartOn::Head => {
        self.revwalk.push_head()?;
      }
      CommitReadStartOn::Oid(oid) => {
        self.revwalk.push(oid)?;
      }
    }
    self.started = true;
    Ok(())
  }
}

impl<'a> Iterator for CommitReader<'a> {
  type Item = Result<CommitInfo, Error>;

  fn next(&mut self) -> Option<Self::Item> {
    // returns err when push start fails.
    if let Err(e) = self.push_start() {
      return Some(Err(e));
    }

    let item = self.revwalk.next().map(|x| match x {
      Ok(oid) => self.repo.find_commit(oid).map(CommitInfo::from),
      Err(e) => Err(e),
    });

    if let Some(Ok(commit)) = &item {
      let end_when = self.end_when.unwrap_or(keep_reading);
      if end_when(commit) {
        return None;
      }
    }

    item
  }
}

#[cfg(test)]
mod tests {
  use git2::Repository;

  use crate::testing::git::FixtureRepository;

  use super::*;

  #[test]
  fn should_read_commits_from_head() {
    let fixture = FixtureRepository::setup_with_script(
      r#"
      git config --local user.email "test@test.com"
      git config --local user.name "Test"

      git commit --allow-empty -m "1"
      git commit --allow-empty -m "2"
      git commit --allow-empty -m "3"
      "#,
    );
    let repo = Repository::open(&fixture.path).unwrap();
    let reader = CommitReader::new(&repo).unwrap().start_on_head();
    let commits: Vec<_> = reader.map(|x| x.unwrap()).collect();

    assert_eq!(commits.len(), 3);
    assert!(commits[0].message.contains('3'));
    assert!(commits[1].message.contains('2'));
    assert!(commits[2].message.contains('1'));
  }

  #[test]
  fn should_read_commits_from_oid() {
    let fixture = FixtureRepository::setup_with_script(
      r#"
      git config --local user.email "test@test.com"
      git config --local user.name "Test"

      git commit --allow-empty -m "1"
      git commit --allow-empty -m "2"
      git commit --allow-empty -m "3"
      "#,
    );
    let repo = Repository::open(&fixture.path).unwrap();
    let reader = CommitReader::new(&repo).unwrap().start_on_head();
    let commits: Vec<_> = reader.map(|x| x.unwrap()).collect();

    let reader = CommitReader::new(&repo)
      .unwrap()
      .start_on_oid(commits[1].id);
    let commits: Vec<_> = reader.map(|x| x.unwrap()).collect();

    assert_eq!(commits.len(), 2);
    assert!(commits[0].message.contains('2'));
    assert!(commits[1].message.contains('1'));
  }

  #[test]
  fn should_read_commits_by_limited() {
    let fixture = FixtureRepository::setup_with_script(
      r#"
      git config --local user.email "test@test.com"
      git config --local user.name "Test"

      git commit --allow-empty -m "1"
      git commit --allow-empty -m "2"
      git commit --allow-empty -m "3"
      "#,
    );
    let repo = Repository::open(&fixture.path).unwrap();
    let reader = CommitReader::new(&repo).unwrap().start_on_head();
    let commits: Vec<_> = reader.take(2).map(|x| x.unwrap()).collect();

    assert_eq!(commits.len(), 2);
    assert!(commits[0].message.contains('3'));
    assert!(commits[1].message.contains('2'));
  }

  #[test]
  fn should_read_commits_until_end() {
    let fixture = FixtureRepository::setup_with_script(
      r#"
      git config --local user.email "test@test.com"
      git config --local user.name "Test"

      git commit --allow-empty -m "1"
      git commit --allow-empty -m "2"
      git commit --allow-empty -m "3"
      git commit --allow-empty -m "4"
      git commit --allow-empty -m "5"
      "#,
    );
    let repo = Repository::open(&fixture.path).unwrap();
    let reader = CommitReader::new(&repo)
      .unwrap()
      .start_on_head()
      .end_when(|commit: &CommitInfo| commit.message.contains('3'));
    let commits: Vec<_> = reader.map(|x| x.unwrap()).collect();

    assert_eq!(commits.len(), 2);
    assert!(commits[0].message.contains('5'));
    assert!(commits[1].message.contains('4'));

    let reader = CommitReader::new(&repo)
      .unwrap()
      .start_on_head()
      .end_when(|commit: &CommitInfo| commit.message.contains('5'));
    let commits: Vec<_> = reader.map(|x| x.unwrap()).collect();

    assert_eq!(commits.len(), 0);
  }
}
