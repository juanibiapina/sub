extern crate itertools;
extern crate clap;

use clap::{App, Arg, AppSettings};

pub struct CLI {
    alias: String,
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
                 .takes_value(true))
            .arg(Arg::with_name("commands")
                 .multiple(true));

        let matches = app.get_matches();

        CLI {
            alias: matches.value_of("alias").unwrap_or("sub").to_owned(),
        }
    }

    pub fn run(&self) {
        let app = App::new(self.alias.as_ref())
            .bin_name(self.alias.as_ref())
            .version(env!("CARGO_PKG_VERSION"))
            .setting(AppSettings::ColoredHelp)
            .setting(AppSettings::SubcommandRequiredElseHelp)
            .setting(AppSettings::VersionlessSubcommands)
            .setting(AppSettings::TrailingVarArg)
            .arg(Arg::with_name("alias")
                 .long("alias")
                 .takes_value(true)
                 .hidden(true));

        let _matches = app.get_matches();
    }
}
