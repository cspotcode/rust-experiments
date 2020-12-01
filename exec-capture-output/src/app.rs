use std::path::{PathBuf, Path};
use std::io::Read;
use std::process::{Command};

use anyhow::Result;
use structopt::StructOpt;
use pty;
use pty::fork::{Fork, Master};
use which::which;

use log::{debug, error, info, trace, warn};

// Local Imports
use crate::helpers::{BoilerplateOpts, HELP_TEMPLATE};
use crate::validators::path_readable_file;

/// The verbosity level when no `-q` or `-v` arguments are given, with `0` being `-q`
pub const DEFAULT_VERBOSITY: u64 = 1;

/// Command-line argument schema
///
/// ## Relevant Conventions:
///
///  * Make sure that there is a blank space between the `<name>` `<version>` line and the
///    description text or the `--help` output won't comply with the platform conventions that
///    `help2man` depends on to generate your manpage. (Specifically, it will mistake the `<name>
///    <version>` line for part of the description.)
///  * `StructOpt`'s default behaviour of including the author name in the `--help` output is an
///    oddity among Linux commands and, if you don't disable it, you run the risk of people
///    unfamiliar with `StructOpt` assuming that you are an egotistical person who made a conscious
///    choice to add it.
///
///    The proper standardized location for author information is the `AUTHOR` section which you
///    can read about by typing `man help2man`.
///
/// ## Cautions:
///  * Subcommands do not inherit `template` and it must be re-specified for each one.
///    ([clap-rs/clap#1184](https://github.com/clap-rs/clap/issues/1184))
///  * Double-check that your choice of `about` or `long_about` is actually overriding this
///    doc comment. The precedence has some bugs such as
///    [TeXitoi/structopt#391](https://github.com/TeXitoi/structopt/issues/391) and
///    [TeXitoi/structopt#333](https://github.com/TeXitoi/structopt/issues/333).
///  * Do not begin the description text for subcommands with `\n`. It will break the formatting in
///    the top-level help output's list of subcommands.
#[derive(StructOpt, Debug)]
#[structopt(template = HELP_TEMPLATE,
            about = "Run a command in a pty, capturing the colorized output to a file.",
            global_setting = structopt::clap::AppSettings::ColoredHelp)]
pub struct CliOpts {
    #[allow(clippy::missing_docs_in_private_items)] // StructOpt compile-time errors if we doc this
    #[structopt(flatten)]
    pub boilerplate: BoilerplateOpts,

    #[structopt(parse(from_os_str), long = "output-file", short = "o")]
    outpath: PathBuf,

    #[structopt(name = "command")]
    command_args: Vec<String>,
}

/// main entrypoint, invoked by our arg parsing boilerplate
pub async fn main(opts: CliOpts) -> Result<()> {
    // for command_arg in &opts.command_args {
    //     println!("{}", command_arg)
    // }

    // Find binary
    let command_exists = Path::new(&opts.command_args[0]).exists();
    let executable_path: PathBuf;
    if command_exists {
        executable_path= PathBuf::from(&opts.command_args[0])
    } else {
        executable_path= which(&opts.command_args[0]).unwrap();
    }

    // Fork child process with a pty
    let fork = Fork::from_ptmx().unwrap();


    if let Some(mut master) = fork.is_parent().ok() {
        // Parent process
        let mut output = String::new();
        match master.read_to_string(&mut output) {
            Ok(_nread) => println!("child tty is: {}", output.trim()),
            Err(e) => panic!("read error: {}", e),
        }
    } else {
        // Child process
        let mut command = Command::new(&executable_path);
        for command_arg in (&opts.command_args).iter().skip(1) {
            command.arg(&command_arg);
        }
        let status = command.status().expect("could not execute command");
        std::process::exit(status.code().expect("could not get exit code"));
    }

    Ok(())
}

// Tests go below the code where they'll be out of the way when not the target of attention
#[cfg(test)]
mod tests {
    use super::CliOpts;

    // TODO: Unit test to verify that the doc comments on `CliOpts` or `BoilerplateOpts` aren't
    // overriding the intended about string.

    #[test]
    /// Test something
    fn test_something() {
        // TODO: Test something
    }
}
