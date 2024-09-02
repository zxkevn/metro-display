use rpi_led_matrix::{LedFont, LedMatrix, LedColor, LedMatrixOptions, LedRuntimeOptions};
use std::path::Path;
use std::thread;
use std::time::Duration;
use tracing::{info, warn, error, debug};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::fmt::format::FmtSpan;

fn main() {
    // File appender for JSON logs
    let file_appender = tracing_appender::rolling::daily("logs", "app.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // JSON formatting layer for file output
    let bunyan_formatting_layer = BunyanFormattingLayer::new("app_name".into(), non_blocking);

    // Stdout formatting layer
    let stdout_log = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_span_events(FmtSpan::CLOSE);

    // Compose the tracing subscriber with both layers
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(JsonStorageLayer)
        .with(bunyan_formatting_layer)
        .with(stdout_log)
        .init();
    
    info!("test 1 2 3");
    warn!("omg warning");
    error!("errror!!!!");

    let mut matrix_options = LedMatrixOptions::new();
    matrix_options.set_hardware_mapping("adafruit-hat");
    matrix_options.set_rows(32);
    matrix_options.set_chain_length(4);
    matrix_options.set_pwm_bits(3);
    matrix_options.set_pwm_lsb_nanoseconds(300);

    let mut runtime_options = LedRuntimeOptions::new();
    runtime_options.set_gpio_slowdown(2);

    let matrix = LedMatrix::new(Some(matrix_options), Some(runtime_options)).unwrap();
    let mut canvas = matrix.offscreen_canvas();
    let font = LedFont::new(Path::new("/home/metro/metro-sign/rpi-metro-display/6x10.bdf")).unwrap();

    for i in 0..255 {
        canvas.clear();
        canvas.draw_text(&font, "TEST 1 2 3 4 5", 0, 7, &blue, 0, false);
        canvas = matrix.swap(canvas);
        thread::sleep(Duration::from_secs(1));
    }
}