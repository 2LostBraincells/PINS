/// Everything related to outputing the ressults if to a file or to the std or 

use metal::Buffer;
use std::mem;
use core::slice;

pub fn parse(offsets: &[u16; 7], results: Buffer) -> String {
    //! Parses the results from a results buffer into a human readable string

    // extract to array
    let ptr = results.contents() as *const bool;
    let len = results.length() as usize / mem::size_of::<bool>();
    let slice = unsafe { slice::from_raw_parts(ptr, len) };

    // pre allocate full string
    let mut parsed = String::with_capacity(1_000_000);

    let mut index = 0;

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

    for year in 0..years {
        let actual_year = start_year + year;

        for month in 0..months {
            let actual_month = start_month + month;

            for day in 0..days {

                if !slice[index as usize] {
                    index += 1;
                    continue;
                }
                index += 1;

                
                parsed.push_str(
                    format!(
                        "{:02}{:02}{:02}-{:04} ",
                        actual_year,
                        actual_month,
                        start_day + day,
                        checksum 
                    ).as_str()
                );
            }
        }
        parsed.push_str("\n");
    }

    parsed
}
