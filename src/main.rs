use metal::*;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::mem;
use std::slice;

mod gpu;
mod parser;
mod testing;

const CUBOIDS: u16 = 10_000;

const START_YEAR: u16 = 0;
const START_MONTH: u16 = 0;
const START_DAY: u16 = 0;

const YEARS: u16 = 99;
const MONTHS: u16 = 99;
const DAYS: u16 = 99;

const TOTAL: usize = YEARS as usize * MONTHS as usize * DAYS as usize;


fn worker(reservation: Arc<Mutex<u16>>, id: u16, steps: u16) {
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
    println!("{}: will validate {} pins in groups {} to {} with a step size of {}, each group containing {} potential pins", id, TOTAL * ((CUBOIDS / steps) as usize), id, CUBOIDS, steps, TOTAL);

    // Setup GPU
    println!("{}: Setting up GPU...", id);
    let device = &gpu::get_device();
    let queue = device.new_command_queue();


    // Define thread count
    let grid_size = metal::MTLSize::new(
        YEARS as u64, //width
        MONTHS as u64, // height
        DAYS as u64); //depth


    // setup buffers
    println!("{}: Creating buffers", id);
    let buffer_offsets = device.new_buffer_with_data(
        unsafe { mem::transmute(offsets.as_ptr()) }, // bytes
        size, // length
        MTLResourceOptions::StorageModeShared, // Storage mode
    );

    
    let buffer_result = device.new_buffer(
        (TOTAL * core::mem::size_of::<bool>()) as u64, // length
        MTLResourceOptions::StorageModeShared, // Storage mode
    );



    println!("{}: Computing...", id);
    for i in (id..CUBOIDS).step_by(steps.into()) {

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

        let parsed = parser::parse(&offsets, slice);
        let bytes = parsed.as_bytes();

        while *reservation.lock().unwrap() != id {}
        *reservation.lock().unwrap() = id;
        
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(false)
            .open("output.txt").unwrap();

        file.write_all(bytes).unwrap();



        if id == steps - 1 {
            *reservation.lock().unwrap() = 0;
        } else {
            *reservation.lock().unwrap() = id+1;
        }
    }

    println!("{}: Done", id);
}

fn main() {

    let writer = Arc::new(Mutex::new(0));
    let writer_a = Arc::clone(&writer);
    let writer_b = Arc::clone(&writer);

    let thread_a = thread::spawn(move || worker(writer_a, 0, 2));
    let thread_b = thread::spawn(move || worker(writer_b, 1, 2));

    thread_a.join().unwrap();
    thread_b.join().unwrap();
}
