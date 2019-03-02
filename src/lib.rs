extern crate itertools;
extern crate clap;

use clap::{App, Arg, ArgMatches, AppSettings, SubCommand};

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::process::exit;

pub struct Sub {
    name: String,
    root: PathBuf,
    args: Option<Vec<String>>,
}

impl Sub {
    pub fn new(name: &str, root: PathBuf, args: Option<Vec<String>>) -> Sub {
        Sub {
            name: name.to_owned(),
            root,
            args,
        }
    }

    pub fn run(&self) -> ! {
        let matches = self.init_commands();

        let command_name = matches.subcommand_name().unwrap();

        let command_path = self.command_path(command_name);

        let mut command = Command::new(command_path);

        let sub_matches = matches.subcommand_matches(command_name).unwrap();
        if let Some(args) = sub_matches.values_of("args") {
            command.args(args.collect::<Vec<_>>());
        }

        let status = command.status().unwrap();

        match status.code() {
            Some(code) => exit(code),
            None => exit(1),
        }
    }

    fn command_path(&self, command_name: &str) -> PathBuf {
        let mut libexec_path = self.libexec_path();
        libexec_path.push(command_name);
        libexec_path
    }

    fn libexec_path(&self) -> PathBuf {
        let mut path = self.root.clone();
        path.push("libexec");
        path
    }

    fn init_commands(&self) -> ArgMatches {
        let mut app = App::new(self.name.as_ref())
            .bin_name(self.name.as_ref())
            .setting(AppSettings::ColoredHelp)
            .setting(AppSettings::NoBinaryName)
            .setting(AppSettings::SubcommandRequiredElseHelp)
            .setting(AppSettings::DisableVersion)
            .setting(AppSettings::VersionlessSubcommands);

        let libexec_path = self.libexec_path();

        if libexec_path.is_dir() {
            for entry in fs::read_dir(self.libexec_path()).unwrap() {
                let name = entry.unwrap().file_name().into_string().unwrap();

                app = app.subcommand(SubCommand::with_name(name.as_ref())
                                     .setting(AppSettings::TrailingVarArg)
                                     .setting(AppSettings::AllowLeadingHyphen)
                                     .arg(Arg::with_name("args")
                                          .hidden(true)
                                          .multiple(true)));
            }
        }

        if self.args.is_none() {
            app.print_help().unwrap();
            exit(0);
        }

        app.get_matches_from(self.args.as_ref().unwrap())
    }

}
