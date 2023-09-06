// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use rug::{Float, Integer};

const P: u32 = 249;
const N: u32 = P;

fn main() {
    let b: Integer = Integer::from(1) << 128;

    println!("pub(crate) const ATANS: [FP248; {N}] = [");
    for i in 0..N {
        let f = Float::with_val(P, Float::i_exp(1, -(i as i32)));
        let a = f.clone().atan();
        let (mut m, e) = a.to_integer_exp().unwrap();
        // println!("{i} {e} {m:064x}");
        m >>= -(P as i32) - e + 1;
        let (q, r) = &m.div_rem(b.clone());
        let hi: u128 = q.to_u128_wrapping();
        let lo: u128 = r.to_u128_wrapping();
        // println!("{}", hi.leading_zeros());
        println!("    // {a}");
        println!(
            "    FP248 {{ sign: 0, signif: u256::new(0x{hi:>032x}, \
             0x{lo:>032x}) }},"
        );
    }
    println!("]");
}
