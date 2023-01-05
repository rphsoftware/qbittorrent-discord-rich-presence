use serde::Deserialize;
// This data model isn't complete because it is not intended to be a broad
// qbittorrent api implementation. Rather, it only defines the needed values
// for this specific applicatoin to work.

#[derive(Debug, Deserialize)]
pub struct TransferInfo {
    pub dl_info_data: u64,
    pub dl_info_speed: u64,
    pub up_info_data: u64,
    pub up_info_speed: u64
}

#[derive(Debug, Deserialize, Clone)]
pub struct TorrentInfo {
    pub downloaded: u64,
    pub uploaded: u64,
    pub state: String
}
