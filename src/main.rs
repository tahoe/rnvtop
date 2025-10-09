use chrono::offset::Local as localtime;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal,
};
use nvml_wrapper::{Device, Nvml};
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
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key_event) = event::read()? {
                    if key_event.code == KeyCode::Char('q') {
                        terminal::disable_raw_mode()?;
                        break Ok(());
                    }
                }
            }
            sleep(Duration::from_secs(loop_secs));
        }
    } else {
        print_nv_results(&nv_dev, do_loop);
        terminal::disable_raw_mode()?;
        Ok(())
    }
}

fn print_nv_results(device: &Device, looping: bool) {
    // print local time of course only if in a loop
    if looping {
        println!("{}\r", localtime::now().format("%Y-%m-%d %H:%M:%S"));
    }

    // Brand is simple...
    let brand = device.brand().unwrap(); // GeForce on my system
    println!("Brand: {:?}\r", brand);

    // Fan speed is also simple
    let fan_speed = device.fan_speed(0).unwrap(); // Currently 17% on my system
    println!("Fan Speed: {fan_speed}%\r");

    // Power output is simple but we want in Watts, not micro watts...
    let pwr_wtts_used = device.power_usage().unwrap() / 1000;
    let pwr_wtts_cap = device.power_management_limit().unwrap() / 1000;
    println!("Power Usage: Used:{pwr_wtts_used}W, Max:{pwr_wtts_cap}W\r");

    // Get base outputs from device
    let memory_info = device.memory_info().unwrap();
    let gpu_util = device.utilization_rates().unwrap().gpu;
    let encoder_util = device.encoder_utilization().unwrap().utilization;
    let decoder_util = device.decoder_utilization().unwrap().utilization;

    // Get MEM specific outputs and print
    let mem_used = format!("{:.2}", memory_info.used as f32 / 1_000_000_000.0);
    let mem_total = format!("{:.2}", memory_info.total as f32 / 1_000_000_000.0);
    println!("Memory Usage: Used:{mem_used}GB, Total:{mem_total}GB\r");

    // print the GPU usage
    // print without newline so as not to waste space...
    println!(
        "GPU Usage: {gpu_util}%, Encoder: {:?}%, Decoder: {:?}%\r",
        encoder_util, decoder_util
    );
}
