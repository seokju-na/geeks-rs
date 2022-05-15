#![allow(dead_code)]

use std::path::PathBuf;

use nanoid::nanoid;
use run_script::{run, ScriptOptions};

pub struct FixtureRepository {
  pub path: PathBuf,
}

impl FixtureRepository {
  pub fn setup() -> Self {
    Self::setup_with_script("")
  }

  pub fn setup_with_script(setup_script: &str) -> Self {
    let path: PathBuf = ["test-fixtures", &nanoid!()].iter().collect();
    let path_as_str = path.to_str().unwrap();

    let init_script = format!(
      r#"
            mkdir -p {}
            cd {}
            git init
            {}
            "#,
      path_as_str, path_as_str, setup_script
    );
    let (exit_code, output, _) = run(&init_script, &vec![], &ScriptOptions::new()).unwrap();
    println!("script output: {}", output);
    if exit_code != 0 {
      panic!("exit with {}", exit_code);
    }

    Self { path }
  }
}

impl Drop for FixtureRepository {
  fn drop(&mut self) {
    let rm_script = format!("rm -rf {}", self.path.to_str().unwrap());
    run(&rm_script, &vec![], &ScriptOptions::new()).unwrap();
  }
}
