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
use rug237::{EMAX, EMIN, FP237, MIN_EXP_SUBNORMAL, PM1};

const SUBNORMAL_EXP_LOWER_BOUND: i32 = MIN_EXP_SUBNORMAL;
const SUBNORMAL_EXP_UPPER_BOUND: i32 = EMIN - 1;
const NORMAL_EXP_LOWER_BOUND: i32 = EMIN;
const EXP_UPPER_BOUND: i32 = EMAX - PM1;

// f256::MIN_GT_ZERO <= |f| < MIN_POSITIVE
const SUBNORMAL_EXP_RANGE: RangeInclusive<i32> =
    SUBNORMAL_EXP_LOWER_BOUND..=SUBNORMAL_EXP_UPPER_BOUND;
// MIN_POSITIVE <= |f| <= f256::MAX
const NORMAL_EXP_RANGE: RangeInclusive<i32> =
    NORMAL_EXP_LOWER_BOUND..=EXP_UPPER_BOUND;

fn print_test_item(x: &FP237, z: &FP237) {
    let rx = x.decode(true);
    let rz = z.decode(true);
    // assert_ne!(
    //     rz.2 .0,
    //     0,
    if rx.2 .0 != 0 && rz.2 .0 == 0 {
        println!(
            "\n{:?}\n{:?}\n{:?}\n{:?}\n",
            rx,
            x.decode(false),
            rz,
            z.decode(false)
        );
        let r = x.clone().sqrt();
        println!("{:?}\n", r.decode(false));
        assert_eq!(&r, z);
        assert!(false)
    };
    println!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        rx.0, rx.1, rx.2 .0, rx.2 .1, rz.0, rz.1, rz.2 .0, rz.2 .1,
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

    let n_sub_normal = args.n_test_data / 100 + 1;
    let n_normal = args.n_test_data - n_sub_normal;

    for _i in 0..n_normal {
        let x = FP237::random_from_exp_range(&NORMAL_EXP_RANGE).abs();
        let z = x.clone().sqrt();
        print_test_item(&x, &z);
    }

    for _i in 0..n_sub_normal {
        let x = FP237::random_from_exp_range(&SUBNORMAL_EXP_RANGE).abs();
        let z = x.clone().sqrt();
        print_test_item(&x, &z);
    }
}
