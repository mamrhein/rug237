// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use std::ops::RangeInclusive;

use clap::Parser;
use rug237::{random_i32, EMAX, FP237};

const EXP_UPPER_BOUND: i32 = 275; // ⌊√EMAX⌉ - PM1
const NORMAL_EXP_LOWER_BOUND: i32 = -EXP_UPPER_BOUND;

// MIN_POSITIVE <= |f| <= f256::MAX
const NORMAL_EXP_RANGE: RangeInclusive<i32> =
    NORMAL_EXP_LOWER_BOUND..=EXP_UPPER_BOUND;

fn print_test_item(x: &FP237, y: &i32, z: &FP237) {
    let rx = x.decode(true);
    let rz = z.decode(true);
    println!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        rx.0, rx.1, rx.2 .0, rx.2 .1, y, rz.0, rz.1, rz.2 .0, rz.2 .1,
    );
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Number of test data to generate
    #[arg(short, long, default_value_t = 25)]
    n_test_data: u32,
}

fn main() {
    let args = Args::parse();

    let n = args.n_test_data;

    for _i in 0..n {
        let x = FP237::random_from_exp_range(&NORMAL_EXP_RANGE);
        let (_, e, _) = x.decode(false);
        let upper_limit = if e == 0 {
            EMAX
        } else {
            ((EMAX / e.abs()) as f64).sqrt() as i32
        };
        let y = random_i32(&(2..=upper_limit));
        let z = x.powi(y);
        print_test_item(&x, &y, &z);
    }
}
