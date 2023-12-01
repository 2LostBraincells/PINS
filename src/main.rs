use metal::*;
use std::mem;
use std::slice;
// Compiled metal lib 
const LIB_DATA: &[u8] = include_bytes!("shaders/compute.metallib");

const years: u64 = 10;
const months: u64 = 1;
const days: u64 = 1;

const total: u64 = years * months * days;

fn luhns(pin: [i32;10]) -> bool {
    //! Check a single PIN using the CPU
    //!
    //! This function is meant to be used to troubleshoot and test all other functions and methods
    //! Its essentialy the single source of truth that all other functions should follow
    let mut sum: i32 = 0;

    for (i, num) in pin.iter().enumerate() {
        sum += num + ((i as i32) & 1 ^ 1) * (num - ((num >= &5) as i32) * 9);
    }

    return sum % 10 == 0;
}

#[test]
fn luhns_check() {
    assert_eq!(luhns([0,6,1,0,0,9,2,4,5,4]), true);
    assert_eq!(luhns([0,6,0,3,1,7,9,2,7,6]), true);

    assert_eq!(luhns([1,6,0,3,1,7,9,2,7,6]), false);
}

fn main() {
    // Device reference
    let device: &DeviceRef = &Device::system_default().expect("No device found");

    // Library reference and function reference
    let lib = device.new_library_with_data(LIB_DATA).unwrap();
    let function = lib.get_function("check_individual", None).unwrap();

    // Pipeline declaration
    let pipeline = device
        .new_compute_pipeline_state_with_function(&function)
        .unwrap();

    let offsets: Vec<u16> = vec![0,6,0,3,1,7,9,2,7,6];
    let offset_length = 10 as u64;
    let offset_size = offset_length * core::mem::size_of::<u16>() as u64;

    let results: Vec<u8> = Vec::with_capacity(total.try_into().unwrap());
    let results_length = total as u64;
    let results_size  = results_length * core::mem::size_of::<u8>() as u64;


    let result_buffer = device.new_buffer_with_data(
        unsafe { mem::transmute(results.as_ptr()) }, // bytes
        results_size, // length
        MTLResourceOptions::StorageModeShared, // Storage mode
    );


    let offsets_buffer = device.new_buffer_with_data(
        unsafe { mem::transmute(offsets.as_ptr()) }, // bytes
        offset_size, // length
        MTLResourceOptions::StorageModeShared, // Storage mode
    );

    let command_queue = device.new_command_queue();

    let command_buffer = command_queue.new_command_buffer();

    let compute_encoder = command_buffer.new_compute_command_encoder();
    compute_encoder.set_compute_pipeline_state(&pipeline);

    compute_encoder.set_buffer(0, Some(&offsets_buffer), 0);
    compute_encoder.set_buffer(1, Some(&result_buffer), 0);

    let grid_size = metal::MTLSize::new(
        years, //width
        months, // height
        days); //depth

    let threadgroup_size = grid_size.clone();

    compute_encoder.dispatch_threads(grid_size, threadgroup_size);

    compute_encoder.end_encoding();
    command_buffer.commit();
    command_buffer.wait_until_completed();

    let ptr = result_buffer.contents() as *const u8;
    let len = result_buffer.length() as usize / mem::size_of::<u8>();
    let slice = unsafe { slice::from_raw_parts(ptr, len) };

    dbg!(slice);
}
