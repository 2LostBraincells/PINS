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


    // pre allocate full string
    let mut parsed = String::with_capacity(1_000_000);
    let mut index: usize = 0;



    // create a 11 byte array of char codes
    let mut base = format!("000000-{:04} ", checksum);
    let pin = unsafe { base.as_bytes_mut() };
    


    let mut actual_year = start_year;
    for year in 0..years {

        // change the digits for the year
        pin[0] = ((actual_year / 10) + 48) as u8;
        pin[1] = ((actual_year % 10) + 48) as u8;

        actual_year += 1;


        let mut actual_month = start_month;
        for month in 0..months {

            // change the digits for the month
            pin[2] = ((actual_month / 10) + 48) as u8;
            pin[3] = ((actual_month % 10) + 48) as u8;

            actual_month += 1;

            

            for day in 0..days {

                // continue if this index should be skipped
                if !slice[index] {
                    index += 1;
                    continue;
                }
                index += 1;

                let actual_day = start_day + day;

                // change the digits for the day
                pin[4] = ((actual_day / 10) + 48) as u8;
                pin[5] = ((actual_day % 10) + 48) as u8;

                
                let str = unsafe{ std::str::from_utf8_unchecked(pin) };
                
                parsed.push_str(str);
            }
        }


        // split lines based on year
        parsed.push('\n');
    }
    parsed
}
