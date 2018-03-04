extern crate clap;
extern crate failure;
extern crate serde_json;
extern crate tera;

use std::path::Path;
use std::path::PathBuf;

use serde_json::Value;
use clap::{App, Arg};
use failure::{Error, SyncFailure};
use tera::Tera;

fn merge(a: &mut Value, b: &Value) {
    match (a, b) {
        (&mut Value::Object(ref mut a), &Value::Object(ref b)) => for (k, v) in b {
            merge(a.entry(k.clone()).or_insert(Value::Null), v);
        },
        (a, b) => {
            *a = b.clone();
        }
    }
}

fn app() -> App<'static, 'static> {
    App::new("tmpl")
        .version("0.1")
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

fn main() {
    use std::process::exit;

    let args = Args::parse();
    exit(match render(args) {
        Err(e) => {
            for cause in e.causes() {
                eprintln!("{}", cause);
            }
            1
        }
        Ok(()) => 0,
    })
}

fn render(args: Args) -> Result<(), Error> {
    use std::fs::File;
    use std::io::Read;
    use std::io::Write;

    let mut variables = Value::Null;
    for source in args.sources {
        let file = File::open(source)?;
        let mut json = serde_json::from_reader(file)?;
        merge(&mut variables, &mut json);
    }

    let mut template = String::new();
    File::open(args.template)?.read_to_string(&mut template)?;

    let result = Tera::one_off(&template, &variables, true).map_err(SyncFailure::new)?;

    if let Some(output) = args.output {
        File::open(output)?.write(result.as_bytes())?;
    } else {
        std::io::stdout().write(result.as_bytes())?;
    }

    Ok(())
}
