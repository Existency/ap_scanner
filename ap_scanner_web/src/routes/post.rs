use crate::{
    readings::{Reading, ReadingID},
    Cache,
};
use rocket::{
    data::ToByteUnit,
    tokio::{fs::File, io::AsyncWriteExt},
    Data,
};

/// Handles the incoming data, assigns it a unique named file and saves it to disk.
/// Only a valid Json will be accepted.
#[post("/", data = "<data>")]
pub async fn upload(data: Data<'_>) -> std::io::Result<String> {
    let id = ReadingID::new();
    let data = data.open(128.kibibytes()).into_string().await?;
    let reading: Reading = serde_json::from_str(data.as_str())?;

    println!(
        "Received a reading with {} total elements. Saving to \"{:?}\"",
        reading.wifi_2_4_ghz.len() + reading.wifi_5_ghz.len(),
        &id.path()
    );

    let mut file = File::create(id.path()).await?;
    file.write_all(data.as_str().as_bytes()).await?;

    Cache::insert_into_cache(reading, id.path().to_str().unwrap());

    Ok(uri!("http://0.0.0.0:9999", super::serve(id)).to_string())
}
