use anyhow::Context;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config_path =
        std::env::var("CONFIG_PATH").context("Failed to read CONFIG_PATH environment variable")?;
    let config_path = config_path.split(':').rev();

    match std::env::args().nth(1).as_deref() {
        Some("setup") => aocbot::setup(config_path).await,
        Some("run") => aocbot::run(config_path).await,
        _ => {
            eprintln!("usage: aocbot [setup|run]");
            std::process::exit(1)
        }
    }
}
