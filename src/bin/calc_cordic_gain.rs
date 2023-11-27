// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use std::ops::Add;

use rug::{Float, Integer};

const P: u32 = 255;
const N: u32 = P;

fn main() {
    let b: Integer = Integer::from(1) << 128;

    let one = Float::with_val(P, 1);
    let mut k = one.clone();
    for i in 0..=N {
        let f = Float::with_val(P, Float::i_exp(1, -2 * i as i32));
        k *= f.add(&one).sqrt();
    }
    // println!("{k}");
    let (m, mut e) = k.to_integer_exp().unwrap();
    e += P as i32 - 1;
    let (q, r) = &m.div_rem(b.clone());
    let hi: u128 = q.to_u128_wrapping();
    let lo: u128 = r.to_u128_wrapping();
    assert_eq!(hi.leading_zeros(), 1);
    // println!("{}", hi.leading_zeros());
    println!("// ≈{k}");
    println!(
        "pub(crate) const K: FP255 = FP255 {{ sign: 1, exp: {e}, signif: \
         u256::new(0x{hi:>032x}, 0x{lo:>032x}) }};"
    );
    let p = Float::with_val(P, k.recip());
    // println!("{p}");
    let (m, mut e) = p.to_integer_exp().unwrap();
    e += P as i32 - 1;
    // println!("{e} {m:064x}");
    let (q, r) = &m.div_rem(b.clone());
    let hi: u128 = q.to_u128_wrapping();
    let lo: u128 = r.to_u128_wrapping();
    assert_eq!(hi.leading_zeros(), 1);
    // println!("{}", hi.leading_zeros());
    println!("// ≈{p}");
    println!(
        "pub(crate) const P: FP255 = FP255 {{ sign: 1, exp: {e}, signif: \
         u256::new(0x{hi:>032x}, 0x{lo:>032x}) }};"
    );
}
