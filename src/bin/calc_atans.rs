// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use rug::{Float, Integer};

const P: u32 = 255;
const N: u32 = P;

fn main() {
    let b: Integer = Integer::from(1) << 128;

    println!("pub(crate) const ATANS: [FP255; {N}] = [");
    for i in 0..N {
        let f = Float::with_val(P, Float::i_exp(1, -(i as i32)));
        let a = f.clone().atan();
        let (m, mut e) = a.to_integer_exp().unwrap();
        e += P as i32 - 1;
        // println!("{i} {e} {m:064x}");
        let (q, r) = &m.div_rem(b.clone());
        let hi: u128 = q.to_u128_wrapping();
        let lo: u128 = r.to_u128_wrapping();
        assert_eq!(hi.leading_zeros(), 1);
        println!("    // {a}");
        println!(
            "    FP255 {{ sign: 0, exp: {e}, signif: \
             u256::new(0x{hi:>032x}, 0x{lo:>032x}) }},"
        );
    }
    println!("]");
}
