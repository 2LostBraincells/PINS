use metal::*;
use std::mem;
use std::slice;
// Compiled metal lib 
const LIB_DATA: &[u8] = include_bytes!("shaders/compute.metallib");

const YEARS: usize = 10;
const MONTHS: usize = 1;
const DAYS: usize = 1;

const TOTAL: usize = YEARS * MONTHS * DAYS;

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

#[cfg(test)]
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
    let function = lib.get_function("check_pin", None).unwrap();

    // Pipeline declaration
    let pipeline = device
        .new_compute_pipeline_state_with_function(&function)
        .unwrap();

    let offsets: Vec<u16> = vec![6, 10, 9, 2454];

    let length = offsets.len() as u64;
    let size = length * core::mem::size_of::<u16>() as u64;

    let buffer_a = device.new_buffer_with_data(
        unsafe { mem::transmute(offsets.as_ptr()) }, // bytes
        size, // length
        MTLResourceOptions::StorageModeShared, // Storage mode
    );

    let buffer_result = device.new_buffer(
        (TOTAL * core::mem::size_of::<bool>()) as u64, // length
        MTLResourceOptions::StorageModeShared, // Storage mode
    );

    let command_queue = device.new_command_queue();
    let command_buffer = command_queue.new_command_buffer();
    let compute_encoder = command_buffer.new_compute_command_encoder();

    compute_encoder.set_compute_pipeline_state(&pipeline);
    compute_encoder.set_buffers(
        0, // start index
        &[Some(&buffer_a), Some(&buffer_result)], //buffers
        &[0; 2], //offset
    );


    let grid_size = metal::MTLSize::new(
        YEARS.try_into().unwrap(), //width
        MONTHS.try_into().unwrap(), // height
        DAYS.try_into().unwrap()); //depth

    let threadgroup_size = metal::MTLSize::new(
        YEARS.try_into().unwrap(), //width
        MONTHS.try_into().unwrap(), // height
        DAYS.try_into().unwrap()); //depth

    compute_encoder.dispatch_threads(grid_size, threadgroup_size);


    compute_encoder.end_encoding();
    command_buffer.commit();
    command_buffer.wait_until_completed();

    let ptr = buffer_result.contents() as *const bool;
    let len = buffer_result.length() as usize / mem::size_of::<bool>();
    let slice = unsafe { slice::from_raw_parts(ptr, len) };
    dbg!(slice);
}
