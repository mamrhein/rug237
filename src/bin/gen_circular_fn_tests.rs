// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use clap::Parser;
use rug::{ops::CompleteRound, Float};
use rug237::{FP237, P, PM1};

const EXP_UPPER_BOUND: i32 = 2 * PM1 + 14;
const EXP_LOWER_BOUND: i32 = -PM1 / 2 - 4;

fn print_test_item(x: &FP237, z: &FP237) {
    let rx = x.decode(false);
    let rz = z.decode(false);
    println!(
        "{}\t{}\t0x{:032x}\t0x{:032x}\t{}\t{}\t0x{:032x}\t0x{:032x}",
        rx.0, rx.1, rx.2 .0, rx.2 .1, rz.0, rz.1, rz.2 .0, rz.2 .1,
    );
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// circular function: sin cos tan cot
    #[arg(short, long, default_value = "sin")]
    func: String,
    /// Range of input value f: C = 0..2π S = 2π..T L = T..
    #[arg(short, long, default_value_t = 'C')]
    range: char,
    /// Number of test data to generate
    #[arg(short, long, default_value_t = 25)]
    n_test_data: u32,
}

fn main() {
    let args = Args::parse();

    let pi = Float::with_val(P + 1, rug::float::Constant::Pi);
    let tau = FP237::new(Float::with_val(P, 2 * pi));
    let lower_limit =
        FP237::new(Float::u_exp(2, EXP_LOWER_BOUND).complete(P));
    let fast_limit = FP237::new(Float::u_exp(2, 20).complete(P));
    let upper_limit =
        FP237::new(Float::u_exp(2, EXP_UPPER_BOUND).complete(P));

    let func = match args.func.as_str() {
        "sin" => FP237::sin,
        "cos" => FP237::cos,
        "tan" => FP237::tan,
        "cot" => FP237::cot,
        _ => panic!("Unkown func"),
    };
    let range = match args.range {
        'C' => lower_limit..tau,
        'S' => tau..fast_limit,
        'L' => fast_limit..upper_limit,
        _ => panic!("Unkown range"),
    };
    let exp_low = range.start.decode(false).1 + PM1;
    let exp_high = range.end.decode(false).1 + PM1;
    let exp_range = exp_low..=exp_high;

    for _i in 0..args.n_test_data {
        loop {
            let a = FP237::random_from_exp_range(&exp_range);
            if range.contains(&a) {
                let res = func(&a);
                print_test_item(&a, &res);
                break;
            }
        }
    }
}
