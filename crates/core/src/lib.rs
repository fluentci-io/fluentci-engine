use std::env;

use opentelemetry::trace::noop::NoopTracerProvider;
use opentelemetry::trace::TraceError;
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::config;
use opentelemetry_sdk::Resource;

pub mod deps;
pub mod edge;
pub mod vertex;

pub fn init_tracer() -> Result<(), TraceError> {
    if let Ok(endpoint) = env::var("OTEL_EXPORTER_OTLP_ENDPOINT") {
        let export_config = opentelemetry_otlp::ExportConfig {
            endpoint,
            ..opentelemetry_otlp::ExportConfig::default()
        };
        let _ = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .http()
                    .with_export_config(export_config),
            )
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
