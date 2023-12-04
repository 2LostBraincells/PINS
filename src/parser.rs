// Everything related to outputing the ressults if to a file or to the std or 

use metal::Buffer;
use std::mem;
use core::slice;

pub fn parse(offsets: &[u16; 7], results: Buffer) -> String {
    //! Parses the results from a results buffer into a human readable string

    // extract to array
    let ptr = results.contents() as *const bool;
    let len = results.length() as usize / mem::size_of::<bool>();
    let slice = unsafe { slice::from_raw_parts(ptr, len) };


    let [
        start_year,
        start_month,
        start_day,
        checksum,
        years,
        months,
        days,
    ] = {
        offsets
            .iter().map(|&num| num as usize) // convert to usize
            .collect::<Vec<usize>>()
            .try_into().unwrap()
    };

    let end_year = start_year + years;
    let end_month = start_month + months;


    // pre allocate full string
    let mut parsed = String::with_capacity(1_000_000);
    let mut index: usize = 0;



    // create a 12 byte array of char codes so that we can change the char codes for the date individualy
    // instead of having to create the entire string for every loop
    let mut base = format!("000000-{:04} ", checksum);
    let pin: &mut [u8] = unsafe { base.as_bytes_mut() };

    for year in start_year..end_year {
        pin[0] = ((year / 10) + 48) as u8; // OXxxxx-xxxx
        pin[1] = ((year % 10) + 48) as u8; // XOxxxx-xxxx


        for month in start_month..end_month {
            pin[2] = ((month / 10) + 48) as u8; // xxOXxx-xxxx
            pin[3] = ((month % 10) + 48) as u8; // xxXOxx-xxxx
            

            for day in 0..days {

                // continue if this index should be skipped
                if !slice[index] {
                    index += 1;
                    continue;
                }
                index += 1;

                let actual_day = start_day + day;

                // change the digits for the day
                pin[4] = ((actual_day / 10) + 48) as u8; // xxxxOX-xxxx
                pin[5] = ((actual_day % 10) + 48) as u8; // xxxxXO-xxxx

                
                let str = unsafe{ std::str::from_utf8_unchecked(pin) };
                
                parsed.push_str(str);
            }
        }


        // split lines based on year
        parsed.push('\n');
    }
    parsed
}
