use chrono::offset::Local as localtime;
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal,
};
use nvml_wrapper::{enum_wrappers::device::Brand, Device, Nvml};
use owo_colors::OwoColorize;
use owo_colors::Stream::Stdout;
use std::io::{self, Write};
use std::thread::sleep;
use std::time::Duration;

fn main() -> io::Result<()> {
    // parse our args into args

    let args = Args::parse();

    // This stuff needs to be in main so we don't re-init the device
    let nvml = Nvml::init();
    // Get the first `Device` (GPU) in the system
    let nvml_obj = nvml.expect("wtf");
    let nv_dev = nvml_obj.device_by_index(0).unwrap();

    // Start the loop if asked for
    if args.loopit {
        // Raw mode required for getting the 'q' so we can quit
        terminal::enable_raw_mode()?;

        loop {
            println!("\x1B[2J\x1B[1;1H");

            if args.oneliner {
                print_oneline(&nv_dev);
            } else {
                print_multiliner(&nv_dev, args.loopit, args.colorize);
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
            print_multiliner(&nv_dev, args.loopit, args.colorize);
        }
        terminal::disable_raw_mode()?;
        Ok(())
    }
}

#[derive(Parser, Debug)]
#[command(version, about)]
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

    // -c argument for colorizing
    #[arg(short, long, default_value_t = false)]
    colorize: bool,
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
    pub cuda_ver: f32,
    pub dev_name: String,
}

impl Stats {
    pub fn new(device: &Device) -> Self {
        let brand = device.brand().unwrap(); // GeForce on my system
        let dev_name = device.name().unwrap_or("unknown".into());
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

        // get the driver memory_info
        let cuda_ver = device.nvml().sys_cuda_driver_version().unwrap_or(0) as f32 / 1000.0;
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
            cuda_ver,
            dev_name,
        }
    }
}

fn print_oneline(device: &Device) {
    let stats = Stats::new(device);
    println!(
        "{:?} Enc: {:?} Dec: {:?} Tmp: {:?} Fan: {:?}\r",
        stats.gpu_util, stats.enc_util, stats.dec_util, stats.gpu_temp, stats.fan_speed
    );
}

fn print_multiliner(device: &Device, looping: bool, colorize: bool) {
    // disable colors if flag set
    // manually setting here since we don't have the option for this
    owo_colors::set_override(colorize);

    let stats = Stats::new(device);
    // print local time of course only if in a loop
    if looping {
        println!(
            "{}\r",
            localtime::now()
                .format("%Y-%m-%d %H:%M:%S")
                .if_supports_color(Stdout, |textual| textual.yellow())
        );
    }

    // print the brand/name
    println!(
        "{} {}\r",
        "GPU: ".if_supports_color(Stdout, |gpu| gpu.red()),
        stats.dev_name.if_supports_color(Stdout, |name| name.cyan())
    );

    // print the driver info
    println!(
        "{} {} {} {:.1}\r",
        "Driver Ver:".if_supports_color(Stdout, |ver| ver.red()),
        stats.drvr_ver.if_supports_color(Stdout, |drvr| drvr.cyan()),
        "CUDA Ver:".if_supports_color(Stdout, |ver| ver.red()),
        stats.cuda_ver.if_supports_color(Stdout, |drvr| drvr.cyan()),
    );

    // print the fan speed
    println!(
        "{} {:?}\r",
        "Fan Speed: ".if_supports_color(Stdout, |clr| clr.red()),
        stats.fan_speed.if_supports_color(Stdout, |clr| clr.cyan())
    );

    // print the gpu temp
    println!(
        "{} {:?}c\r",
        "GPU Temp:".if_supports_color(Stdout, |clr| clr.red()),
        stats.gpu_temp.if_supports_color(Stdout, |clr| clr.cyan())
    );

    // print the power used/cap
    println!(
        "{} {}{:?}, {}{:?}\r",
        "Power Usageg:".if_supports_color(Stdout, |clr| clr.red()),
        "Used:".if_supports_color(Stdout, |clr| clr.cyan()),
        stats.pwr_used.if_supports_color(Stdout, |clr| clr.yellow()),
        "Max:".if_supports_color(Stdout, |clr| clr.cyan()),
        stats.pwr_cap.if_supports_color(Stdout, |clr| clr.yellow()),
    );

    // print the mem used/total
    println!(
        "{} {}{:.2?}, {}{:.2?}\r",
        "Memory Usage:".if_supports_color(Stdout, |clr| clr.red()),
        "Used:".if_supports_color(Stdout, |clr| clr.cyan()),
        stats.mem_used.if_supports_color(Stdout, |clr| clr.yellow()),
        "Max:".if_supports_color(Stdout, |clr| clr.cyan()),
        stats
            .mem_total
            .if_supports_color(Stdout, |clr| clr.yellow()),
    );

    // print the GPU usage
    // print without newline so as not to waste space...
    println!(
        "{} {:?}% {} {:?}% {} {:?}%\r",
        "GPU Usage:".if_supports_color(Stdout, |clr| clr.red()),
        stats.gpu_util.if_supports_color(Stdout, |clr| clr.cyan()),
        "Encoder:".if_supports_color(Stdout, |clr| clr.red()),
        stats.enc_util.if_supports_color(Stdout, |clr| clr.cyan()),
        "Decoder:".if_supports_color(Stdout, |clr| clr.red()),
        stats.dec_util.if_supports_color(Stdout, |clr| clr.cyan())
    );
}
