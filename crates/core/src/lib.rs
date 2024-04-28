use std::collections::HashMap;
use std::env;
use std::process::Command;

use anyhow::Error;
use opentelemetry::trace::noop::NoopTracerProvider;
use opentelemetry::trace::TraceError;
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::{Protocol, WithExportConfig};
use opentelemetry_sdk::trace::config;
use opentelemetry_sdk::Resource;

pub mod deps;
pub mod edge;
pub mod vertex;

pub fn init_tracer() -> Result<(), TraceError> {
    if let Ok(endpoint) = env::var("OTEL_EXPORTER_OTLP_ENDPOINT") {
        let protocol = match env::var("OTEL_EXPORTER_OTLP_PROTOCOL") {
            Ok(protocol) => match protocol.as_str() {
                "grpc" => Protocol::Grpc,
                _ => Protocol::HttpBinary,
            },
            Err(_) => Protocol::HttpBinary,
        };
        let export_config = opentelemetry_otlp::ExportConfig {
            endpoint,
            protocol,
            ..opentelemetry_otlp::ExportConfig::default()
        };

        let mut headers = HashMap::new();
        let tracing = opentelemetry_otlp::new_pipeline().tracing();

        if let Ok(api_key) = env::var("OTLP_API_KEY") {
            headers.insert("x-api-key".into(), api_key);
        }

        if let Ok(honeycomb_api_key) = env::var("HONEYCOMB_API_KEY") {
            headers.insert("x-honeycomb-team".into(), honeycomb_api_key);
        }

        if let Ok(honeycomb_dataset) = env::var("HONEYCOMB_DATASET") {
            headers.insert("x-honeycomb-dataset".into(), honeycomb_dataset);
        }

        if let Ok(baselime_dataset) = env::var("BASELIME_DATASET") {
            headers.insert("x-baselime-dataset".into(), baselime_dataset);
        }

        let tracing = match protocol {
            Protocol::Grpc => tracing.with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_export_config(export_config),
            ),
            _ => tracing.with_exporter(
                opentelemetry_otlp::new_exporter()
                    .http()
                    .with_export_config(export_config)
                    .with_headers(headers),
            ),
        };

        let _ = tracing
            .with_trace_config(config().with_resource(Resource::new(vec![
                KeyValue::new("service.name", "fluentci-engine"),
                KeyValue::new("service.namespace", "fluentci-core"),
                KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                KeyValue::new("exporter", "otlp"),
            ])))
            .install_batch(opentelemetry_sdk::runtime::Tokio)?;

        return Ok(());
    }

    if let Ok(endpoint) = env::var("OTEL_EXPORTER_JAEGER_AGENT_HOST") {
        #[allow(deprecated)]
        let _ = opentelemetry_jaeger::new_agent_pipeline()
            .with_endpoint(endpoint)
            .with_service_name("fluentci-core")
            .with_trace_config(config().with_resource(Resource::new(vec![
                KeyValue::new("service.name", "fluentci-engine"),
                KeyValue::new("service.namespace", "fluentci-core"),
                KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                KeyValue::new("exporter", "jaeger"),
            ])))
            .install_simple()?;

        return Ok(());
    }

    if let Ok(endpoint) = env::var("OTEL_EXPORTER_ZIPKIN_ENDPOINT") {
        let _ = opentelemetry_zipkin::new_pipeline()
            .with_service_name("fluentci-core")
            .with_collector_endpoint(endpoint)
            .install_batch(opentelemetry_sdk::runtime::Tokio)?;

        return Ok(());
    }

    let provider = NoopTracerProvider::new();
    let _ = global::set_tracer_provider(provider);
    Ok(())
}

pub fn set_git_repo_metadata() -> Result<(), Error> {
    let child = Command::new("sh")
        .arg("-c")
        .arg("git log -1 --pretty=%s")
        .stdout(std::process::Stdio::piped())
        .spawn()?;
    let output = child.wait_with_output()?;
    let commit_message = String::from_utf8(output.stdout)?;

    let child = Command::new("sh")
        .arg("-c")
        .arg("git rev-parse --abbrev-ref HEAD")
        .stdout(std::process::Stdio::piped())
        .spawn()?;
    let output = child.wait_with_output()?;
    let branch = String::from_utf8(output.stdout)?;

    let child = Command::new("sh")
        .arg("-c")
        .arg("git log -1 --pretty=%h")
        .stdout(std::process::Stdio::piped())
        .spawn()?;
    let output = child.wait_with_output()?;
    let commit_hash = String::from_utf8(output.stdout)?;

    let child = Command::new("sh")
        .arg("-c")
        .arg("git remote get-url origin")
        .stdout(std::process::Stdio::piped())
        .spawn()?;
    let output = child.wait_with_output()?;
    let remote_url = String::from_utf8(output.stdout)?;

    let child = Command::new("sh")
        .arg("-c")
        .arg("git log -1 --pretty=%an")
        .stdout(std::process::Stdio::piped())
        .spawn()?;
    let output = child.wait_with_output()?;
    let author = String::from_utf8(output.stdout)?;

    env::set_var("GIT_COMMIT_MESSAGE", commit_message.trim());
    env::set_var("GIT_BRANCH", branch.trim());
    env::set_var("GIT_COMMIT_HASH", commit_hash.trim());
    env::set_var("GIT_REMOTE_URL", remote_url.trim());
    env::set_var("GIT_AUTHOR", author.trim());
    Ok(())
}
