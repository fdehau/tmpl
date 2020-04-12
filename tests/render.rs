use assert_cmd::Command;
use assert_fs::{
    assert::PathAssert,
    fixture::{FileTouch, FileWriteStr, PathChild},
    TempDir,
};

#[test]
fn test_merge() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;

    let vars1 = tmp.child("vars1.json");
    vars1.write_str(
        r#"
        {
            "user": {
                "id":"1",
                "name": "florian",
                "path": "/users/1"
            }
        }
    "#,
    )?;
    let vars2 = tmp.child("vars2.json");
    vars2.write_str(
        r#"
        {
            "user": {
                "id" : "2"
            }
        }
    "#,
    )?;

    Command::cargo_bin(env!("CARGO_PKG_NAME"))?
        .write_stdin("{{ user.id }},{{ user.name }},{{ user.path }}")
        .arg(vars1.path())
        .arg(vars2.path())
        .assert()
        .success()
        .stdout("2,florian,/users/1");

    Ok(())
}

#[test]
fn test_escape() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;

    let vars = tmp.child("vars.json");
    vars.write_str(
        r#"
        {
            "user": {
                "id":"1",
                "name": "florian",
                "path": "/users/1"
            }
        }
    "#,
    )?;

    Command::cargo_bin(env!("CARGO_PKG_NAME"))?
        .write_stdin("{{ user.id }},{{ user.name }},{{ user.path }}")
        .arg(vars.path())
        .arg("-e")
        .assert()
        .success()
        .stdout("1,florian,&#x2F;users&#x2F;1");

    Ok(())
}

#[test]
fn test_read_from_file() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;

    let vars = tmp.child("vars.json");
    vars.write_str(
        r#"
        {
            "user": {
                "id":"1",
                "name": "florian",
                "path": "/users/1"
            }
        }
    "#,
    )?;

    let template = tmp.child("template.tera");
    template.write_str("{{ user.id }},{{ user.name }},{{ user.path }}")?;

    Command::cargo_bin(env!("CARGO_PKG_NAME"))?
        .arg("-t")
        .arg(template.path())
        .arg(vars.path())
        .assert()
        .success()
        .stdout("1,florian,/users/1");

    Ok(())
}

#[test]
fn test_write_to_missing_file() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;

    let vars = tmp.child("vars.json");
    vars.write_str(
        r#"
        {
            "user": {
                "id":"1",
                "name": "florian",
                "path": "/users/1"
            }
        }
    "#,
    )?;
    let output = tmp.child("output.txt");

    Command::cargo_bin(env!("CARGO_PKG_NAME"))?
        .write_stdin("{{ user.id }},{{ user.name }},{{ user.path }}")
        .arg("-o")
        .arg(output.path())
        .arg(vars.path())
        .assert()
        .success()
        .stdout("");

    output.assert("1,florian,/users/1");

    Ok(())
}

#[test]
fn test_write_to_existing_file() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;

    let vars = tmp.child("vars.json");
    vars.write_str(
        r#"
        {
            "user": {
                "id":"1",
                "name": "florian",
                "path": "/users/1"
            }
        }
    "#,
    )?;

    let output = tmp.child("output.txt");
    output.touch()?;

    Command::cargo_bin(env!("CARGO_PKG_NAME"))?
        .write_stdin("{{ user.id }},{{ user.name }},{{ user.path }}")
        .arg("-o")
        .arg(output.path())
        .arg(vars.path())
        .assert()
        .success()
        .stdout("");

    output.assert("1,florian,/users/1");

    Ok(())
}

#[cfg(not(target_os = "windows"))]
const TEMPLATE_NOT_FOUND_ERROR: &str = r#"Error: Failed to load template from file

Caused by:
    No such file or directory (os error 2)
"#;

#[cfg(target_os = "windows")]
const TEMPLATE_NOT_FOUND_ERROR: &str = r#"Error: Failed to load template from file

Caused by:
    The system cannot find the file specified. (os error 2)
"#;

#[test]
fn test_template_file_not_found() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;

    let vars = tmp.child("vars.json");
    vars.write_str(
        r#"
        {
            "user": {
                "id":"1",
                "path": "/users/1"
            }
        }
    "#,
    )?;

    Command::cargo_bin(env!("CARGO_PKG_NAME"))?
        .arg("-t")
        .arg(tmp.child("template.tera").path())
        .arg(vars.path())
        .assert()
        .failure()
        .code(1)
        .stdout("")
        .stderr(TEMPLATE_NOT_FOUND_ERROR);

    Ok(())
}

const UNDEFINED_VARIABLE_ERROR: &str = r#"Error: Failed to render template

Caused by:
    0: Failed to render '__tera_one_off'
    1: Variable `user.name` not found in context while rendering '__tera_one_off'
"#;

#[test]
fn test_undefined_var() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;

    let vars = tmp.child("vars.json");
    vars.write_str(
        r#"
        {
            "user": {
                "id":"1",
                "path": "/users/1"
            }
        }
    "#,
    )?;

    Command::cargo_bin(env!("CARGO_PKG_NAME"))?
        .write_stdin("{{ user.id }},{{ user.name }},{{ user.path }}")
        .arg(vars.path())
        .assert()
        .failure()
        .code(1)
        .stdout("")
        .stderr(UNDEFINED_VARIABLE_ERROR);

    Ok(())
}
