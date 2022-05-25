#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

mod readings;
mod routes;
use routes::*;
mod cache;
use cache::Cache;

lazy_static::lazy_static! {
    // static cache: HashMap<(SSID,MAC), Filename> = HashMap::new();

    static ref HOST: String = "http://0.0.0.0:9999".into();

    static ref SCAN_PATH: String = "upload".into();
}

#[launch]
fn rocket() -> _ {
    // create the directory
    Cache::create_dir().expect("Could not create directory and/or cache file");

    Cache::from_file("upload/cache");

    // starts a thread that will save the cache every 30 seconds.
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(30));
        Cache::to_file("upload/cache").unwrap_or(());
    });

    rocket::build().mount(
        "/",
        routes![
            suggestion,
            suggestion_raw,
            upload,
            default_route,
            serve,
            index
        ],
    )
}
