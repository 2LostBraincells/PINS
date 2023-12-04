

#[allow(dead_code)]
pub fn check_luhns(pin: [i32;10]) -> bool {
    //! Check a single PIN using luhns algorithm on the CPU
    //!

    let mut sum: i32 = 0;

    for (i, num) in pin.iter().enumerate() {
        sum += num + ((i as i32) & 1 ^ 1) * (num - ((num >= &5) as i32) * 9);
    }

    return sum % 10 == 0;
}

#[allow(dead_code)]
pub fn check_date(pin: [i32;10]) -> bool {
    //! Check a single PIN's date validity
    //!
    //! i.e return false for all PINS where the date is impossible.
    //! For example no one can have the pin 0613092454 as there is no month 13

    let year    = pin[0] * 10 + pin[1];
    let month   = pin[2] * 10 + pin[3];
    let day     = pin[4] * 10 + pin[5];

    if month > 12 { return false; }
    if month == 0 { return false; }

    let max_day = match month {
        2 => 28 + (year % 4 == 0) as i32,
        4 | 6 | 9 | 11 => 30,
        _ => 31
    };

    if day > max_day && day < 61 { return false; }
    if day > max_day + 60 { return false; }

    return true;
}

#[allow(dead_code)]
pub fn check(pin: [i32;10]) -> bool {
    //! This function is meant to be used to troubleshoot and test all other functions and methods
    //! Its essentialy the single source of truth that all other functions should follow


    if !check_date(pin) { return false; }
    if !check_luhns(pin) { return false; }

    return true;
}

#[allow(dead_code)]
pub fn test_pin(pin: [i32;10], expected: bool) {
    //! Test function with better diagnostics than a normal assert_eq

    let actual = check(pin);
    dbg!(actual);

    // if result is correct, do nothing and return
    if actual == expected { return; }

    let pin_str: String = pin.iter().map(|&num| num.to_string()).collect();
    let luhns_result = check_luhns(pin);
    let date_result = check_date(pin);

    panic!(
        "Check on PIN {} returned an invalid result:\n  expected: {}\n  actual:   {}\n     Luhns:    {}\n     Date:     {}",
        pin_str, expected, actual, luhns_result, date_result
    );


}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_valid() {
        test_pin([0, 6, 1, 0, 0, 9, 2, 4, 5, 4], true);
        test_pin([0, 6, 0, 3, 1, 7, 9, 2, 7, 6], true);

        test_pin([0, 9, 0, 6, 2, 7, 8, 8, 9, 0], true);
        test_pin([7, 1, 0, 7, 0, 8, 8, 5, 0, 7], true);
        test_pin([6, 5, 0, 6, 1, 4, 8, 9, 9, 5], true);
    }

    #[test]
    fn should_be_valid() {
        //! Hand calculated to be valid but not from actual known people
        test_pin([0,0,0,1,0,6,2,4,5,4], true);

    }

    #[test]
    fn faulty_date() {
        test_pin([0, 6, 1, 0, 5, 8, 2, 4, 5, 4], false);
        test_pin([0, 1, 1, 0, 9, 5, 2, 4, 5, 4], false);
    }

    #[test]
    fn faulty_luhns() {
        test_pin([0, 5, 1, 3, 0, 7, 2, 4, 5, 4], false);
    }
}
