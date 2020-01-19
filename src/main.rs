use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use clap::{App, Arg};
use serde_json::Value;
use tera::{Context, Tera};

const VERSION: &str = env!("CARGO_PKG_VERSION");

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

fn app() -> App<'static, 'static> {
    App::new("tmpl")
        .version(VERSION)
        .author("Florian Dehau")
        .about("Renders Tera templates from the command line")
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .help("Write output to the given file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("sources")
                .value_name("SRC")
                .help("JSON files containing the variable")
                .required(true)
                .multiple(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("template")
                .value_name("TEMPLATE")
                .required(true)
                .takes_value(true),
        )
}

struct Args {
    sources: Vec<PathBuf>,
    template: PathBuf,
    output: Option<PathBuf>,
}

impl Args {
    fn parse() -> Args {
        let matches = app().get_matches();
        let sources = matches
            .values_of("sources")
            .unwrap()
            .map(|p| Path::new(p).to_path_buf())
            .collect();
        let template = Path::new(matches.value_of("template").unwrap()).to_path_buf();
        let output = matches
            .value_of("output")
            .map(|p| Path::new(p).to_path_buf());
        Args {
            sources,
            template,
            output,
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let mut variables = Value::Null;
    for source in args.sources {
        let file = fs::File::open(source)?;
        let json = serde_json::from_reader(file)?;
        merge(&mut variables, &json);
    }

    let template = fs::read_to_string(args.template)?;

    let ctx = Context::from_serialize(variables)?;
    let result = Tera::one_off(&template, &ctx, true)?;

    if let Some(output) = args.output {
        fs::File::open(output)?.write_all(result.as_bytes())?;
    } else {
        std::io::stdout().write_all(result.as_bytes())?;
    }
    Ok(())
}
