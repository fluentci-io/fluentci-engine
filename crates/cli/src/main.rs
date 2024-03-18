use anyhow::Error;
use clap::{arg, Arg, Command};
use cmd::run::run;
use cmd::serve::serve;

pub mod cmd;

fn cli() -> Command<'static> {
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
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
            Command::new("serve")
                .arg(arg!(--listen "The address to listen on").default_value("127.0.0.1:6880"))
                .about("Starts the FluentCI Engine server"),
        )
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("run", args)) => {
            let command = args.values_of("command").unwrap();
            run(&command.collect::<Vec<_>>().join(" ")).await?;
        }
        Some(("serve", args)) => {
            let listen = args.value_of("listen").unwrap();
            serve(listen).await?;
        }
        _ => cli().print_help()?,
    };
    Ok(())
}
