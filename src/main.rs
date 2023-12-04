use metal::*;
#[allow(unused_imports)]
use std::io::BufReader;
use std::time::{Instant, Duration};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::mem;
use core::slice;

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
    //! * `file` - A shared handle for writing to the file.
    //! * `id` - Unique id. Used for getting a offset for the checksum
    //! * `steps` - How many steps to take between checksum numbers.
    //!
    //! # Example
    //! To have two threads sharing the workload, one could set them up like this
    //! ```rust
    //! let file = Arc::new(Mutex::new(File::create("output.txt")));
    //! let file_a = Arc::clone(&file);
    //! let file_b = Arc::clone(&file);
    //!
    //!
    //! thread::spawn(move || worker(file_a, 0, 2));
    //! thread::spawn(move || worker(file_b, 1, 2));
    //! ```

    // initalize timers
    let mut setup_timer = Duration::new(0, 0);
    let mut compute_timer = Duration::new(0, 0);
    let mut parse_timer = Duration::new(0, 0);
    // let mut wait_timer = Duration::new(0, 0);
    let mut write_timer = Duration::new(0, 0);


    // while *reservation.lock().unwrap() != id {}

    let mut offsets_buffer: [u16; 7] = [
        START_YEAR,
        START_MONTH,
        START_DAY,
        id,
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
        unsafe { mem::transmute(offsets_buffer.as_ptr()) },
        size,
        MTLResourceOptions::StorageModeShared,
    );


    let buffer_multipliers = device.new_buffer_with_data(
        unsafe { mem::transmute(MULTIPLIERS.as_ptr()) },
        10 * core::mem::size_of::<u16>() as u64,
        MTLResourceOptions::StorageModeShared,
    );

    
    let buffer_result = device.new_buffer(
        (TOTAL * core::mem::size_of::<bool>()) as u64,
        MTLResourceOptions::StorageModeShared,
    );

    let ptr = buffer_result.contents() as *const bool;
    let len = buffer_result.length() as usize ; 
    let prev = unsafe { slice::from_raw_parts(ptr, len) };

    let mut prev_results = vec![false; buffer_result.length().try_into().unwrap()];

    // Pointer to the contents of the offsets buffer
    // Used to directly change the checksum for every compute cycle
    let a_ptr = buffer_offsets.contents() as *mut u16;


    println!("{}: Computing {} blocks", id, ((CUBOIDS - id - 1) / steps)+1);
    for i in (id..CUBOIDS).step_by(steps.into()) {

        let now = Instant::now();

        // cycle setup
        let buffer = queue.new_command_buffer();
        let encoder = buffer.new_compute_command_encoder();
        gpu::use_function(&device, "check_pin", encoder);

        // init buffers
        encoder.set_buffer(0, Some(&buffer_offsets), 0);
        encoder.set_buffer(1, Some(&buffer_result), 0);
        encoder.set_buffer(2, Some(&buffer_multipliers), 0);

        // Finalize the dispatch group
        encoder.dispatch_threads(grid_size, group);
        encoder.end_encoding();

        // clone prevous results
        prev_results.clone_from_slice(&prev);


        // chagne the checksum for the gpu
        unsafe { 
            let ptr = a_ptr.offset(3);
            *ptr = i;
        }

        setup_timer += now.elapsed();

        // Start compute cycle for the gpu
        let now2 = Instant::now();
        buffer.commit();

        if i != id {
            let now = Instant::now();
            // parse the results
            let parsed = parser::parse(&offsets_buffer, &prev_results);
            let bytes = parsed.as_bytes();
            parse_timer += now.elapsed();

            // write to file
            file.lock().unwrap().write_all(bytes).unwrap();
        }

        offsets_buffer[3] = i;

        buffer.wait_until_completed();
        compute_timer += now2.elapsed();
    }
    prev_results.clone_from_slice(&prev);

    let now = Instant::now();

    let parsed = parser::parse(&offsets_buffer, &prev_results);
    let bytes = parsed.as_bytes();
    parse_timer += now.elapsed();

    // write to file
    file.lock().unwrap().write_all(bytes).unwrap();

    // Write self time diagnostics to stdout
    println!(
        "{}: Done! {:03}ms {:03}ms {:03}ms {:03}ms",
        id,
        setup_timer.as_millis(),
        compute_timer.as_millis(),
        parse_timer.as_millis(),
        write_timer.as_millis(),
        );
}

fn main() {

    

    // list of thread handles
    let mut workers = vec![];


    // create and empty file
    println!("Emptying {}", OUTPUT);
    let file = Arc::new(Mutex::new(OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open("output.txt").unwrap()));
    file.lock().unwrap().set_len(0).expect("Unable to empty file");

    let now = Instant::now();

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


    println!("I: Done! setup compu parse write ");
    println!("\nFinal: {}ms", now.elapsed().as_millis());


    // scan file for invalid pin's
    let file = File::open("output.txt").unwrap();
    let reader = BufReader::new(file);

    let mut digits_array = [1; 10]; // Initialize an array of 10 elements with default value 0

    // get each line separatly
    for line in reader.lines() {
        match line {
            Ok(contents) => {
                for pin in contents.split(" ") {
                    // ignore invalid lengths
                    if pin.len() != 11 {continue;}

                    // fill digits_array with i32's of the given number
                    for (index, c) in pin.chars().filter(|c| c.is_digit(10)).enumerate() {
                        if index >= 10 {
                            break; // Break if we've collected 10 digits
                        }
                        digits_array[index] = c.to_digit(10).unwrap() as i32;
                    }

                    // test the pin
                    testing::test_pin(digits_array, true);
                }
            },
            Err(_) => println!("nth"),
        }
    }
}
