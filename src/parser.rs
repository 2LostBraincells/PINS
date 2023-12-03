/// Everything related to outputing the ressults if to a file or to the std or 

use metal::Buffer;
use std::mem;
use core::slice;

pub fn parse(offsets: &[u16; 7], results: Buffer) -> String {

    // extract to array
    let ptr = results.contents() as *const bool;
    let len = results.length() as usize / mem::size_of::<bool>();
    let slice = unsafe { slice::from_raw_parts(ptr, len) };

    let mut parsed = String::new();

    let mut index = 0;

    for year in offsets[0]..offsets[4]{
        for month in offsets[1]..offsets[5]{
            for day in offsets[2]..offsets[6]{
                if !slice[index as usize] {
                    index += 1;
                    continue;
                }
                index += 1;

                parsed.push_str(
                    format!(
                        "{:02}{:02}{:02}-{:04} ",
                        year+offsets[0], month+offsets[1], day+offsets[2], offsets[3]
                    ).as_str()
                );
            }
        }
        parsed.push_str("\n");
    }

    parsed
}
