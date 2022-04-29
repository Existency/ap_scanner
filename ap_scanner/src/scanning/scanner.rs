use super::wifi::Wifi;
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::process::Command;

pub struct Scanner;

#[cfg(target_os = "linux")]
impl Scanner {
    pub fn scan() -> anyhow::Result<Vec<Wifi>> {
        // TODO: Find the correct interface

        let interface = Self::get_interface()?;

        let child = Command::new("iwlist")
            .args([&interface, "scanning"])
            .output()?;

        let cls = String::from_utf8(child.stdout)?;

        let mut clients = cls.split("Cell ");
        // skip first client since it's trash
        clients.next();

        // sudo iwlist wlp3s0 scanning
        // need to use sudo because of the iwlist command (else it only shows the current connection)
        // need to figure out how to get the right interface
        // scan with iwlist accesspoints

        Ok(clients
            .par_bridge()
            .filter_map(|x| Wifi::try_from(x).ok())
            .collect::<Vec<_>>())
    }

    fn get_interface() -> anyhow::Result<String> {
        let iw_out = Command::new("iw")
            .arg("dev")
            .output()
            .map_err(|_| anyhow::anyhow!("The command `iw` was not found."))?
            .stdout;

        Ok(String::from_utf8_lossy(&iw_out)
            .split("Interface ")
            .take(2)
            .last()
            .ok_or_else(|| anyhow::anyhow!("No interface found."))?
            .split('\n')
            .next()
            .ok_or_else(|| anyhow::anyhow!("No interface found."))?
            .to_string())
    }
}

#[cfg(target_os = "windows")]
impl Scanner {
    pub fn scan() -> anyhow::Result<Vec<Wifi>> {
        // netsh wlan show all (limpar lixo)
        compile_error!("Windows support not yet implemented. Give the dev a coffee.")
    }
}

#[cfg(target_os = "macos")]
impl Scanner {
    pub fn scan() -> anyhow::Result<Vec<Wifi>> {
        compile_error!("MacOS support not yet implemented. Give the dev a coffee.")
    }
}
