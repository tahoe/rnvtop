use chrono::offset::Local as localtime;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal,
};
use nvml_wrapper::{enum_wrappers::device::Brand, Device, Nvml};
use std::env;
use std::io::{self, Write};
use std::thread::sleep;
use std::time::Duration;

fn main() -> io::Result<()> {
    // Raw mode required for getting the 'q' so we can quit
    terminal::enable_raw_mode()?;

    let args: Vec<String> = env::args().collect();

    // set two vars for controlling the app
    // one determines whether to loop or not
    // second one determines how frequent to restart loop
    let mut do_loop = false;
    let mut loop_secs: u64 = 1;

    // update the two vars above if the correct args are provided
    if args.len() > 1 && args[1] == "-l" {
        do_loop = true;
        // only if the first one is true, do we look for the second var
        if args.len() > 2 && args[2].chars().all(|c| c.is_ascii_digit()) {
            loop_secs = args[2].parse::<u64>().unwrap_or(1);
        }
    }

    // This stuff needs to be in main so we don't re-init the device
    let nvml = Nvml::init();
    // Get the first `Device` (GPU) in the system
    let nvml_obj = nvml.expect("wtf");
    let nv_dev = nvml_obj.device_by_index(0).unwrap();

    // Start the loop if asked for
    if do_loop {
        loop {
            println!("\x1B[2J\x1B[1;1H");

            print_nv_results(&nv_dev, do_loop);

            io::stdout().flush()?;
            if event::poll(Duration::from_millis(100))?
                && let Event::Key(key_event) = event::read()?
                && key_event.code == KeyCode::Char('q')
            {
                terminal::disable_raw_mode()?;
                break Ok(());
            }
            sleep(Duration::from_secs(loop_secs));
        }
    } else {
        print_nv_results(&nv_dev, do_loop);
        terminal::disable_raw_mode()?;
        Ok(())
    }
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
        }
    }
}

fn print_nv_results(device: &Device, looping: bool) {
    let stats = Stats::new(device);
    // print local time of course only if in a loop
    if looping {
        println!("{}\r", localtime::now().format("%Y-%m-%d %H:%M:%S"));
    }

    // print the brand name
    println!("Brand: {:?}\r", stats.brand);

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
