extern crate itertools;
extern crate clap;

use clap::{App, Arg, AppSettings, SubCommand};

use std::fs;
use std::path::PathBuf;

pub struct CLI {
    alias: String,
    root: PathBuf,
}

impl CLI {
    pub fn new() -> CLI {
        let app = App::new("sub")
            .version(env!("CARGO_PKG_VERSION"))
            .setting(AppSettings::ColoredHelp)
            .setting(AppSettings::VersionlessSubcommands)
            .setting(AppSettings::TrailingVarArg)
            .arg(Arg::with_name("alias")
                 .long("alias")
                 .required(true)
                 .takes_value(true)
                 .help("Sets the binary name"))
            .arg(Arg::with_name("root")
                 .long("root")
                 .required(true)
                 .takes_value(true)
                 .help("Sets the root directory"))
            .arg(Arg::with_name("commands")
                 .multiple(true));

        let matches = app.get_matches();

        CLI {
            alias: matches.value_of("alias").unwrap().to_owned(),
            root: fs::canonicalize(matches.value_of("root").unwrap()).unwrap(),
        }
    }

    pub fn run(&self) {
        let mut app = App::new(self.alias.as_ref())
            .bin_name(self.alias.as_ref())
            .setting(AppSettings::ColoredHelp)
            .setting(AppSettings::SubcommandRequiredElseHelp)
            .setting(AppSettings::DisableVersion)
            .setting(AppSettings::VersionlessSubcommands)
            .arg(Arg::with_name("alias")
                 .long("alias")
                 .takes_value(true)
                 .hidden(true))
            .arg(Arg::with_name("root")
                 .long("root")
                 .hidden(true)
                 .takes_value(true));

        let libexec_path = self.libexec_path();

        if libexec_path.is_dir() {
            for entry in fs::read_dir(self.libexec_path()).unwrap() {
                let name = entry.unwrap().file_name().into_string().unwrap();
                app = app.subcommand(SubCommand::with_name(name.as_ref()));
            }
        }

        let _matches = app.get_matches();
    }

    fn libexec_path(&self) -> PathBuf {
        let mut path = self.root.clone();

        path.push("libexec");

        path
    }
}
