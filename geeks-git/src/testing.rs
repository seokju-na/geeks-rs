#[cfg(test)]
pub(crate) mod git {
  use std::path::PathBuf;

  use nanoid::nanoid;
  use run_script::{run, ScriptOptions};

  pub(crate) struct FixtureRepository {
    pub path: String,
  }

  impl FixtureRepository {
    pub fn setup() -> Self {
      Self::setup_with_script("")
    }

    pub fn setup_with_script(setup_script: &str) -> Self {
      let path: PathBuf = ["test-fixtures", &nanoid!()].iter().collect();
      let path = path.to_str().unwrap();

      let init_script = format!(
        r#"
            mkdir -p {}
            cd {}
            git init -b main
            git show HEAD
            {}
            "#,
        path, path, setup_script
      );
      let result = run(&init_script, &vec![], &ScriptOptions::new()).unwrap();
      println!("script result: {}", result.1);

      Self {
        path: path.to_owned(),
      }
    }
  }

  impl Drop for FixtureRepository {
    fn drop(&mut self) {
      let rm_script = format!("rm -rf {}", self.path);
      run(&rm_script, &vec![], &ScriptOptions::new()).unwrap();
    }
  }
}
