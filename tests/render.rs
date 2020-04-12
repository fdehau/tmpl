use anyhow::Context;
use assert_cmd::{assert::OutputAssertExt, Command};
use std::{
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
};
use tempdir::TempDir;

struct TestContext {
    dir: TempDir,
}

impl TestContext {
    fn new() -> io::Result<TestContext> {
        let dir = TempDir::new("tmpl_tests")?;
        let ctx = TestContext { dir };
        Ok(ctx)
    }

    fn create_file<P>(&self, name: P, content: &str) -> anyhow::Result<PathBuf>
    where
        P: AsRef<Path>,
    {
        let path = self.dir.path().join(name.as_ref());
        let mut file = File::create(&path)
            .with_context(|| format!("Failed to create file {}", path.display()))?;
        file.write_all(content.as_bytes())?;
        Ok(path)
    }

    fn close(self) -> io::Result<()> {
        self.dir.close()
    }
}

const TEMPLATE: &'static str = "{{ user.id }}|{{ user.name }}";

const VARIABLES_1: &'static str = r#"
{
    "user": {
        "id": 1,
        "name": "florian"
    }
}
"#;

const VARIABLES_2: &'static str = r#"
{
    "user": {
        "id": 2,
        "name": "florian"
    }
}
"#;

#[test]
fn test_success() {
    let context = TestContext::new().unwrap();

    let vars1 = context.create_file("vars_1.json", VARIABLES_1).unwrap();
    let vars2 = context.create_file("vars_2.json", VARIABLES_2).unwrap();

    let cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .write_stdin(TEMPLATE)
        .arg(vars1)
        .arg(vars2)
        .unwrap();
    cmd.assert().success().stdout("2|florian");

    context.close().unwrap();
}
