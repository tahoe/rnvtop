use chrono::offset::Local as localtime;
use clap::Parser;
use colored_json::to_colored_json_auto;
use nvml_wrapper::{Device, Nvml};
use owo_colors::OwoColorize;
use owo_colors::Stream::Stdout;
use serde::Serialize;
use std::fmt;
use std::io;
use std::thread::sleep;
use std::time::Duration;
use tabled::settings::Settings;
use tabled::settings::Style;
use tabled::{
    settings::{object::Rows, Color, Concat},
    Table, Tabled,
};

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
        loop {
            print!("\x1B[2J\x1B[1;1H");

            if args.tabular {
                print_tabular(&nv_dev, args.colorize);
            } else if args.json {
                print_json(&nv_dev);
            } else {
                print_multiliner(&nv_dev, args.loopit, args.colorize);
            }
            sleep(Duration::from_secs(args.freq));
        }
    } else {
        if args.json {
            print_json(&nv_dev);
        } else if args.tabular {
            print_tabular(&nv_dev, args.colorize);
        } else {
            print_multiliner(&nv_dev, args.loopit, args.colorize);
        }
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

    // -c argument for colorizing
    #[arg(short, long, default_value_t = false)]
    colorize: bool,

    // -j argument for json output
    #[arg(short, long, conflicts_with = "tabular", default_value_t = false)]
    json: bool,

    // -t argument for tabular output
    #[arg(short, long, conflicts_with = "json", default_value_t = false)]
    tabular: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Tabled)]
pub struct GpuStats {
    #[tabled(rename = "GPU Util")]
    pub gpu_util: u32,
    #[tabled(rename = "Enc Util")]
    pub enc_util: u32,
    #[tabled(rename = "Dec Util")]
    pub dec_util: u32,
}

impl fmt::Display for GpuStats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "gpu_util: {}, enc_util: {}, dec_util: {}",
            self.gpu_util, self.enc_util, self.dec_util
        )
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Tabled)]
pub struct FanTemp {
    #[tabled(rename = "Fan Speed")]
    pub fan_speed: u32,
    #[tabled(rename = "GPU Temp")]
    pub gpu_temp: u32,
}

impl fmt::Display for FanTemp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "fan_speed: {}, gpu_temp: {}",
            self.fan_speed, self.gpu_temp
        )
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Tabled)]
pub struct Power {
    #[tabled(rename = "PWR Used")]
    pub pwr_used: u32,
    #[tabled(rename = "PWR Max")]
    pub pwr_cap: u32,
}

impl fmt::Display for Power {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "pwr_used: {}, pwr_cap: {}", self.pwr_used, self.pwr_cap)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Tabled)]
pub struct Memory {
    #[tabled(rename = "Memory Used")]
    pub mem_used: f32,
    #[tabled(rename = "Memory Total")]
    pub mem_total: f32,
}

impl fmt::Display for Memory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "mem_used: {}, mem_total: {}",
            self.mem_used, self.mem_total
        )
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Tabled)]
pub struct DeviceInfo {
    #[tabled(rename = "Driver Ver")]
    pub drvr_ver: String,
    #[tabled(rename = "Cuda Ver")]
    pub cuda_ver: f32,
    #[tabled(rename = "Device Name")]
    pub dev_name: String,
}

impl fmt::Display for DeviceInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "drvr_ver: {}, cuda_ver: {}, dev_name: {}",
            self.drvr_ver, self.cuda_ver, self.dev_name
        )
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Tabled)]
pub struct Stats {
    // DeviceInfo sub struct
    #[tabled(rename = "DeviceInfo")]
    pub devinfo: DeviceInfo,

    // Memory sub struct
    #[tabled(rename = "Memory")]
    pub memory: Memory,

    // GPU Stats sub struct
    #[tabled(rename = "GPU Stats")]
    pub gpustats: GpuStats,

    // Temp and Fan speed sub struct
    #[tabled(rename = "Fan/Temp")]
    pub fantemp: FanTemp,

    // Power sub struct
    #[tabled(rename = "Power")]
    pub power: Power,
}

impl Stats {
    pub fn new(device: &Device) -> Self {
        // get fan/temp info
        let fan_speed = device.fan_speed(0).unwrap_or(0); // Currently 17% on my system
        let gpu_temp = device
            .temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
            .unwrap_or(0);

        // populate FanTemp struct
        let fantemp = FanTemp {
            fan_speed,
            gpu_temp,
        };

        // get power info
        let pwr_used = device.power_usage().unwrap_or(0) / 1000;
        let pwr_cap = device.power_management_limit_default().unwrap_or(0) / 1000;

        // populate Power struct
        let power = Power { pwr_used, pwr_cap };

        // get utilization info
        let gpu_util = device.utilization_rates().unwrap().gpu;
        let enc_util = device.encoder_utilization().unwrap().utilization;
        let dec_util = device.decoder_utilization().unwrap().utilization;

        // populate GpuStats
        let gpustats = GpuStats {
            gpu_util,
            enc_util,
            dec_util,
        };

        // Get MEM specific outputs and print
        let memory_info = device.memory_info().unwrap();
        let mem_used = memory_info.used as f32 / 1_024_000_000.0;
        let mem_total = memory_info.total as f32 / 1_024_000_000.0;

        // populate Memory struct
        let memory = Memory {
            mem_used,
            mem_total,
        };

        // get the driver memory_info
        let dev_name = device.name().unwrap_or("unknown".into());
        let drvr_ver = device
            .nvml()
            .sys_driver_version()
            .unwrap_or("N/A".to_owned());
        let cuda_ver = device.nvml().sys_cuda_driver_version().unwrap_or(0) as f32 / 1000.0;

        // populate the DeviceInfo struct
        let devinfo = DeviceInfo {
            drvr_ver,
            cuda_ver,
            dev_name,
        };

        Self {
            devinfo,
            memory,
            power,
            fantemp,
            gpustats,
        }
    }
}

