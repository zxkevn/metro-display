use clap::Parser;
use rpi_led_matrix::{LedFont, LedMatrix, LedColor, LedMatrixOptions, LedRuntimeOptions};
use reqwest;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;
use tracing::{info, warn, error, debug};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::fmt::format::FmtSpan;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Args {
    #[arg(short, long, value_name="FILE", default_value="config.yaml")]
    config: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    apikey: String,
}

fn test_led_display() {
    let mut matrix_options = LedMatrixOptions::new();
    matrix_options.set_hardware_mapping("adafruit-hat");
    matrix_options.set_rows(32);
    matrix_options.set_chain_length(4);
    let _ = matrix_options.set_pwm_bits(3);
    matrix_options.set_pwm_lsb_nanoseconds(300);

    let mut runtime_options = LedRuntimeOptions::new();
    runtime_options.set_gpio_slowdown(2);

    let matrix = LedMatrix::new(Some(matrix_options), Some(runtime_options)).unwrap();
    let mut canvas = matrix.offscreen_canvas();
    let font = LedFont::new(Path::new("/home/metro/metro-sign/rpi-metro-display/6x10.bdf")).unwrap();

    // Define the blue color
    let blue = LedColor { red: 0, green: 0, blue: 255 };

    for _i in 0..255 {
        canvas.clear();
        canvas.draw_text(&font, "TEST 1 2 3 4 5", 0, 7, &blue, 2, false);
        canvas = matrix.swap(canvas);
        thread::sleep(Duration::from_secs(1));
    }
}

fn load_config(path: &str) -> Result<Config, Box<dyn Error>> {
    let config_str = fs::read_to_string(path)?;
    let config: Config = serde_yaml::from_str(&config_str)?;
    Ok(config)
}

// utility function to get a json response from the wmata api
async fn get_json(apikey: &str, url: &str) -> Result<serde_json::Value, Box<dyn Error>> {
    let mut headers = HeaderMap::new();
    headers.insert("api_key", HeaderValue::from_str(apikey)?);
    headers.insert("Accept", HeaderValue::from_static("application/json"));

    let client = reqwest::Client::new();
    let resp = client.get(url)
        .headers(headers)
        .send()
        .await?;

    debug!("Response status: {}", resp.status());

    let json = resp.json().await?;
    Ok(json)
}

// pull the latest info on the metro lines and parse
async fn get_metro_lines(apikey: &str) -> Result<(), Box<dyn Error>> {
    let json = get_json(apikey, "https://api.wmata.com/Rail.svc/json/jLines").await?;

    println!("{}", serde_json::to_string_pretty(&json)?);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // set up logging:
    //  - log to file + stdout
    //  - json format

    // file appender for JSON logs
    let file_appender = tracing_appender::rolling::daily("logs", "app.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // JSON formatting layer for file output
    let bunyan_formatting_layer = BunyanFormattingLayer::new("app_name".into(), non_blocking);

    // stdout formatting layer
    let stdout_log = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_span_events(FmtSpan::CLOSE);

    // compose the tracing subscriber with both layers
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(JsonStorageLayer)
        .with(bunyan_formatting_layer)
        .with(stdout_log)
        .init();
    
    info!("test 1 2 3");
    warn!("omg warning");
    error!("errror!!!!");

    // parse args
    let args = Args::parse();
    
    // load config
    let config = load_config(&args.config)?;

    get_metro_lines(&config.apikey).await?;

    test_led_display();

    Ok(())
}