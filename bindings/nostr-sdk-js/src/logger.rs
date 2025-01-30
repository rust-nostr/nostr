// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use core::fmt::{self, Write};
use core::sync::atomic::{AtomicUsize, Ordering};

use tracing::dispatcher::SetGlobalDefaultError;
use tracing::field::{Field, Visit};
use tracing::{Level, Subscriber};
use tracing_subscriber::layer::*;
use tracing_subscriber::registry::*;
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};
use crate::git_hash_version;

#[wasm_bindgen(js_name = LogLevel)]
pub struct JsLogLevel {
    inner: Level,
}

impl From<Level> for JsLogLevel {
    fn from(inner: Level) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = LogLevel)]
impl JsLogLevel {
    pub fn trace() -> Self {
        Level::TRACE.into()
    }

    pub fn debug() -> Self {
        Level::DEBUG.into()
    }

    pub fn info() -> Self {
        Level::INFO.into()
    }

    pub fn warn() -> Self {
        Level::WARN.into()
    }

    pub fn error() -> Self {
        Level::ERROR.into()
    }
}

#[wasm_bindgen(js_name = initLogger)]
pub fn init_logger(level: JsLogLevel) -> Result<()> {
    let level: Level = level.inner;
    let config = WASMLayerConfigBuilder::default()
        .set_max_level(level)
        .build();
    try_set_as_global_default(config).map_err(into_err)?;

    tracing::info!("Wasm logger initialized");

    // Log git hash (defined at compile time)
    match git_hash_version() {
        Some(hash) => tracing::info!("Git hash: {hash}"),
        None => tracing::warn!("Git hash not defined!"),
    };

    Ok(())
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = performance)]
    fn mark(a: &str);
    #[wasm_bindgen(catch, js_namespace = performance)]
    fn measure(name: String, startMark: String) -> Result<(), JsValue>;
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log1(message: String);
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log2(message1: &str, message2: &str);
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log3(message1: &str, message2: &str, message3: &str);
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log4(message1: String, message2: &str, message3: &str, message4: &str);
}

pub enum ConsoleConfig {
    NoReporting,
    ReportWithoutConsoleColor,
    ReportWithConsoleColor,
}

pub struct WASMLayerConfigBuilder {
    /// Log events will be marked and measured so they appear in performance Timings
    report_logs_in_timings: bool,
    /// Log events will be logged to the browser console
    report_logs_in_console: bool,
    /// Only relevant if report_logs_in_console is true, this will use color style strings in the console.
    use_console_color: bool,
    /// Log events will be reported from this level -- Default is ALL (TRACE)
    max_level: Level,
}

impl WASMLayerConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether events should appear in performance Timings
    pub fn set_report_logs_in_timings(&mut self, report_logs_in_timings: bool) -> &mut Self {
        self.report_logs_in_timings = report_logs_in_timings;
        self
    }

    /// Set the maximal level on which events should be displayed
    pub fn set_max_level(&mut self, max_level: Level) -> &mut Self {
        self.max_level = max_level;
        self
    }

    /// Set if and how events should be displayed in the browser console
    pub fn set_console_config(&mut self, console_config: ConsoleConfig) -> &mut Self {
        match console_config {
            ConsoleConfig::NoReporting => {
                self.report_logs_in_console = false;
                self.use_console_color = false;
            }
            ConsoleConfig::ReportWithoutConsoleColor => {
                self.report_logs_in_console = true;
                self.use_console_color = false;
            }
            ConsoleConfig::ReportWithConsoleColor => {
                self.report_logs_in_console = true;
                self.use_console_color = true;
            }
        }

        self
    }

    /// Build the WASMLayerConfig
    pub fn build(&self) -> WASMLayerConfig {
        WASMLayerConfig {
            report_logs_in_timings: self.report_logs_in_timings,
            report_logs_in_console: self.report_logs_in_console,
            use_console_color: self.use_console_color,
            max_level: self.max_level,
        }
    }
}

