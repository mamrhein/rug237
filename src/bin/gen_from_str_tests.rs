// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use clap::Parser;
use rand::prelude::*;
use rug237::FP237;
use std::ops::RangeInclusive;
use std::str::FromStr;

const E10MAX: i32 = 78913;
const E10MIN: i32 = 1 - E10MAX;
const FAST_EXACT_EXP_RANGE: RangeInclusive<i32> = -102..=102;
const FAST_APPROX_EXP_RANGE: RangeInclusive<i32> = -512..=512;
const NORMAL_EXP_RANGE: RangeInclusive<i32> = -1024..=1024;
const EXTREME_EXP_RANGE: RangeInclusive<i32> = E10MIN..=E10MAX;
const SUBNORMAL_EXP_RANGE: RangeInclusive<i32> =
    E10MIN - MAX_N_DIGITS as i32..=E10MIN;

const MAX_N_DIGITS: u32 = 77;
const FAST_EXACT_MAX_N_DIGITS: u32 = 71;
const SLOW_MAX_N_DIGITS: u32 = 80;
const EXTREME_MAX_N_DIGITS: u32 = 183470;

const DIGITS: &[u8] = b"0123456789";

fn print_test_item(lit: &str, f: FP237) {
    let (s, e, (h, l)) = f.decode();
    println!("\"{}\"\t{}\t{}\t{}\t{}", lit, s, e, h, l)
}

fn gen_number_str(exp_range: &RangeInclusive<i32>) -> String {
    let mut rng = thread_rng();
    let sign: &str = match rng.gen_range(0..=2) {
        0 => "+",
        1 => "-",
        _ => "",
    };
    let max_n_digits = match *exp_range {
        FAST_EXACT_EXP_RANGE => FAST_EXACT_MAX_N_DIGITS,
        FAST_APPROX_EXP_RANGE => MAX_N_DIGITS,
        EXTREME_EXP_RANGE => EXTREME_MAX_N_DIGITS,
        _ => SLOW_MAX_N_DIGITS,
    };
    let n_digits: u32 = rng.gen_range(1..=max_n_digits);
    let mut n_fract_digits: u32 = rng.gen_range(0..n_digits);
    let n_int_digits: u32 = n_digits - n_fract_digits;
    let mut signif_digit = false;
    let int_digits: String = (0..n_int_digits)
        .map(|_| {
            let idx = rng.gen_range(0..DIGITS.len());
            if idx > 0 {
                signif_digit = true;
            }
            DIGITS[idx] as char
        })
        .collect();
    let mut fract_digits: String = (0..n_fract_digits)
        .map(|_| {
            let idx = rng.gen_range(0..DIGITS.len());
            if idx > 0 {
                signif_digit = true;
            }
            DIGITS[idx] as char
        })
        .collect();
    if !signif_digit {
        fract_digits.push('7');
        n_fract_digits += 1;
    }
    let exp: i32 = rng.gen_range(exp_range.clone()) + n_fract_digits as i32;
    if exp == 0 {
        format!("{}{}.{}", sign, int_digits, fract_digits)
    } else {
        if n_fract_digits == 0 {
            fract_digits = "0".to_string();
        }
        format!("{}{}.{}e{}", sign, int_digits, fract_digits, exp)
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Type of number: E = fast exact A = fast approx N = normal,
    /// S = subnormal, X = extreme
    #[arg(short, long, default_value_t = 'E')]
    type_of_num: char,

    /// Number of test data to generate
    #[arg(short, long, default_value_t = 10)]
    n_test_data: u32,
}

fn main() {
    let args = Args::parse();

    let exp_range = match args.type_of_num {
        'E' => &FAST_EXACT_EXP_RANGE,
        'A' => &FAST_APPROX_EXP_RANGE,
        'N' => &NORMAL_EXP_RANGE,
        'X' => &EXTREME_EXP_RANGE,
        'S' => &SUBNORMAL_EXP_RANGE,
        _ => panic!("Unkown type of number"),
    };

    for _i in 0..args.n_test_data {
        let s = gen_number_str(exp_range);
        let f = FP237::from_str(&*s).unwrap();
        print_test_item(&*s, f);
    }
}
