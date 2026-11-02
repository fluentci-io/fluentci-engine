use anyhow::Error;
use clap::{arg, Arg, Command};
use cmd::call::call;
use cmd::run::run;
use cmd::serve::serve;

pub mod cmd;

fn cli() -> Command<'static> {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    Command::new("fluentci-engine")
        .version(VERSION)
        .about(
            r#"

       ________                 __  __________   ______            _
      / ____/ /_  _____  ____  / /_/ ____/  _/  / ____/___  ____ _(_)___  ___
     / /_  / / / / / _ \/ __ \/ __/ /    / /   / __/ / __ \/ __ `/ / __ \/ _ \
    / __/ / / /_/ /  __/ / / / /_/ /____/ /   / /___/ / / / /_/ / / / / /  __/
   /_/   /_/\__,_/\___/_/ /_/\__/\____/___/  /_____/_/ /_/\__, /_/_/ /_/\___/
                                                         /____/

   A Programmable CI/CD engine without Containers, built on top of Nix ❄️
      "#,
        )
        .author("Tsiry Sandratraina <tsiry.sndr@fluentci.io>")
        .subcommand(
            Command::new("execute")
                .arg(Arg::new("progress").long("progress").takes_value(true).possible_values(["auto", "plain", "tui", "json"]).default_value("auto"))
                .arg(Arg::new("plan").required(true))
                .arg(Arg::new("run-dir").long("run-dir").takes_value(true).required(true))
                .about("Executes an explicit host task plan with bounded parallelism"),
        )
        .subcommand(
            Command::new("run")
                .arg(
                    Arg::new("command")
                        .multiple_occurrences(true)
                        .multiple_values(true)
                        .required(true),
                )
                .about("Executes the specified command in a FluentCI Session"),
        )
        .subcommand(
            Command::new("call")
                .arg(arg!(-m --module <PATH> "path or url to the module to call").required(true))
                .arg(
                    Arg::new("command")
                        .multiple_occurrences(true)
                        .multiple_values(true)
                        .required(true),
                )
                .about("Calls the specified function in a FluentCI Session"),
        )
        .subcommand(
            Command::new("serve")
                .arg(arg!(--listen "The address to listen on").default_value("127.0.0.1:6880"))
                .about("Starts the FluentCI Engine server"),
        )
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("execute", args)) => {
            cmd::execute::execute(args.value_of("plan").unwrap(), args.value_of("run-dir").unwrap(), args.value_of("progress").unwrap()).await?;
        }
        Some(("run", args)) => {
            let command = args.values_of("command").unwrap();
            run(&command.collect::<Vec<_>>().join(" ")).await?;
        }
        Some(("serve", args)) => {
            let listen = args.value_of("listen").unwrap();
            serve(listen).await?;
        }
        Some(("call", args)) => {
            let module = args.value_of("module").unwrap();
            let command = args.values_of("command").unwrap();
            let command = command.collect::<Vec<_>>().join(" ");
            call(module, &command)?;
        }
        _ => cli().print_help()?,
    };
    Ok(())
}