impl Default for WASMLayerConfigBuilder {
    fn default() -> Self {
        Self {
            report_logs_in_timings: true,
            report_logs_in_console: true,
            use_console_color: true,
            max_level: Level::TRACE,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct WASMLayerConfig {
    report_logs_in_timings: bool,
    report_logs_in_console: bool,
    use_console_color: bool,
    max_level: Level,
}

impl Default for WASMLayerConfig {
    fn default() -> Self {
        Self {
            report_logs_in_timings: true,
            report_logs_in_console: true,
            use_console_color: true,
            max_level: Level::TRACE,
        }
    }
}

/// Implements [tracing_subscriber::layer::Layer] which uses [wasm_bindgen] for marking and measuring with `window.performance`
pub struct WASMLayer {
    last_event_id: AtomicUsize,
    config: WASMLayerConfig,
}

impl WASMLayer {
    pub fn new(config: WASMLayerConfig) -> Self {
        Self {
            last_event_id: AtomicUsize::new(0),
            config,
        }
    }
}

impl Default for WASMLayer {
    fn default() -> Self {
        Self::new(WASMLayerConfig::default())
    }
}

#[inline]
fn thread_display_suffix() -> &'static str {
    ""
}

fn mark_name(id: &tracing::Id) -> String {
    format!("t{:x}", id.into_u64())
}

impl<S: Subscriber + for<'a> LookupSpan<'a>> Layer<S> for WASMLayer {
    fn enabled(&self, metadata: &tracing::Metadata<'_>, _: Context<'_, S>) -> bool {
        let level = metadata.level();
        level <= &self.config.max_level
    }

    fn on_new_span(
        &self,
        attrs: &tracing::span::Attributes<'_>,
        id: &tracing::Id,
        ctx: Context<'_, S>,
    ) {
        let mut new_debug_record = StringRecorder::new();
        attrs.record(&mut new_debug_record);

        if let Some(span_ref) = ctx.span(id) {
            span_ref
                .extensions_mut()
                .insert::<StringRecorder>(new_debug_record);
        }
    }

    /// doc: Notifies this layer that a span with the given Id recorded the given values.
    fn on_record(&self, id: &tracing::Id, values: &tracing::span::Record<'_>, ctx: Context<'_, S>) {
        if let Some(span_ref) = ctx.span(id) {
            if let Some(debug_record) = span_ref.extensions_mut().get_mut::<StringRecorder>() {
                values.record(debug_record);
            }
        }
    }

    // /// doc: Notifies this layer that a span with the ID span recorded that it follows from the span with the ID follows.
    // fn on_follows_from(&self, _span: &tracing::Id, _follows: &tracing::Id, ctx: Context<'_, S>) {}
    /// doc: Notifies this layer that an event has occurred.
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        if self.config.report_logs_in_timings || self.config.report_logs_in_console {
            let mut recorder = StringRecorder::new();
            event.record(&mut recorder);
            let meta = event.metadata();
            let level = meta.level();
            if self.config.report_logs_in_console {
                let module = meta
                    .module_path()
                    .and_then(|file| meta.line().map(|ln| format!("{}:{}", file, ln)))
                    .unwrap_or_default();

                if self.config.use_console_color {
                    log4(
                        format!(
                            "%c{}%c {}{}%c{}",
                            level,
                            module,
                            thread_display_suffix(),
                            recorder,
                        ),
                        match *level {
                            tracing::Level::TRACE => "color: dodgerblue; background: #444",
                            tracing::Level::DEBUG => "color: lawngreen; background: #444",
                            tracing::Level::INFO => "color: whitesmoke; background: #444",
                            tracing::Level::WARN => "color: orange; background: #444",
                            tracing::Level::ERROR => "color: red; background: #444",
                        },
                        "color: gray; font-style: italic",
                        "color: inherit",
                    );
                } else {
                    log1(format!(
                        "{} {}{} {}",
                        level,
                        module,
                        thread_display_suffix(),
                        recorder,
                    ));
                }
            }
            if self.config.report_logs_in_timings {
                let mark_name =
                    format!("c{:x}", self.last_event_id.fetch_add(1, Ordering::Relaxed));
                // mark and measure so you can see a little blip in the profile
                mark(&mark_name);
                let _ = measure(
                    format!(
                        "{} {}{} {}",
                        level,
                        meta.module_path().unwrap_or("..."),
                        thread_display_suffix(),
                        recorder,
                    ),
                    mark_name,
                );
            }
        }
    }

    /// doc: Notifies this layer that a span with the given ID was entered.
    fn on_enter(&self, id: &tracing::Id, _ctx: Context<'_, S>) {
        mark(&mark_name(id));
    }

    /// doc: Notifies this layer that the span with the given ID was exited.
    fn on_exit(&self, id: &tracing::Id, ctx: Context<'_, S>) {
        if let Some(span_ref) = ctx.span(id) {
            let meta = span_ref.metadata();
            if let Some(debug_record) = span_ref.extensions().get::<StringRecorder>() {
                let _ = measure(
                    format!(
                        "\"{}\"{} {} {}",
                        meta.name(),
                        thread_display_suffix(),
                        meta.module_path().unwrap_or("..."),
                        debug_record,
                    ),
                    mark_name(id),
                );
            } else {
                let _ = measure(
                    format!(
                        "\"{}\"{} {}",
                        meta.name(),
                        thread_display_suffix(),
                        meta.module_path().unwrap_or("..."),
                    ),
                    mark_name(id),
                );
            }
        }
    }
}

/// Set the global default with [tracing::subscriber::set_global_default]
fn try_set_as_global_default(config: WASMLayerConfig) -> Result<(), SetGlobalDefaultError> {
    tracing::subscriber::set_global_default(Registry::default().with(WASMLayer::new(config)))
}

struct StringRecorder {
    display: String,
    is_following_args: bool,
}

impl StringRecorder {
    fn new() -> Self {
        Self {
            display: String::new(),
            is_following_args: false,
        }
    }
}

impl Visit for StringRecorder {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if field.name() == "message" {
            if !self.display.is_empty() {
                self.display = format!("{:?}\n{}", value, self.display)
            } else {
                self.display = format!("{:?}", value)
            }
        } else {
            if self.is_following_args {
                // following args
                writeln!(self.display).unwrap();
            } else {
                // first arg
                write!(self.display, " ").unwrap();
                self.is_following_args = true;
            }
            write!(self.display, "{} = {:?};", field.name(), value).unwrap();
        }
    }
}

impl fmt::Display for StringRecorder {
    fn fmt(&self, mut f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.display.is_empty() {
            write!(&mut f, " {}", self.display)
        } else {
            Ok(())
        }
    }
}

impl Default for StringRecorder {
    fn default() -> Self {
        Self::new()
    }
}
