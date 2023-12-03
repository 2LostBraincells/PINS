/// Everything related to outputing the ressults if to a file or to the std or 
use std::fs::File;
use std::io::prelude::*;

#[allow(dead_code)]
pub fn print(offsets: &[u16; 7], results: &[bool]) {
    for year in 0..offsets[4]{
        for month in 0..offsets[5]{
            for day in 0..offsets[6]{
                let index = year + offsets[4] * month + offsets[4] * offsets[5] * day;
                if !results[index as usize] {
                    continue;
                }
                print!("{:02}", year + offsets[0]);
                print!("{:02}", month + offsets[1]);
                print!("{:02}", day + offsets[2]);
                print!("{:04}", offsets[3]);
                println!();
            }
        }
    }
}

pub fn write(file: &mut File, offsets: &[u16; 7], results: &[bool]) {
    let mut parsed = String::new();

    for year in 0..offsets[4]{
        for month in 0..offsets[5]{
            for day in 0..offsets[6]{
                let index = year + offsets[4] * month + offsets[4] * offsets[5] * day;
                if !results[index as usize] {
                    continue;
                }

                parsed.push_str(
                    format!(
                        "{:02}{:02}{:02}-{:04} ",
                        year, month, day, offsets[3]
                    ).as_str()
                );
            }
        }
        parsed.push_str("\n");
    }

    file.write_all(parsed.as_bytes()).unwrap();

}
