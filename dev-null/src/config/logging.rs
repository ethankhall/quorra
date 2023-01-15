use clap::{ArgAction, ArgGroup, Parser};
use lazy_static::lazy_static;
use opentelemetry::{
    global, runtime,
    sdk::{
        export::metrics::aggregation::cumulative_temporality_selector,
        metrics::selectors,
        propagation::TraceContextPropagator,
        trace::{self, RandomIdGenerator, Sampler},
        Resource,
    },
    KeyValue,
};
use opentelemetry_otlp::{ExportConfig, Protocol, TonicExporterBuilder, WithExportConfig};
use std::time::Duration;
use tracing::{self, level_filters::LevelFilter};
use tracing_opentelemetry::MetricsLayer;
use tracing_subscriber::{
    filter::filter_fn,
    fmt::format::{Format, JsonFields, PrettyFields},
    layer::SubscriberExt,
    Layer, Registry,
};

lazy_static! {
    static ref IGNORED_MODULES: &'static [&'static str] = &[
        "want",
        "hyper",
        "mio",
        "rustls",
        "tokio_threadpool",
        "tokio_reactor",
        "tower",
        "tonic",
        "h2",
    ];
}

#[derive(Parser, Debug)]
pub struct RuntimeArgs {
    /// The URL to publish metrics to.
    #[clap(
        long = "otel-collector",
        env = "OTEL_EXPORTER_OTLP_TRACES_ENDPOINT",
        default_value("http://localhost:4317")
    )]
    otel_collector: String,
}

#[derive(Parser, Debug)]
#[clap(group = ArgGroup::new("logging"))]
pub struct LoggingOpts {
    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, action = ArgAction::Count, global(true), group = "logging")]
    pub debug: u8,

    /// Enable warn logging
    #[clap(short, long, global(true), group = "logging")]
    pub warn: bool,

    /// Disable everything but error logging
    #[clap(short, long, global(true), group = "logging")]
    pub error: bool,
}

impl From<&LoggingOpts> for LevelFilter {
    fn from(opts: &LoggingOpts) -> Self {
        if opts.error {
            LevelFilter::ERROR
        } else if opts.warn {
            LevelFilter::WARN
        } else if opts.debug == 0 {
            LevelFilter::INFO
        } else if opts.debug == 1 {
            LevelFilter::DEBUG
        } else {
            LevelFilter::TRACE
        }
    }
}

pub fn configure_logging(logging_opts: &LoggingOpts, runtime_args: &RuntimeArgs) {
    global::set_text_map_propagator(TraceContextPropagator::new());

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(make_exporter(runtime_args))
        .with_trace_config(
            trace::config()
                .with_sampler(Sampler::AlwaysOn)
                .with_id_generator(RandomIdGenerator::default())
                .with_max_events_per_span(64)
                .with_max_attributes_per_span(16)
                .with_max_events_per_span(16)
                .with_resource(Resource::new(vec![KeyValue::new(
                    "service.name",
                    "dev-null",
                )])),
        )
        .install_batch(runtime::Tokio)
        .unwrap();

    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    let is_terminal = atty::is(atty::Stream::Stdout) && cfg!(debug_assertions);
    let pretty_logger = if is_terminal {
        Some(
            tracing_subscriber::fmt::layer()
                .event_format(Format::default().pretty())
                .fmt_fields(PrettyFields::new())
                .with_filter(filter_fn(|metadata| {
                    !metadata
                        .module_path()
                        .map(|x| IGNORED_MODULES.iter().any(|module| x.starts_with(module)))
                        .unwrap_or(true)
                })),
        )
    } else {
        None
    };

    let json_logger = if !is_terminal {
        Some(
            tracing_subscriber::fmt::layer()
                .event_format(Format::default().json().flatten_event(true))
                .fmt_fields(JsonFields::new())
                .with_filter(filter_fn(|metadata| {
                    !metadata
                        .module_path()
                        .map(|x| IGNORED_MODULES.iter().any(|module| x.starts_with(module)))
                        .unwrap_or(true)
                })),
        )
    } else {
        None
    };

    let meter = opentelemetry_otlp::new_pipeline()
        .metrics(
            selectors::simple::inexpensive(),
            cumulative_temporality_selector(),
            runtime::Tokio,
        )
        .with_exporter(make_exporter(runtime_args))
        .with_period(Duration::from_secs(3))
        .with_timeout(Duration::from_secs(10))
        .build()
        .unwrap();

    let opentelemetry_metrics = MetricsLayer::new(meter);

    let subscriber = Registry::default()
        .with(LevelFilter::from(logging_opts))
        .with(otel_layer)
        .with(opentelemetry_metrics)
        .with(json_logger)
        .with(pretty_logger);

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    tracing_log::LogTracer::init().expect("logging to work correctly");
}

fn make_exporter(runtime_args: &RuntimeArgs) -> TonicExporterBuilder {
    let export_config = ExportConfig {
        endpoint: runtime_args.otel_collector.to_string(),
        timeout: Duration::from_secs(3),
        protocol: Protocol::Grpc,
    };

    opentelemetry_otlp::new_exporter()
        .tonic()
        .with_export_config(export_config)
}
