use chrono::offset::Local as localtime;
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal,
};
use nvml_wrapper::{enum_wrappers::device::Brand, Device, Nvml};
// use std::env;
use std::io::{self, Write};
use std::thread::sleep;
use std::time::Duration;

fn main() -> io::Result<()> {
    // Raw mode required for getting the 'q' so we can quit
    terminal::enable_raw_mode()?;

    // parse our args into args
    let args = Args::parse();

    // This stuff needs to be in main so we don't re-init the device
    let nvml = Nvml::init();
    // Get the first `Device` (GPU) in the system
    let nvml_obj = nvml.expect("wtf");
    let nv_dev = nvml_obj.device_by_index(0).unwrap();

    // Start the loop if asked for
    if args.loopit {
        loop {
            println!("\x1B[2J\x1B[1;1H");

            if args.oneliner {
                print_oneline(&nv_dev);
            } else {
                print_multiliner(&nv_dev, args.loopit);
            }

            io::stdout().flush()?;
            if event::poll(Duration::from_millis(100))?
                && let Event::Key(key_event) = event::read()?
                && key_event.code == KeyCode::Char('q')
            {
                terminal::disable_raw_mode()?;
                break Ok(());
            }
            sleep(Duration::from_secs(args.freq));
        }
    } else {
        if args.oneliner {
            print_oneline(&nv_dev);
        } else {
            print_multiliner(&nv_dev, args.loopit);
        }
        terminal::disable_raw_mode()?;
        Ok(())
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    // -l argument for running in a loop
    #[arg(short, long, default_value_t = false)]
    loopit: bool,

    // -f argument for loop frequency
    #[arg(short, long, default_value_t = 1)]
    freq: u64,

    // -o argument for printing single line!
    // This should just change the default which is multi line, verbose output
    #[arg(short, long, default_value_t = false)]
    oneliner: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Stats {
    pub brand: Brand,
    pub fan_speed: u32,
    pub gpu_temp: u32,
    pub pwr_used: u32,
    pub pwr_cap: u32,
    pub gpu_util: u32,
    pub enc_util: u32,
    pub dec_util: u32,
    pub mem_used: f32,
    pub mem_total: f32,
    pub drvr_ver: String,
}

impl Stats {
    pub fn new(device: &Device) -> Self {
        let brand = device.brand().unwrap(); // GeForce on my system
        let fan_speed = device.fan_speed(0).unwrap_or(0); // Currently 17% on my system
        let gpu_temp = device
            .temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
            .unwrap_or(0);
        let pwr_used = device.power_usage().unwrap_or(0) / 1000;
        let pwr_cap = device.power_management_limit_default().unwrap_or(0) / 1000;

        let gpu_util = device.utilization_rates().unwrap().gpu;
        let enc_util = device.encoder_utilization().unwrap().utilization;
        let dec_util = device.decoder_utilization().unwrap().utilization;

        // Get MEM specific outputs and print
        let memory_info = device.memory_info().unwrap();
        let mem_used = memory_info.used as f32 / 1_024_000_000.0;
        let mem_total = memory_info.total as f32 / 1_024_000_000.0;

        // get the driver memory_info
        let drvr_ver = device
            .nvml()
            .sys_driver_version()
            .unwrap_or("N/A".to_owned());
        Self {
            brand,
            fan_speed,
            gpu_temp,
            pwr_cap,
            pwr_used,
            gpu_util,
            enc_util,
            dec_util,
            mem_total,
            mem_used,
            drvr_ver,
        }
    }
}

fn print_oneline(device: &Device) {
    let stats = Stats::new(device);
    println!(
        "GPU: {:?} Enc: {:?} Dec: {:?} Tmp: {:?} Fan: {:?}\r",
        stats.gpu_util, stats.enc_util, stats.dec_util, stats.gpu_temp, stats.fan_speed
    );
}

fn print_multiliner(device: &Device, looping: bool) {
    let stats = Stats::new(device);
    // print local time of course only if in a loop
    if looping {
        println!("{}\r", localtime::now().format("%Y-%m-%d %H:%M:%S"));
    }

    // print the brand name
    println!("Brand: {:?}, Version: {}\r", stats.brand, stats.drvr_ver);

    // print the fan speed
    println!("Fan Speed: {:?}%\r", stats.fan_speed);

    // print the gpu temp
    println!("GPU Temp: {:?}C\r", stats.gpu_temp);

    // print the power used/cap
    println!(
        "Power Usage: Used:{:?}W, Max:{:?}W\r",
        stats.pwr_used, stats.pwr_cap
    );

    // print the mem used/total
    println!(
        "Memory Usage: Used:{:.2?}GB, Total:{:.2?}GB\r",
        stats.mem_used, stats.mem_total
    );

    // print the GPU usage
    // print without newline so as not to waste space...
    println!(
        "GPU Usage: {:?}%, Encoder: {:?}%, Decoder: {:?}%\r",
        stats.gpu_util, stats.enc_util, stats.dec_util
    );
}
