use nvml_wrapper::{Device, Nvml};
use std::env;
use std::io::{self, Write};
use std::thread::sleep;
use std::time::Duration;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut do_loop = false;
    if args.len() > 1 && args[1] == "-l" {
        do_loop = true;
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

            print_nv_results(&nv_dev);

            io::stdout().flush()?;
            sleep(Duration::from_secs(1));
        }
    } else {
        print_nv_results(&nv_dev);
        Ok(())
    }
}

fn print_nv_results(device: &Device) {
    // Brand is simple...
    let brand = device.brand().unwrap(); // GeForce on my system
    println!("Brand: {:?}", brand);

    // Fan speed is also simple
    let fan_speed = device.fan_speed(0).unwrap(); // Currently 17% on my system
    println!("Fan Speed: {fan_speed}%");

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
    println!("GPU Use: {gpu_util}%");
}
