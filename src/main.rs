use std::cmp::min;
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};
use structopt::StructOpt;
use threadpool::ThreadPool;

use cli::ConnTestCfg;

mod cli;

fn main() {
    if let Err(err) = conntest() {
        eprintln!("ERROR in main: {}", &err);
        std::process::exit(11);
    }
}

fn tcp_conn(ip: String, cfg: Arc<ConnTestCfg>) {
    let ip_port = format!("{}:{}", ip,cfg.default_port);
    match tcp_conn_inner(&ip_port, cfg) {
        Ok(dur) => println!("{}  worked in {:.3} seconds", ip_port, dur.as_secs_f64()),
        Err(e) => println!("{}  failed due to: {}", ip_port, e),
    }
}

fn tcp_conn_inner(ip: &String, cfg: Arc<ConnTestCfg>) -> Result<Duration> {
    let start = Instant::now();
    let mut saddr = ip.to_socket_addrs()?.next().with_context(||"uh bad address not sure")?; // parse::<IpAddr>()?;
    if saddr.port() == 0 {
        saddr.set_port(cfg.default_port);
    }

    match TcpStream::connect_timeout(&saddr, cfg.timeout) {
        Err(err) => {
            match err.kind() {
                // shimmy the time out error a bit cause it's special
                std::io::ErrorKind::TimedOut => Err(anyhow!("Timeout after: {:.3} seconds", start.elapsed().as_secs_f64())),
                _ => Err(err)?,
            }
        },
        Ok(_) => Ok(start.elapsed()),
    }
}

fn conntest() -> Result<()> {
    let cfg = Arc::new(ConnTestCfg::from_args());

    let mut no_thr = if cfg.no_threads>0 {
        cfg.no_threads
    } else {
        cfg.ips.len()
    };

    if cfg.ips.len() >= 1024 && cfg.no_threads == 0 {
        eprintln!("You might consider setting no_threads to some limit with so many [{}] IPs", cfg.ips.len());
    }

    no_thr = min(256, no_thr);

    let pool = ThreadPool::new(no_thr);

    for ip in cfg.ips.iter() {
        let c_cfg = cfg.clone();
        let c_ip = ip.clone();
        pool.execute(move|| tcp_conn(c_ip, c_cfg));
    }

    pool.join();

    Ok(())
}


