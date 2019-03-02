extern crate clap;
extern crate itertools;

use clap::{App, AppSettings, Arg, SubCommand};

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::exit;
use std::process::Command;

pub struct Sub {
    name: String,
    root: PathBuf,
    args: Vec<String>,
}

impl Sub {
    pub fn new(name: &str, root: PathBuf, args: Vec<String>) -> Sub {
        Sub {
            name: name.to_owned(),
            root,
            args,
        }
    }

    pub fn run(&self) -> ! {
        let mut args = self.args.clone();

        if args.len() == 0 {
            self.display_help();
            exit(0);
        }

        let command_args = {
            if args.len() > 1 {
                args.drain(1..).collect()
            } else {
                Vec::new()
            }
        };
        let command_name = args.pop().unwrap();

        if command_name == "help" {
            self.display_help();
            exit(0);
        }

        let command_path = self.command_path(&command_name);

        let mut command = Command::new(command_path);
        command.args(command_args);

        let status = command.status().unwrap();

        match status.code() {
            Some(code) => exit(code),
            None => exit(1),
        }
    }

    fn display_help(&self) {
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
                let entry = entry.unwrap();
                let name = entry.file_name().into_string().unwrap();

                if name.starts_with(".") {
                    continue;
                }

                if entry.metadata().unwrap().permissions().mode() & 0o111 == 0 {
                    continue;
                }

                app = app.subcommand(
                    SubCommand::with_name(name.as_ref())
                    .setting(AppSettings::TrailingVarArg)
                    .setting(AppSettings::AllowLeadingHyphen)
                    .arg(Arg::with_name("args")
                         .hidden(true)
                         .multiple(true)),
                         );
            }
        }

        app.print_help().unwrap();
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
}
