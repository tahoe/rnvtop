use nvml_wrapper::Nvml;

fn main() -> Result<(), nvml_wrapper::error::NvmlError> {
    let nvml = Nvml::init()?;
    // Get the first `Device` (GPU) in the system
    let device = nvml.device_by_index(0)?;

    let brand = device.brand()?; // GeForce on my system
    println!("Brand: {:?}", brand);
    let fan_speed = device.fan_speed(0)?; // Currently 17% on my system
    println!("Fan Speed: {fan_speed}");
    let power_limit = device.enforced_power_limit()?; // 275k milliwatts on my system
    println!("Power Limit: {power_limit}");
    let encoder_util = device.encoder_utilization()?; // Currently 0 on my system; Not encoding anything
    println!("Encoder Use: {:?}", encoder_util);
    let gpu_util = device.utilization_rates()?;
    println!("GPU Use: {:?}", gpu_util);
    let memory_info = device.memory_info()?; // Currently 1.63/6.37 GB used on my system
                                             // println!("Memory Info: {:?}", memory_info);
    let mem_used = memory_info.used / 1_000_000_000;
    let mem_total = memory_info.total / 1_000_000_000;
    println!("Memory Usage: Used:{mem_used}GB, Total:{mem_total}");
    // println!("Memory Info: {:?}", memory_info);
    //
    // free: 11459231744,
    // reserved: 396558336,
    // total: 12884901888,
    // used: 1029111808,
    // version: 33554472,
    //

    // ... and there's a whole lot more you can do. Most everything in NVML is wrapped and ready to go
    Ok(())
}
