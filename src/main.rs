use nvml_wrapper::Nvml;
use std::io::{self, Write};
use std::thread::sleep;
use std::time::Duration;

fn main() -> io::Result<()> {
    loop {
        println!("\x1B[2J\x1B[1;1H");
        print_nvl();
        io::stdout().flush()?;
        sleep(Duration::from_secs(1));
    }
}

fn print_nvl() {
    let nvml = Nvml::init();
    // Get the first `Device` (GPU) in the system
    let nvml_obj = nvml.expect("wtf"); //.device_by_index(0).unwrap();
    let device = nvml_obj.device_by_index(0).unwrap();

    // Brand is simple...
    let brand = device.brand().unwrap(); // GeForce on my system
    println!("Brand: {:?}", brand);

    // Fan speed is also simple
    let fan_speed = device.fan_speed(0).unwrap(); // Currently 17% on my system
    println!("Fan Speed: {:?}%", fan_speed);

    // Power output is simple but we want in Watts, not micro watts...
    let pwr_wtts_used = device.power_usage().unwrap() / 1000;
    let pwr_wtts_cap = device.power_management_limit().unwrap() / 1000;
    println!("Power Usage: Used:{pwr_wtts_used}W, Max:{pwr_wtts_cap}W");

    // Get base outputs from device
    let memory_info = device.memory_info().unwrap();
    let gpu_util = device.utilization_rates().unwrap().gpu;
    // let encoder_util = device.encoder_utilization();
    // let decoder_util = device.decoder_utilization();

    // Get MEM specific outputs and print
    let mem_used = format!("{:.2}", memory_info.used as f32 / 1_000_000_000.0);
    let mem_total = format!("{:.2}", memory_info.total as f32 / 1_000_000_000.0);
    println!("Memory Usage: Used:{mem_used}GB, Total:{mem_total}GB");

    // get GPU specific usage info and print
    println!("GPU Use: {:?}%", gpu_util);
    // println!("Memory Info: {:?}", memory_info);
    //
    // free: 11459231744,
    // reserved: 396558336,
    // total: 12884901888,
    // used: 1029111808,
    // version: 33554472,
    //

    // ... and there's a whole lot more you can do. Most everything in NVML is wrapped and ready to go
    //let _ = io::stdout().flush();
    //Ok(())
}
