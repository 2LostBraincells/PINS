// Compiled metal lib 
const LIB_DATA: &[u8] = include_bytes!("compute.metallib");

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
