// Daemon idea:
//  - Run in background as a service
//  - Periodically take new readings
//  - Upload them to the server

//  - Periodically check if the server has new advice
//  - Server will periodically send new advice
use super::reading::Reading;

pub fn daemon_service(local: String) {
    loop {
        std::thread::sleep(std::time::Duration::from_secs(300));

        // upload readings to server
        match Reading::new(local.to_string()) {
            Ok(reading) => upload_reading(reading),
            Err(_) => continue,
        }
    }
}

#[inline]
fn upload_reading(readings: Reading) {
    // use reqwest to upload reading to server
    // use serde_json to serialize reading
    let body = serde_json::to_string(&readings).unwrap();
    let blocking_client = reqwest::blocking::Client::new();
    match blocking_client
        .post("http://0.0.0.0:9999/")
        .body(body)
        .send()
    {
        Ok(_resp) => println!("Uploaded reading"),
        Err(_) => println!("Failed to upload reading"),
    };
}
