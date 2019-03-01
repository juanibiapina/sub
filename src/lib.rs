extern crate itertools;
extern crate clap;

use clap::{App, Arg, AppSettings};
use itertools::Itertools;

use std::env;

pub struct CLI;

impl CLI {
    pub fn new() -> CLI {
        CLI
    }

    pub fn run(&self) {
        let mut name = "sub".to_owned();

        for (arg, value) in env::args().tuple_windows() {
            if arg == "--alias" {
                name = value;
                break;
            }
        }

        let app = App::new(name.as_ref())
            .bin_name(name.as_ref())
            .version(env!("CARGO_PKG_VERSION"))
            .setting(AppSettings::ColoredHelp)
            .setting(AppSettings::SubcommandRequiredElseHelp)
            .arg(Arg::with_name("alias")
                 .long("alias")
                 .takes_value(true)
                 .hidden(true));

        let _matches = app.get_matches();
    }
}
