/// Everything related to outputing the ressults if to a file or to the std or 

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
