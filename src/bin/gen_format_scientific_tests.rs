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
use rand::prelude::*;
use rug237::{EMAX, EMIN, FP255, MIN_EXP_SUBNORMAL, PM1};

const SUBNORMAL_EXP_LOWER_BOUND: i32 = MIN_EXP_SUBNORMAL;
const SUBNORMAL_EXP_UPPER_BOUND: i32 = EMIN - 1;
const NORMAL_EXP_LOWER_BOUND: i32 = EMIN;
const FAST_LOWER_BOUND: i32 = 0;
const FAST_LOWER_BOUND_MINUS_1: i32 = FAST_LOWER_BOUND - 1;
const FAST_UPPER_BOUND: i32 = 511_i32;
const FAST_UPPER_BOUND_PLUS_1: i32 = FAST_UPPER_BOUND + 1;
const EXP_UPPER_BOUND: i32 = EMAX;

// f256::MIN_GT_ZERO <= |f| < MIN_POSITIVE
const SUBNORMAL_EXP_RANGE: RangeInclusive<i32> =
    SUBNORMAL_EXP_LOWER_BOUND..=SUBNORMAL_EXP_UPPER_BOUND;
// f256::MIN_POSITIVE <= |f| < 1
const FRACT_EXP_RANGE: RangeInclusive<i32> =
    NORMAL_EXP_LOWER_BOUND..=FAST_LOWER_BOUND_MINUS_1;
// 1 <= |f| < 2²³⁶
const SMALL_FLOAT_EXP_RANGE: RangeInclusive<i32> = FAST_LOWER_BOUND..=PM1;
// 2²³⁶ <= |f| < 2⁵¹²
const SMALL_INT_EXP_RANGE: RangeInclusive<i32> = PM1..=FAST_UPPER_BOUND;
// 2⁵¹² <= |f| <= f256::MAX
const LARGE_INT_EXP_RANGE: RangeInclusive<i32> =
    FAST_UPPER_BOUND_PLUS_1..=EXP_UPPER_BOUND;

fn print_test_item(f: FP255, p: usize, lit: &str) {
    let (s, e, (h, l)) = f.decode(false);
    println!("{}\t{}\t{}\t{}\t{}\t\"{}\"", s, e, h, l, p, lit)
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Type of number: N = small float I = small int F = normal,
    /// S = subnormal, X = large int
    #[arg(short, long, default_value_t = 'N')]
    type_of_num: char,

    /// Number of test data to generate
    #[arg(short, long, default_value_t = 10)]
    n_test_data: u32,
}

fn main() {
    let mut rng = thread_rng();
    let args = Args::parse();

    let exp_range = match args.type_of_num {
        'N' => &SMALL_FLOAT_EXP_RANGE,
        'I' => &SMALL_INT_EXP_RANGE,
        'F' => &FRACT_EXP_RANGE,
        'X' => &LARGE_INT_EXP_RANGE,
        'S' => &SUBNORMAL_EXP_RANGE,
        _ => panic!("Unkown type of number"),
    };

    for _i in 0..args.n_test_data {
        let f = FP255::random_from_exp_range(exp_range);
        let p = rng.gen_range(0..=75);
        // rug takes the precision as the total number of digits, not the
        // number of fractional digits!
        let s = format!("{f:.*e}", p + 1);
        print_test_item(f, p, &*s);
    }
}