fn print_json(device: &Device) {
    let stats = Stats::new(device);
    let stats = to_colored_json_auto(&stats).expect("Fuck");
    println!("{}", stats);
}

fn print_tabular(device: &Device, colorize: bool) {
    //     devinfo,
    //     memory,
    //     power,
    //     fantemp,
    //     gpustats,
    let stats = Stats::new(device);
    let v_devinfo = vec![stats.devinfo];
    let v_memory = vec![stats.memory];
    let v_power = vec![stats.power];
    let v_fantemp = vec![stats.fantemp];
    let v_gpustats = vec![stats.gpustats];

    let mut devinfo_table = Table::new(v_devinfo);
    let memory_table = Table::new(v_memory);
    let power_table = Table::new(v_power);
    let fantemp_table = Table::new(v_fantemp);
    let gpustats_table = Table::new(v_gpustats);

    devinfo_table.with(Concat::vertical(memory_table));
    devinfo_table.with(Concat::vertical(power_table));
    devinfo_table.with(Concat::vertical(fantemp_table));
    devinfo_table.with(Concat::vertical(gpustats_table));

    // this finally lets us have BOLD colors
    let header_options = Settings::empty().with(Color::FG_BRIGHT_GREEN | Color::BOLD);
    if colorize {
        devinfo_table.modify(Rows::one(0), header_options.clone());
        devinfo_table.modify(Rows::one(1), Color::FG_BRIGHT_CYAN);
        devinfo_table.modify(Rows::one(2), header_options.clone());
        devinfo_table.modify(Rows::one(3), Color::FG_BRIGHT_CYAN);
        devinfo_table.modify(Rows::one(4), header_options.clone());
        devinfo_table.modify(Rows::one(5), Color::FG_BRIGHT_CYAN);
        devinfo_table.modify(Rows::one(6), header_options.clone());
        devinfo_table.modify(Rows::one(7), Color::FG_BRIGHT_CYAN);
        devinfo_table.modify(Rows::one(8), Color::FG_BRIGHT_GREEN);
        devinfo_table.modify(Rows::one(9), Color::FG_BRIGHT_CYAN);
    }
    devinfo_table.with(Style::modern_rounded());

    println!("{}", devinfo_table);
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
        "GPU:".if_supports_color(Stdout, |gpu| gpu.red()),
        stats
            .devinfo
            .dev_name
            .if_supports_color(Stdout, |name| name.cyan())
    );

    // print the driver info
    println!(
        "{} {} {} {:.1}\r",
        "Driver Ver:".if_supports_color(Stdout, |ver| ver.red()),
        stats
            .devinfo
            .drvr_ver
            .if_supports_color(Stdout, |drvr| drvr.cyan()),
        "CUDA Ver:".if_supports_color(Stdout, |ver| ver.red()),
        stats
            .devinfo
            .cuda_ver
            .if_supports_color(Stdout, |drvr| drvr.cyan()),
    );

    // print the fan speed
    println!(
        "{} {:?}\r",
        "Fan Speed: ".if_supports_color(Stdout, |clr| clr.red()),
        stats
            .fantemp
            .fan_speed
            .if_supports_color(Stdout, |clr| clr.cyan())
    );

    // print the gpu temp
    println!(
        "{} {:?}{}\r",
        "GPU Temp:".if_supports_color(Stdout, |clr| clr.red()),
        stats
            .fantemp
            .gpu_temp
            .if_supports_color(Stdout, |clr| clr.cyan()),
        "c".yellow()
    );

    // print the power used/cap
    println!(
        "{} {}{:?}, {}{:?}\r",
        "Power Usageg:".if_supports_color(Stdout, |clr| clr.red()),
        "Used:".if_supports_color(Stdout, |clr| clr.cyan()),
        stats
            .power
            .pwr_used
            .if_supports_color(Stdout, |clr| clr.yellow()),
        "Max:".if_supports_color(Stdout, |clr| clr.cyan()),
        stats
            .power
            .pwr_cap
            .if_supports_color(Stdout, |clr| clr.yellow()),
    );

    // print the mem used/total
    println!(
        "{} {}{:.2?}, {}{:.2?}\r",
        "Memory Usage:".if_supports_color(Stdout, |clr| clr.red()),
        "Used:".if_supports_color(Stdout, |clr| clr.cyan()),
        stats
            .memory
            .mem_used
            .if_supports_color(Stdout, |clr| clr.yellow()),
        "Max:".if_supports_color(Stdout, |clr| clr.cyan()),
        stats
            .memory
            .mem_total
            .if_supports_color(Stdout, |clr| clr.yellow()),
    );

    // print the GPU usage
    println!(
        "{} {:?}% {} {:?}% {} {:?}%\r",
        "GPU Usage:".if_supports_color(Stdout, |clr| clr.red()),
        stats
            .gpustats
            .gpu_util
            .if_supports_color(Stdout, |clr| clr.cyan()),
        "Encoder:".if_supports_color(Stdout, |clr| clr.red()),
        stats
            .gpustats
            .enc_util
            .if_supports_color(Stdout, |clr| clr.cyan()),
        "Decoder:".if_supports_color(Stdout, |clr| clr.red()),
        stats
            .gpustats
            .dec_util
            .if_supports_color(Stdout, |clr| clr.cyan())
    );
}
