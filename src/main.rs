mod types;

use anyhow::{Result};
use discord_rpc_client::Client;
use crate::types::{TorrentInfo, TransferInfo};
use size;
use size::Size;

struct CurrentState {
    uploaded_lifetime: u64,
    downloaded_lifetime: u64,

    uploading: u64,
    downloading: u64,

    download_speed: u64,
    upload_speed: u64,

    session_downloaded: u64,
    session_uploaded: u64,

    image_asset: String
}

async fn gather_data(qbittorrent_api: &String) -> Result<CurrentState> {
    let torrents_response = reqwest::get(format!("{}/api/v2/torrents/info", qbittorrent_api))
        .await?
        .json::<Vec<TorrentInfo>>()
        .await?;

    let mut uploaded_lifetime = 0u64;
    let mut downloaded_lifetime = 0u64;

    let mut uploading = 0;
    let mut downloading = 0;

    for torrent in torrents_response {
        uploaded_lifetime += torrent.uploaded;
        downloaded_lifetime += torrent.downloaded;

        match torrent.state.as_str() {
            "uploading" | "pausedUP" | "stalledUP" | "forcedUP" => {
                uploading += 1;
            }
            "downloading" | "metaDL" | "pausedDL" | "queuedDL" | "stalledDL" => {
                downloading += 1;
            }
            _ => {}
        }
    }

    let speed_response = reqwest::get(format!("{}/api/v2/transfer/info", qbittorrent_api))
        .await?
        .json::<TransferInfo>()
        .await?;

    let image_asset = if speed_response.dl_info_speed == 0 {
        if speed_response.up_info_speed == 0 { "idle" } else { "upload" }
    } else { "download" };

    Ok(CurrentState {
        uploaded_lifetime,
        downloaded_lifetime,
        uploading,
        downloading,
        download_speed: speed_response.dl_info_speed,
        upload_speed: speed_response.up_info_speed,
        session_downloaded: speed_response.dl_info_data,
        session_uploaded: speed_response.up_info_data,
        image_asset: image_asset.into()
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut drpc = Client::new(1060498512091037756);
    let qbittorrent_url = std::env::var("QBITTORRENT_API_URL").unwrap_or("http://127.0.0.1".into());

    drpc.start();

    println!("Chooching!");
    loop {
        let data = gather_data(&qbittorrent_url).await?;
        let top_text = format!(
            "Up: {}/s | Down: {}/s",
            Size::from_bytes(data.upload_speed),
            Size::from_bytes(data.download_speed)
        );

        let bottom_text = format!(
            "{} Seeding, {} Downloading",
            data.uploading,
            data.downloading
        );

        let large_image_text = format!(
            "Lifetime: {} up, {} down",
            Size::from_bytes(data.uploaded_lifetime),
            Size::from_bytes(data.downloaded_lifetime)
        );

        let small_image_text = format!(
            "Session: {} up, {} down",
            Size::from_bytes(data.session_uploaded),
            Size::from_bytes(data.session_downloaded)
        );

        drpc.set_activity(
            |act| act
                .state(top_text)
                .details(bottom_text)
                .assets( |assets| assets
                    .large_image("app_logo")
                    .large_text(large_image_text)
                    .small_image(data.image_asset)
                    .small_text(small_image_text)
                )
        )?;

        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}
