use prelude::*;

mod cloudflare;
mod config;

#[tokio::main]
async fn main() {
    env_logger::init();

    log::info!("Starting uploader");
    if let Err(e) = run().await {
        log::error!("Service failed:\n\n{:?}", e);
    } else {
        log::info!("Service successfully stopped");
    }
}

async fn run() -> Result<()> {
    dotenv::dotenv().ok();
    let config = config::Config::new()?;

    let client = cloudflare::stream::ClientBuilder::new()
        .token(config.token)
        .account_id(config.account_id)
        .build().await?;

    for url in std::env::args() {
        let res = client.upload_video(cloudflare::stream::VideoRequest{
            url,
            meta: cloudflare::stream::VideoMeta{
                name: "Test video".into(),
            }
        }).await.context("Failed to upload video")?;
        log::info!("Uploaded video {}, preview: {}", res.uid, res.preview);
    }

    Ok(())
}

