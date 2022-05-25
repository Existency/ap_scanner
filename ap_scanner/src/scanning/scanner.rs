use anyhow::{Context, Result};
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::{process::Command, vec};

use super::wifi::Wifi;

pub struct Scanner;

impl Scanner {
    pub fn scan() -> Result<Vec<Wifi>> {
        // iw dev wlp3s0 scan
        let ch = Command::new("iw")
            .args(["dev", &Self::get_interface()?, "scan"])
            .output()
            .map(|out| String::from_utf8(out.stdout))
            .with_context(|| anyhow::anyhow!("No output from \"iw dev iface scan\""))??;

        let patterns = &["\tBSS", " BSS "];
        let replaces = &["\tbss", " bss "];

        let mut out = vec![];
        let ac = aho_corasick::AhoCorasick::new(patterns);
        ac.stream_replace_all(ch.as_bytes(), &mut out, replaces)?;

        let output = String::from_utf8(out)?;
        let mut clean = output.split("BSS ");

        clean.next();

        Ok(clean.par_bridge().flat_map(str::parse).collect::<Vec<_>>())
    }

    fn get_interface() -> Result<String> {
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
