use super::wifi::Wifi;
use anyhow::Context;
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::process::Command;

pub struct Scanner;

#[cfg(target_os = "linux")]
impl Scanner {
    pub fn scan() -> anyhow::Result<Vec<Wifi>> {
        // TODO: Find the correct interface
        let child_stdout = Command::new("iwlist")
            .args([&Self::get_interface()?, "scanning"])
            .output()
            .map(|out| String::from_utf8(out.stdout))
            .with_context(|| anyhow::anyhow!("No output from \"iwlist scanning\""))??;

        let mut clients = child_stdout.split("Cell ");
        // skip first client since it's trash
        clients.next();

        // sudo iwlist wlp3s0 scanning
        // need to use sudo because of the iwlist command (else it only shows the current connection)
        // need to figure out how to get the right interface
        // scan with iwlist accesspoints
        Ok(clients.par_bridge().flat_map(str::parse).collect())
    }

    fn get_interface() -> anyhow::Result<String> {
        let iw_out = Command::new("iw")
            .arg("dev")
            .output()
            .with_context(|| anyhow::anyhow!("The command `iw` was not found."))?
            .stdout;

        Ok(String::from_utf8_lossy(&iw_out)
            .split("Interface ")
            .take(2)
            .last()
            .and_then(|tok| tok.split('\n').next())
            .with_context(|| anyhow::anyhow!("No interface found."))?
            .to_string())
    }
}

#[cfg(target_os = "windows")]
impl Scanner {
    pub fn scan() -> anyhow::Result<Vec<Wifi>> {
        use super::wifi::into_wifi_vec;

        let child_out = Command::new("netsh")
            .args(["wlan", "show", "network", "mode=BSSID"])
            .output()
            .map(|out| String::from_utf8(out.stdout))
            .with_context(|| anyhow::anyhow!("No output from \"netsh\""))??;

        let mut ssids = child_out.split("SSID ");
        ssids.next();

        let wifi_vec = ssids
            .par_bridge()
            .flat_map(into_wifi_vec)
            .flatten()
            .collect::<Vec<_>>();

        Ok(wifi_vec)
    }
}

#[cfg(target_os = "macos")]
impl Scanner {
    pub fn scan() -> anyhow::Result<Vec<Wifi>> {
        compile_error!("MacOS support not yet implemented. Give the dev a coffee.")
    }
}
