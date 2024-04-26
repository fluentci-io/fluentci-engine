use std::collections::HashMap;
use std::env;

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
