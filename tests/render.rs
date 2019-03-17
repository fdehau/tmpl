use std::fs::File;
use std::io;
use std::io::Write;

use assert_cli::Assert;
use tempdir::TempDir;

struct Context {
    dir: TempDir,
}

impl Context {
    fn new() -> io::Result<Context> {
        let dir = TempDir::new("tmpl_tests")?;
        let ctx = Context { dir };
        Ok(ctx)
    }

    fn create_file(&self, name: &str, content: &str) -> io::Result<String> {
        let path = self.dir.path().join(name);
        let res = path.clone().as_path().to_str().unwrap().to_owned();
        let mut file = File::create(path)?;
        file.write(content.as_bytes())?;
        Ok(res)
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
    let context = Context::new().unwrap();
    let vars1 = context.create_file("vars_1.json", VARIABLES_1).unwrap();
    let vars2 = context.create_file("vars_2.json", VARIABLES_2).unwrap();
    let template = context.create_file("template", TEMPLATE).unwrap();

    let args = vec![vars1.as_str(), vars2.as_str(), template.as_str()];

    Assert::main_binary()
        .with_args(&args)
        .stdout()
        .is("2|florian")
        .unwrap();

    context.close().unwrap();
}
