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

fn main() {
    let b: Integer = Integer::from(1) << 128;

    let pi = Float::with_val(P + 1, rug::float::Constant::Pi);
    let (m, e) = pi.to_integer_exp().unwrap();
    // println!("{e} {m:064x}");
    assert_eq!(e, -248);
    let (q, r) = &m.div_rem(b.clone());
    let hi: u128 = q.to_u128_wrapping();
    let lo: u128 = r.to_u128_wrapping();
    assert_eq!(hi.leading_zeros(), 6);
    println!("    // {pi}");
    println!(
        "    pub(crate) const PI: FP248 = FP248 {{\n sign: 0,\n signif: \
         u256::new(0x{hi:>032x}, 0x{lo:>032x}),\n }};"
    );
    let frac_pi_2 = Float::with_val(P, pi / 2);
    let (m, e) = frac_pi_2.to_integer_exp().unwrap();
    // println!("{e} {m:064x}");
    assert_eq!(e, -248);
    let (q, r) = &m.div_rem(b.clone());
    let hi: u128 = q.to_u128_wrapping();
    let lo: u128 = r.to_u128_wrapping();
    assert_eq!(hi.leading_zeros(), 7);
    println!("    // {frac_pi_2}");
    println!(
        "    pub(crate) const FRAC_PI_2: FP248 = FP248 {{\n sign: 0,\n \
         signif: u256::new(0x{hi:>032x}, 0x{lo:>032x}),\n }};"
    );
}
