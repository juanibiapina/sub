extern crate sub;

extern crate clap;

use clap::{App, Arg, AppSettings};

use std::fs;

fn main() {
    let app = init_cli();

    let matches = app.get_matches();

    let name = matches.value_of("name").unwrap();
    let root = fs::canonicalize(matches.value_of("root").unwrap()).unwrap();
    let args = matches.values_of("commands").and_then(|args| Some(args.map(|s| s.to_owned()).collect::<Vec<_>>()));

    let cli = sub::CLI::new(name, root, args);

    cli.run();
}

fn init_cli<'a, 'b>() -> App<'a, 'b> {
    App::new("sub")
        .version(env!("CARGO_PKG_VERSION"))
        .setting(AppSettings::ColoredHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .setting(AppSettings::AllowLeadingHyphen)
        .setting(AppSettings::TrailingVarArg)
        .arg(Arg::with_name("name")
             .long("name")
             .required(true)
             .takes_value(true)
             .help("Sets the binary name"))
        .arg(Arg::with_name("root")
             .long("root")
             .required(true)
             .takes_value(true)
             .help("Sets the root directory"))
        .arg(Arg::with_name("commands")
             .allow_hyphen_values(true)
             .last(true)
             .multiple(true))
}
