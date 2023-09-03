use anyhow::{Context, Result};
use opentelemetry::global;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_stdout as stdout;
use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{fmt::MakeWriter, layer::SubscriberExt, EnvFilter, Registry};

pub fn get_tracing_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> Result<impl Subscriber + Send + Sync>
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name(name.clone())
        .install_simple()
        .with_context(|| "Failed to install opentelemetry-jaeger")?;

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let formatting_layer = BunyanFormattingLayer::new(name.clone(), sink);
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
        .with(telemetry);

    Ok(subscriber)
}
