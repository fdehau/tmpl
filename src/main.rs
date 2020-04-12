use anyhow::Context;
use argh::FromArgs;
use serde_json::Value;
use std::{
    fs,
    io::Write,
    io::{self, Read},
    path::PathBuf,
};

fn merge(a: &mut Value, b: &Value) {
    match (a, b) {
        (&mut Value::Object(ref mut a), &Value::Object(ref b)) => {
            for (k, v) in b {
                merge(a.entry(k.clone()).or_insert(Value::Null), v);
            }
        }
        (a, b) => {
            *a = b.clone();
        }
    }
}

/// Render Tera template from the command line
#[derive(Debug, PartialEq, FromArgs)]
struct Args {
    /// a template to render. If not specified the template is read from stdin.
    #[argh(option, short = 't')]
    template: Option<PathBuf>,
    /// an output file to write the rendered template. If not specified the template is rendered to
    /// stdout.
    #[argh(option, short = 'o')]
    output: Option<PathBuf>,
    /// whether the output will be escaped.
    #[argh(switch, short = 'e')]
    escape: bool,
    /// a list of json files containing variables used when rendering the template.
    #[argh(positional)]
    sources: Vec<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let args: Args = argh::from_env();
    let mut variables = Value::Null;
    for source in args.sources {
        let file = fs::File::open(&source)
            .with_context(|| format!("Failed to open {}", source.display()))?;
        let json = serde_json::from_reader(file)?;
        merge(&mut variables, &json);
    }

    let template = if let Some(template) = args.template {
        fs::read_to_string(template).context("Failed to load template from file")?
    } else {
        let mut buffer = String::new();
        let mut stdin = io::stdin();
        stdin
            .read_to_string(&mut buffer)
            .context("Failed to load template from stdin")?;
        buffer
    };

    let ctx = if variables == Value::Null {
        tera::Context::new()
    } else {
        tera::Context::from_serialize(variables).context("Failed to create template context")?
    };
    let result =
        tera::Tera::one_off(&template, &ctx, args.escape).context("Failed to render template")?;

    if let Some(output) = args.output {
        fs::File::open(&output)
            .with_context(|| format!("Failed to open {}", output.display()))?
            .write_all(result.as_bytes())?;
    } else {
        io::stdout()
            .write_all(result.as_bytes())
            .context("Failed to write result to stdout")?;
    }
    Ok(())
}
