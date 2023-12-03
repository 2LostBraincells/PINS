use metal::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::fs::File;
use std::mem;
use std::slice;
use indicatif::ProgressBar;
use indicatif::MultiProgress;
use indicatif::ProgressStyle;

mod gpu;
mod parser;

const CUBOIDS: u16 = 10_000;

const START_YEAR: u16 = 0;
const START_MONTH: u16 = 0;
const START_DAY: u16 = 0;

const YEARS: u16 = 100;
const MONTHS: u16 = 100;
const DAYS: u16 = 100;

const TOTAL: usize = YEARS as usize * MONTHS as usize * DAYS as usize;


#[allow(dead_code)]
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


fn worker(reservation: Arc<Mutex<u16>>, id: u16, steps: u16, progress: Arc<Mutex<ProgressBar>>) {
    //! A compute worker
    //!
    //! Validates all pins with checksum 0 to 10_000 with a step size of [steps] and a inital
    //! offset of [id]
    //!
    //! * `reservation` - A shared variable where a thread that is currently writing puts its id
    //! * `id` - Unique id. Used for getting a offset for the checksum
    //! * `steps` - How many steps to take between checksum numbers.
    //!
    //! # Example
    //! To have two threads sharing the workload, one could set them up like this
    //! ```rust
    //! let writer = Arc::new(Mutex::new(0));
    //!
    //! thread::spawn(move || worker(writer, 0, 2));
    //! thread::spawn(move || worker(writer, 1, 2));
    //! ```

    let mut offsets: [u16; 7] = [
        START_YEAR,
        START_MONTH,
        START_DAY,
        0,
        YEARS,
        MONTHS,
        DAYS
    ];

    let length = offsets.len() as u64;
    let size = length * core::mem::size_of::<u16>() as u64;
    println!("Worker {} will validating {} pins in {} groups, each group containing {} potential pins", id, TOTAL * ((CUBOIDS / steps) as usize), CUBOIDS, TOTAL);

    // Setup GPU
    println!("{}, Setting up GPU...", id);
    let device = &gpu::get_device();
    let queue = device.new_command_queue();


    // Define thread count
    let grid_size = metal::MTLSize::new(
        YEARS as u64, //width
        MONTHS as u64, // height
        DAYS as u64); //depth


    // setup buffers
    println!("{}, Creating buffers", id);
    let buffer_offsets = device.new_buffer_with_data(
        unsafe { mem::transmute(offsets.as_ptr()) }, // bytes
        size, // length
        MTLResourceOptions::StorageModeShared, // Storage mode
    );

    
    let buffer_result = device.new_buffer(
        (TOTAL * core::mem::size_of::<bool>()) as u64, // length
        MTLResourceOptions::StorageModeShared, // Storage mode
    );


    progress.lock().unwrap().set_style(ProgressStyle::with_template("[{elapsed_precise}] {bar:40.white/black} {pos:>7}/{len:7} {msg}").unwrap());

    for i in (id..CUBOIDS).step_by(steps.into()) {
        progress.lock().unwrap().inc(1);
        progress.lock().unwrap().set_message("Computing");

        // Update checksum
        offsets[3] = i;
        let a_ptr = buffer_offsets.contents() as *mut u16;
        unsafe { 
            let ptr = (a_ptr).offset(3);

            *ptr = i
        }

        let buffer = queue.new_command_buffer();
        let encoder = buffer.new_compute_command_encoder();

        // setup shader function
        gpu::use_function(&device, "check_pin", encoder);

        // init buffers
        encoder.set_buffer(0, Some(&buffer_offsets), 0);
        encoder.set_buffer(1, Some(&buffer_result), 0);

        // Compute
        encoder.dispatch_threads(grid_size, gpu::max_group());
        encoder.end_encoding();
        buffer.commit();
        buffer.wait_until_completed();

        // results
        let ptr = buffer_result.contents() as *const bool;
        let len = buffer_result.length() as usize / mem::size_of::<bool>();
        let slice = unsafe { slice::from_raw_parts(ptr, len) };
        progress.lock().unwrap().set_message("Waiting");

        while *reservation.lock().unwrap() != id {}
        *reservation.lock().unwrap() = id;
        
        progress.lock().unwrap().set_message("Writing");
        parser::write(&offsets, slice);

        if id == steps - 1 {
            *reservation.lock().unwrap() = 0;
        } else {
            *reservation.lock().unwrap() = id+1;
        }
    }
    progress.lock().unwrap().set_message("Done");
    progress.lock().unwrap().finish();
}

fn main() {

    let writer = Arc::new(Mutex::new(0));
    let writer_a = Arc::clone(&writer);
    let writer_b = Arc::clone(&writer);

    let progress = MultiProgress::new();

    let prgs_a = Arc::new(Mutex::new(progress.add(ProgressBar::new((CUBOIDS/2).into()))));
    let prgs_b = Arc::new(Mutex::new(progress.add(ProgressBar::new((CUBOIDS/2).into()))));

    let thread_a = thread::spawn(move || worker(writer_a, 0, 2, prgs_a));
    let thread_b = thread::spawn(move || worker(writer_b, 1, 2, prgs_b));

    thread_a.join().unwrap();
    thread_b.join().unwrap();
}
