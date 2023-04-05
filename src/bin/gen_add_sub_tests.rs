// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use clap::Parser;
use rug237::{EMAX, EMIN, FP237, MIN_EXP_SUBNORMAL, P};
use std::ops::RangeInclusive;

const SUBNORMAL_EXP_LOWER_BOUND: i32 = MIN_EXP_SUBNORMAL;
const SUBNORMAL_EXP_UPPER_BOUND: i32 = EMIN - 1;
const MIXED_EXP_UPPER_BOUND: i32 = EMIN + 2;
const NORMAL_EXP_LOWER_BOUND: i32 = EMIN;
const EXP_UPPER_BOUND: i32 = EMAX as i32;

// f256::MIN_GT_ZERO <= |f| < MIN_POSITIVE
const SUBNORMAL_EXP_RANGE: RangeInclusive<i32> =
    SUBNORMAL_EXP_LOWER_BOUND..=SUBNORMAL_EXP_UPPER_BOUND;
// f256::MIN_GT_ZERO <= |f| < 4 * MIN_POSITIVE
const MIXED_EXP_RANGE: RangeInclusive<i32> =
    SUBNORMAL_EXP_LOWER_BOUND..=MIXED_EXP_UPPER_BOUND;
// MIN_POSITIVE <= |f| <= f256::MAX
const NORMAL_EXP_RANGE: RangeInclusive<i32> =
    NORMAL_EXP_LOWER_BOUND..=EXP_UPPER_BOUND;

fn print_test_item(x: &FP237, y: &FP237, z: &FP237) {
    let rx = x.decode(true);
    let ry = y.decode(true);
    let rz = z.decode(true);
    println!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        rx.0,
        rx.1,
        rx.2 .0,
        rx.2 .1,
        ry.0,
        ry.1,
        ry.2 .0,
        ry.2 .1,
        rz.0,
        rz.1,
        rz.2 .0,
        rz.2 .1,
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

    let n_sub_normal = args.n_test_data / 20;
    let n_normal = args.n_test_data - n_sub_normal;

    for _i in 0..n_normal {
        let x = FP237::random_from_exp_range(&NORMAL_EXP_RANGE);
        let (_, e, _) = x.decode(false);
        let y = FP237::random_from_exp_range(&(e - P as i32..=e + P as i32));
        let z = &x + &y;
        print_test_item(&x, &y, &z);
    }

    for _i in 0..n_sub_normal {
        let x = FP237::random_from_exp_range(&MIXED_EXP_RANGE);
        let y = FP237::random_from_exp_range(&SUBNORMAL_EXP_RANGE);
        let z = &x + &y;
        print_test_item(&x, &y, &z);
    }
}
