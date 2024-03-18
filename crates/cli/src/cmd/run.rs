use get_port::tcp::TcpPort;
use get_port::{Ops, Range};
use std::{
    net::SocketAddr,
    process::{Command, Stdio},
    thread,
};

use anyhow::Error;
use fluentci_server::start;
use tokio::runtime;

pub async fn run(command: &str) -> Result<(), Error> {
    let tcp_port = TcpPort::in_range(
        "127.0.0.1",
        Range {
            min: 6088,
            max: 65535,
        },
    )
    .unwrap();

    let addr = SocketAddr::from(([127, 0, 0, 1], tcp_port));

    let host = addr.ip();
    let port = addr.port();
    let listen = format!("{}:{}", host, port);

    thread::spawn(move || {
        let rt = runtime::Runtime::new().unwrap();
        match rt.block_on(start(&listen)) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    });

    let mut child = Command::new("sh")
        .arg("-c")
        .arg(command)
        .env("FLUENTCI_SESSION_PORT", port.to_string())
        .env("FLUENTCI_SESSION_HOST", host.to_string())
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    let status = child.wait()?;
    if !status.success() {
        std::process::exit(1);
    }
    std::process::exit(0);
}
