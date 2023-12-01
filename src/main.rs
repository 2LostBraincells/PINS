use metal::DeviceRef;
use metal::Device;
// Compiled metal lib 
const LIB_DATA: &[u8] = include_bytes!("compute.metallib");

fn check(pin: [u8;10]) -> bool {
    //! Check a single PIN using the CPU
    //!
    //! This function is meant to be used to troubleshoot and test all other functions and methods
    //! Its essentialy the single source of truth that all other functions should follow

    return false;
}

fn main() {
    println!("Hello, world!");

    // Device reference
    let device: &DeviceRef = &Device::system_default().expect("No device found");

    // Library reference and function reference
    let lib = device.new_library_with_data(LIB_DATA).unwrap();
    let function = lib.get_function("calculate_pins", None).unwrap();

    // Pipeline declaration
    let pipeline = device
        .new_compute_pipeline_state_with_function(&function)
        .unwrap();
}
