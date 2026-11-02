//! Opt-in tests for disposable native CI hosts. These install jq.
use fluentci_ext::{bsd::BsdPackages, Extension};
use fluentci_types::Output;
use std::{sync::mpsc, time::Duration};

fn verify(mut extension: impl Extension) {
    extension.setup().unwrap();
    let (tx, rx) = mpsc::channel();
    let status = extension
        .exec("jq --version", tx, Output::Stdout, true, ".")
        .unwrap();
    assert!(status.success());
    assert!(rx
        .recv_timeout(Duration::from_secs(10))
        .unwrap()
        .contains("jq-"));
}

#[test]
#[ignore = "installs packages; run only on a disposable native BSD CI host"]
fn native_bsd_package_install() {
    let adapter = match std::env::consts::OS {
        "freebsd" => BsdPackages::freebsd(),
        "netbsd" => BsdPackages::netbsd(),
        "openbsd" => BsdPackages::openbsd(),
        other => panic!("requires a BSD host, got {other}"),
    };
    adapter.install(vec!["jq"]).unwrap();
    verify(adapter);
}

#[test]
#[ignore = "installs packages; run only on a disposable supported Homebrew CI host"]
fn native_homebrew_package_install() {
    let adapter = fluentci_ext::homebrew::Homebrew;
    adapter.install(vec!["jq"]).unwrap();
    verify(adapter);
}

#[test]
#[ignore = "installs packages; requires a provisioned disposable macOS host and prefix permissions"]
fn native_macports_package_install() {
    use fluentci_ext::macports::{MacPorts, MacPortsBackend};
    let backend = match std::env::var("FLUENTCI_TEST_MACPORTS_BACKEND").as_deref() {
        Ok("rust") => MacPortsBackend::Rust,
        Ok("native") | Err(_) => MacPortsBackend::Native,
        Ok(value) => panic!("invalid backend {value}"),
    };
    let mut adapter = MacPorts::new(backend);
    if let Ok(binary) = std::env::var("FLUENTCI_TEST_MACPORTS_BINARY") {
        adapter = adapter.with_executable(binary);
    }
    adapter.install(vec!["jq"]).unwrap();
    verify(adapter);
}
