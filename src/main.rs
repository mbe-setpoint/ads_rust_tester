use ads::Handle;
use clap::Parser;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs;
use std::str::FromStr;
use std::{thread, time};

#[derive(Debug, Serialize, Deserialize)]
struct AdsSymbols {
    read_symbols: Vec<String>,
    write_symbols: Vec<String>,
}

#[derive(Parser)]
struct Cli {
    #[arg(long = "target_net_id")]
    target_net_id: String,
    #[arg(long = "target_ip")]
    target_ip: String,
    #[arg(long = "delay")]
    delay_ms: u64,
    #[arg(long = "ads_file")]
    ads_file: String,
}

fn main() -> ads::Result<()> {
    let args = Cli::parse();
    let ads_symbols: AdsSymbols = read_json(args.ads_file);

    // Open a connection to an ADS device identified by hostname/IP and port.
    // For TwinCAT devices, a route must be set to allow the client to connect.
    // The source AMS address is automatically generated from the local IP,
    // but can be explicitly specified as the third argument.
    let client = ads::Client::new(
        (args.target_ip, ads::PORT),
        ads::Timeouts::none(),
        ads::Source::Auto,
    )?;

    // Specify the target ADS device to talk to, by NetID and AMS port.
    // Port 851 usually refers to the first PLC instance.
    let ams_addr: String = args.target_net_id + ":851";
    let net_id: ads::AmsAddr = ads::AmsAddr::from_str(&ams_addr).unwrap();
    let device = client.device(net_id);
    // let device = client.device(ads::AmsAddr::new([5, 32, 116, 5, 1, 1].into(), 851));

    // Ensure that the PLC instance is running.
    assert!(
        device.get_state()?.0 == ads::AdsState::Run,
        "PLC not in run!"
    );

    loop {
        for symbol in ads_symbols.read_symbols.iter() {
            // Request a handle to a named symbol in the PLC instance.
            let handle = Handle::new(device, symbol)?;
            // Read data in form of an u32 from the handle.
            let value: u32 = handle.read_value()?;
            println!("{symbol} value is {value}");
        }

        for write_symbol in ads_symbols.write_symbols.iter() {
            let write_handle = Handle::new(device, write_symbol)?;
            let write_value = rand::thread_rng().gen_range(0..=100);
            let _nbr_written = write_handle.write_value(&write_value).expect("failed");
            println!("Wrote {write_value} to {write_symbol}");
        }

        println!("Sleeping {} ms", args.delay_ms);
        thread::sleep(time::Duration::from_millis(args.delay_ms));
    }

    // Connection will be closed when the client is dropped.
    // Ok(())
}

fn read_json(path: String) -> AdsSymbols {
    let data = fs::read_to_string(path).expect("Unable to read file");
    let obj: AdsSymbols = serde_json::from_str(&data).expect("Unable to parse");
    // println!("{:?}", obj);
    return obj;
}
