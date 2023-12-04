use metal::*;
#[allow(unused_imports)]
use std::time::{Instant, Duration};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use std::path::Path;
use std::thread;
use std::mem;

mod gpu;
mod parser;
mod testing;

const OUTPUT: &str = "output.txt";

const CUBOIDS: u16 = 10_000;
const WORKERS: usize = 8;

const START_YEAR: u16 = 0;
const START_MONTH: u16 = 0;
const START_DAY: u16 = 0;

const YEARS: u16 = 100;
const MONTHS: u16 = 100;
const DAYS: u16 = 100;

const TOTAL: usize = YEARS as usize * MONTHS as usize * DAYS as usize;

const MULTIPLIERS: [u16;10] = [0, 2, 4, 6, 8, 1, 3, 5, 7, 9];


fn worker(file: Arc<Mutex<std::fs::File>>, id: u16, steps: u16) {
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

    // initalize timers
    // let mut setup_timer = Duration::new(0, 0);
    let mut compute_timer = Duration::new(0, 0);
    let mut parse_timer = Duration::new(0, 0);
    // let mut wait_timer = Duration::new(0, 0);
    let mut write_timer = Duration::new(0, 0);


    // while *reservation.lock().unwrap() != id {}

    let mut offsets_buffer: [u16; 7] = [
        START_YEAR,
        START_MONTH,
        START_DAY,
        0,
        YEARS,
        MONTHS,
        DAYS,
    ];

    let length = offsets_buffer.len() as u64;
    let size = length * core::mem::size_of::<u16>() as u64;

    // Setup GPU
    let device = &gpu::get_device();
    let queue = device.new_command_queue();


    // Define thread count
    let grid_size = metal::MTLSize::new(
        YEARS as u64, //width
        MONTHS as u64, // height
        DAYS as u64); //depth

    let group = gpu::max_group();


    // setup buffers
    let buffer_offsets = device.new_buffer_with_data(
        unsafe { mem::transmute(offsets_buffer.as_ptr()) }, // bytes
        size, // length
        MTLResourceOptions::StorageModeShared, // Storage mode
    );


    let buffer_multipliers = device.new_buffer_with_data(
        unsafe { mem::transmute(MULTIPLIERS.as_ptr()) },
        10 * core::mem::size_of::<u16>() as u64,
        MTLResourceOptions::StorageModeShared,
    );

    
    let buffer_result = device.new_buffer(
        (TOTAL * core::mem::size_of::<bool>()) as u64, // length
        MTLResourceOptions::StorageModeShared, // Storage mode
    );




    let a_ptr = buffer_offsets.contents() as *mut u16;

    println!("{}: Computing {} blocks", id, ((CUBOIDS - id - 1) / steps)+1);
    for i in (id..CUBOIDS).step_by(steps.into()) {

        // Update checksum
        offsets_buffer[3] = i;

        unsafe { 
            let ptr = a_ptr.offset(3);
            *ptr = i;
        }

        let buffer = queue.new_command_buffer();
        let encoder = buffer.new_compute_command_encoder();

        // setup shader function
        gpu::use_function(&device, "check_pin", encoder);


        // init buffers
        encoder.set_buffer(0, Some(&buffer_offsets), 0);
        encoder.set_buffer(1, Some(&buffer_result), 0);
        encoder.set_buffer(2, Some(&buffer_multipliers), 0);


        // Compute
        encoder.dispatch_threads(grid_size, group);
        encoder.end_encoding();

        let now = Instant::now();
        buffer.commit();
        buffer.wait_until_completed();
        compute_timer += now.elapsed();


        // results
        let now = Instant::now();
        let parsed = parser::parse(&offsets_buffer, buffer_result.clone());
        parse_timer += now.elapsed();
        let bytes = parsed.as_bytes();


        // wait for self's turn to write to file
        
        // write pre-computed contents to file
        let now = Instant::now();
        file.lock().unwrap().write_all(bytes).unwrap();
        write_timer += now.elapsed();

        // increment index
    }

    println!(
        "{}: Done! {:03}ms {:03}ms {:03}ms ",
        id,
        compute_timer.as_millis(),
        parse_timer.as_millis(),
        write_timer.as_millis(),
        );
}

fn main() {

    let now = Instant::now();
    
    // check if the output file exists
    let exists = Path::new(OUTPUT).exists();
    if !exists {
        println!("Not output file, creating {}", OUTPUT);
    } else {
        println!("Emptying {}", OUTPUT)
    }

    let file = File::create(OUTPUT).expect("Unable to open file");
    file.set_len(0).expect("Unable to empty file");


    let mut workers = vec![];

    let file = Arc::new(Mutex::new(OpenOptions::new()
        .write(true)
        .append(true)
        .create(false)
        .open("output.txt").unwrap()));

    // spawn threads
    for i in 0..WORKERS {
        let file = Arc::clone(&file);
        let handle = thread::spawn(move || worker(file, i.try_into().unwrap(), WORKERS.try_into().unwrap()));
        workers.push(handle);
    }

    println!("\nAll threads spawned\n");

    // wait for threads to finish
    for handle in workers {
        handle.join().unwrap();
    }

    println!("I: Done! compu parse write");
    println!("\nFinal: {}ms", now.elapsed().as_millis())
}
