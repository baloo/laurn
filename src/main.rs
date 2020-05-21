use std::env::current_dir;
use std::io::Error as IoError;
use std::path::Path;

use clap::{App, Arg, SubCommand};
use liblaurn::config::{load_config, Config, ConfigError};
use liblaurn::{run, BuildError, Container};

#[derive(Debug)]
enum Error {
    Run(run::RunError),
    Build(BuildError),
    CurrentDir(IoError),
    CurrentExe(IoError),
    Config(ConfigError),
}

// A dummy hook in bash
fn hook_bash() -> Result<String, Error> {
    let exe = std::env::current_exe().map_err(Error::CurrentExe)?;
    Ok(format!(
        r#"
LAURN_PREVIOUS_PATH=""
_laurn_hook() {{
    local previous_exit_status=$?;
    if [ "$(pwd)" != "$LAURN_PREVIOUS_PATH" ]; then
        LAURN_PREVIOUS_PATH="$(pwd)";
        if [ -e .laurnrc ]; then
           {laurn} shell;
        else
            return $previous_exit_status;
        fi
    else
        return $previous_exit_status;
    fi
}}
if ! [[ "${{PROMPT_COMMAND:-}}" =~ _laurn_hook ]]; then
  PROMPT_COMMAND="_laurn_hook${{PROMPT_COMMAND:+;$PROMPT_COMMAND}}"
fi
"#,
        laurn = exe.as_path().display()
    ))
}

fn main() -> Result<(), Error> {
    let matches = App::new("laurn")
        .version("0.0.1")
        .author("Arthur Gautier <laurn@superbaloo.net>")
        .about("nix-based containers")
        .subcommand(
            SubCommand::with_name("run").about("run a container").arg(
                Arg::with_name("path")
                    .short("p")
                    .value_name("FILE")
                    .takes_value(true)
                    .required(true)
                    .help("path to the root nix definition"),
            ),
        )
        .subcommand(SubCommand::with_name("shell").about("start a shell in the current directory"))
        .subcommand(
            SubCommand::with_name("hook")
                .about("hook into a shell")
                .subcommand(SubCommand::with_name("bash").about("hook into bash")),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("run") {
        let source = matches.value_of("path").unwrap();
        let source = Path::new(source);
        let laurn_config = Config::default();

        let container = Container::build(source).map_err(Error::Build)?;

        let code = run::run(container, laurn_config).map_err(Error::Run)?;
        std::process::exit(code)
    } else if let Some(_matches) = matches.subcommand_matches("shell") {
        let project_dir = current_dir().map_err(Error::CurrentDir)?;
        let laurn_config_file = project_dir.join(".laurnrc");

        let laurn_config = load_config(laurn_config_file.as_path()).map_err(Error::Config)?;

        let source = project_dir.join("laurn.nix");
        let container = Container::build(source.as_path()).map_err(Error::Build)?;

        let code = run::run(container, laurn_config).map_err(Error::Run)?;
        std::process::exit(code)
    } else if let Some(matches) = matches.subcommand_matches("hook") {
        if let Some(_) = matches.subcommand_matches("bash") {
            if let Ok(hook) = hook_bash() {
                println!("{}", hook);
                std::process::exit(0);
            }
        }
    }
    std::process::exit(1)
}
