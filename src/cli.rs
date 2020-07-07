use std::time::Duration;

use anyhow::{anyhow, Result};
use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
global_settings(& [
structopt::clap::AppSettings::ColoredHelp,
//structopt::clap::AppSettings::UnifiedHelpMessage
]),
)]
/// Test TCP connectivity to many IPs/ports fast
pub struct ConnTestCfg {
    #[structopt(required(true))]
    /// IP list
    pub ips: Vec<String>,

    #[structopt(short="p", long, default_value("22"))]
    /// TCP port
    pub default_port: u16,

    #[structopt(short, long,parse(try_from_str = dur_from_str), default_value("5s"))]
    /// Timeout - in ms by default, but also 1m = 1 min; 1h25ms = 1 hour + 24 millis
    pub timeout: Duration,

    #[structopt(short, long, default_value("0"))]
    /// Simultaneous checks / thread, 0 means no limit
    pub no_threads: usize,

}

pub fn dur_from_str(s: &str) -> Result<Duration> {
    let mut num = String::with_capacity(5);
    let mut units = String::with_capacity(5);
    let mut tot_ms = 0u64;
    let mut in_char = false;
    for c in s.chars() {
        if c >= '0' && c <= '9' { // is digit
            if in_char {
                if num.len() > 0 {
                    tot_ms += dur_units(&mut num, &mut units)?;
                }
            }
            num.push(c);
            in_char = false;
        } else {
            in_char = true;
            units.push(c);
        }
    }
    if num.len() > 0 {
        tot_ms += dur_units(&mut num, &mut units)?;
    }
    Ok(Duration::from_millis(tot_ms))
}

fn dur_units(num: &mut String, unit: &mut String) -> Result<u64> {
    let mut tot_ms = 0;
    if num.len() > 0 {
        tot_ms += match unit.as_str() {
            "" | "ms" => num.parse::<u64>()?,
            "s" => num.parse::<u64>()? * 1000,
            "m" => num.parse::<u64>()? * 1000 * 60,
            "h" => num.parse::<u64>()? * 1000 * 3600,
            "d" => num.parse::<u64>()? * 1000 * 24 * 3600,
            "w" => num.parse::<u64>()? * 1000 * 24 * 3600 * 7,
            "y" => num.parse::<u64>()? * 1000 * 24 * 3600 * 365,
            _ => Err(anyhow!("unit {} not understood", &unit))?,
        };
    }
    num.clear();
    unit.clear();

    Ok(tot_ms)
}

