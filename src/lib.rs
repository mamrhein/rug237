// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use std::{
    cmp::Ordering,
    fmt::{Display, Formatter, LowerExp},
    ops::{Add, Div, Mul, RangeInclusive, Rem, Sub},
    str::FromStr,
};

use rand::prelude::*;
use rug::{
    float::{Constant, ParseFloatError, Round},
    ops::Pow,
    Assign, Float, Integer,
};

pub const P: u32 = 237;
pub const PM1: i32 = P as i32 - 1;
pub const EMAX: i32 = 262143;
pub const EMIN: i32 = 1 - EMAX;
pub const MIN_EXP_SUBNORMAL: i32 = EMIN - PM1;

#[derive(Clone, Debug, PartialEq)]
pub struct FP237 {
    pub f: Float,
    pub(crate) o: Ordering,
}

impl FP237 {
    #[allow(non_snake_case)]
    pub fn Log2() -> Self {
        Self {
            f: Float::with_val(P, Constant::Log2),
            o: Ordering::Equal,
        }
    }

    #[allow(non_snake_case)]
    pub fn Pi() -> Self {
        Self {
            f: Float::with_val(P, Constant::Pi),
            o: Ordering::Equal,
        }
    }

    #[allow(non_snake_case)]
    pub fn Euler() -> Self {
        Self {
            f: Float::with_val(P, Constant::Euler),
            o: Ordering::Equal,
        }
    }

    #[allow(non_snake_case)]
    pub fn Catalan() -> Self {
        Self {
            f: Float::with_val(P, Constant::Catalan),
            o: Ordering::Equal,
        }
    }

    pub fn new(val: Float) -> Self {
        Self {
            f: val,
            o: Ordering::Equal,
        }
    }

    pub fn trunc(&self) -> Self {
        Self {
            f: self.f.clone().trunc(),
            o: Ordering::Equal,
        }
    }

    pub fn abs(self) -> Self {
        Self {
            f: self.f.abs(),
            o: Ordering::Equal,
        }
    }

    pub fn sqrt(self) -> Self {
        Self {
            f: self.f.sqrt(),
            o: Ordering::Equal,
        }
    }

    pub fn fma(&self, m: &Self, a: &Self) -> Self {
        let f = &self.f * &m.f + &a.f;
        let (f, o) = Float::with_val_round(P, f, Round::Nearest);
        Self { f, o }
    }

    pub fn sos(&self, other: &Self) -> Self {
        let f = self.f.clone().mul_add_mul(&self.f, &other.f, &other.f);
        let (f, o) = Float::with_val_round(P, f, Round::Nearest);
        Self { f, o }
    }

    pub fn sin(&self) -> Self {
        let f = self.f.sin_ref();
        let (f, o) = Float::with_val_round(P, f, Round::Nearest);
        Self { f, o }
    }

    pub fn cos(&self) -> Self {
        let f = self.f.cos_ref();
        let (f, o) = Float::with_val_round(P, f, Round::Nearest);
        Self { f, o }
    }

    pub fn tan(&self) -> Self {
        let f = self.f.tan_ref();
        let (f, o) = Float::with_val_round(P, f, Round::Nearest);
        Self { f, o }
    }

    pub fn cot(&self) -> Self {
        let f = self.f.cot_ref();
        let (f, o) = Float::with_val_round(P, f, Round::Nearest);
        Self { f, o }
    }

    pub fn decode(&self, reduce: bool) -> (u32, i32, (u128, u128)) {
        let b: Integer = Integer::from(u128::MAX) + 1;
        match self.f.to_integer_exp() {
            Some((mut i, mut e)) => {
                let s = self.f.is_sign_negative() as u32;
                if e > EMAX - PM1 {
                    return (s, EMAX + 1, (0, 0));
                }
                i.abs_mut();
                if reduce && i != 0 {
                    while i.is_even() {
                        i >>= 1;
                        e += 1;
                    }
                }
                if e < MIN_EXP_SUBNORMAL {
                    let shift = MIN_EXP_SUBNORMAL - e;
                    let mask = (Integer::from(1) << shift) - 1;
                    let tie = Integer::from(1) << (shift - 1);
                    let rem = &i & mask;
                    i >>= shift;
                    if rem > tie
                        || rem == tie
                            && (self.o != Ordering::Greater || i.is_odd())
                    {
                        i += 1;
                    }
                    e = MIN_EXP_SUBNORMAL;
                }
                if i == 0 {
                    // println!("Near 0: {:?}", self.f.to_integer_exp());
                    return (s, 0, (0, 0));
                }
                let h = Integer::from(&i / &b).to_u128().unwrap();
                let l = Integer::from(&i % &b).to_u128().unwrap();
                (s, e, (h, l))
            }
            _ => panic!("Value is NaN or infinite."),
        }
    }

    pub fn random_from_exp_range(exp_range: &RangeInclusive<i32>) -> Self {
        const HI_HIDDEN_BIT: u128 = 1_u128 << 108;
        const HI_MAX: u128 = HI_HIDDEN_BIT - 1;
        let mut rng = thread_rng();
        let s = rng.gen_range(0..=1_u32);
        let mut t: i32 = rng.gen_range(exp_range.clone());
        let mut h = rng.gen_range(0..=HI_MAX);
        let l = rng.gen_range(0..=u128::MAX);
        let mut prec = P;
        if t >= EMIN {
            t -= PM1;
            h += HI_HIDDEN_BIT;
        } else {
            let msb = if h != 0 {
                128 - h.leading_zeros()
            } else {
                256 - l.leading_zeros()
            };
            prec = msb;
        }
        let mut c = (Integer::from(h) << 128) + l;
        let (mut f, o) = if t < 0 {
            let mut p = Float::new(P);
            p.assign(Float::i_exp(2, t));
            let (fr, o) =
                Float::with_val_round(prec, &c * &p, Round::Nearest);
            (Float::with_val(P, fr), o)
        } else {
            let p = Integer::from(2).pow(t as u32);
            c *= p;
            Float::with_val_round(P, &c, Round::Nearest)
        };
        if s == 1 {
            f = -f;
        }
        Self { f, o }
    }
}

impl FromStr for FP237 {
    type Err = ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match Float::parse(s) {
            Ok(p) => {
                let (mut f, mut o) =
                    Float::with_val_round(P, p, Round::Nearest);
                o = f.subnormalize_ieee_round(o, Round::Nearest);
                Ok(Self { f, o })
            }
            Err(e) => Err(e),
        }
    }
}

impl Display for FP237 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.f.is_integer() {
            let mut i = self.f.to_integer().unwrap();
            let mut s = i.to_string();
            let n = s.len() as u32;
            if n > 72 {
                let d = Integer::from(10).pow(n - 72);
                let mut t = Integer::new();
                t.assign(&d >> 1);
                let qr = i.div_rem_ref(&d);
                let mut q = Integer::new();
                let mut r = Integer::new();
                (&mut q, &mut r).assign(qr);
                if r > t || r == t && q.is_odd() {
                    q += 1;
                }
                i.assign(q * &d);
                s = i.to_string();
            }
            f.write_str(&s)
        } else {
            Display::fmt(&self.f, f)
        }
    }
}

impl LowerExp for FP237 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        LowerExp::fmt(&self.f, f)
    }
}

impl PartialOrd for FP237 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.f.partial_cmp(&other.f)
    }
}

impl Add for &FP237 {
    type Output = FP237;

    fn add(self, rhs: Self) -> Self::Output {
        let f = &self.f + &rhs.f;
        let (f, o) = Float::with_val_round(P, f, Round::Nearest);
        Self::Output { f, o }
    }
}

impl Sub for &FP237 {
    type Output = FP237;

    fn sub(self, rhs: Self) -> Self::Output {
        let f = &self.f - &rhs.f;
        let (f, o) = Float::with_val_round(P, f, Round::Nearest);
        Self::Output { f, o }
    }
}

impl Mul for &FP237 {
    type Output = FP237;

    fn mul(self, rhs: Self) -> Self::Output {
        let f = &self.f * &rhs.f;
        let (f, o) = Float::with_val_round(P, f, Round::Nearest);
        Self::Output { f, o }
    }
}

impl Div for &FP237 {
    type Output = FP237;

    fn div(self, rhs: Self) -> Self::Output {
        let f = &self.f / &rhs.f;
        let (f, o) = Float::with_val_round(P, f, Round::Nearest);
        Self::Output { f, o }
    }
}

impl Rem for &FP237 {
    type Output = FP237;

    fn rem(self, rhs: Self) -> Self::Output {
        let f = &self.f % &rhs.f;
        let (f, o) = Float::with_val_round(P, f, Round::Nearest);
        Self::Output { f, o }
    }
}

#[cfg(test)]
mod decode_tests {
    use super::*;

    #[test]
    fn test_from_str() {
        let s = "17.625";
        let f = FP237::from_str(s).unwrap();
        // println!("{}", f);
        assert_eq!(f.decode(true), (0, -3, (0, 141)));
    }

    #[test]
    fn test_min_pos_subnormal() {
        let e = Float::with_val(P, Float::parse("-262378.").unwrap());
        let t = e.exp2();
        let f = FP237 {
            f: t.clone(),
            o: Ordering::Equal,
        };
        assert_eq!(f.decode(true), (0, -262378, (0, 1)));
    }

    #[test]
    fn test_subnormal() {
        let s = "-0.9818036132127703363504450836394764653184121e-78913";
        let f = FP237::from_str(s).unwrap();
        assert_eq!(
            f.decode(true),
            (
                1,
                -262378,
                (
                    128347527004149295075436743924545,
                    200698461692417807477600193256349332369
                )
            )
        );
    }

    #[test]
    fn test_subnormal_near_zero() {
        let s = "-21.75e-78985";
        let f = FP237::from_str(s).unwrap();
        assert_eq!(f.decode(true), (1, -262378, (0, 1)));
    }

    #[test]
    fn test_max() {
        let e = Float::with_val(P, Float::parse("262144.").unwrap());
        let a = e.exp2();
        let e = Float::with_val(P, Float::parse("261907.").unwrap());
        let b = e.exp2();
        let t = a - b;
        let f = FP237 {
            f: t.clone(),
            o: Ordering::Equal,
        };
        // println!("{f}");
        assert_eq!(
            f.decode(true),
            (0, 261907, ((1 << (237 - 128)) - 1, u128::MAX))
        );
    }

    #[test]
    fn test_normal_gt1() {
        let s = "320.1000009";
        let f = FP237::from_str(s).unwrap();
        // println!("{}", f);
        assert_eq!(
            f.decode(true),
            (
                0,
                -228,
                (
                    405774958273941771624501157387890,
                    164981958326160731124041003267846608813
                )
            )
        );
    }

    #[test]
    fn test_normal_2_pow_275() {
        const HI_HIDDEN_BIT: u128 = 1_u128 << 108;
        let t = 275;
        let mut c = Integer::from(HI_HIDDEN_BIT) << 128;
        let p = Integer::from(2).pow(t as u32);
        c *= p;
        // println!("{c}");
        let (f, o) = Float::with_val_round(P, &c, Round::Nearest);
        // println!("{f:.0}");
        let f = FP237 { f, o };
        assert_eq!(f.f.prec(), P);
        let (s, e, (h, _)) = f.decode(false);
        assert!(s == 0 || s == 1);
        assert_eq!(h.leading_zeros(), (256 - P));
        assert_eq!(e, 275);
    }

    #[test]
    fn test_normal_1() {
        let (e, (h, l)) = (
            -131198,
            (
                531439310060859797527772502089072_u128,
                50579904501390594736454450167121974314_u128,
            ),
        );
        let mut m = Integer::from(h);
        m <<= 128;
        m += Integer::from(l);
        let t = Float::with_val(P, e).exp2();
        let f = FP237 {
            f: m * t,
            o: Ordering::Equal,
        };
        assert_eq!(f.decode(false), (0_u32, e, (h, l)));
        println!("{:?}", f.decode(false));
        println!("{:?}", f.decode(true));
    }
}

#[cfg(test)]
mod rnd_tests {
    use super::*;

    #[test]
    fn test_normal_lt1() {
        let exp_range: RangeInclusive<i32> = -304..=-236;
        let f = FP237::random_from_exp_range(&exp_range);
        assert_eq!(f.f.prec(), P);
        let (s, e, (h, _)) = f.decode(true);
        assert!(s == 0 || s == 1);
        assert!(exp_range.contains(&(e + PM1)));
        assert!(h.leading_zeros() >= 256 - P);
    }

    #[test]
    fn test_normal_2_pow_275() {
        let exp_range: RangeInclusive<i32> = 275..=275;
        let f = FP237::random_from_exp_range(&exp_range);
        assert_eq!(f.f.prec(), P);
        let (s, e, (h, _)) = f.decode(false);
        assert!(s == 0 || s == 1);
        assert_eq!(h.leading_zeros(), (256 - P));
        assert_eq!(e + PM1, 275);
    }
}

#[cfg(test)]
mod const_tests {
    use super::*;

    #[test]
    fn show_consts() {
        let c = FP237::Log2();
        println!("Log2:\n{c} = {:?}", c.decode(true));
        let c = FP237::Pi();
        println!("Pi:\n{c} = {:?}", c.decode(true));
        let c = FP237::Euler();
        println!("Euler:\n{c} = {:?}", c.decode(true));
        let c = FP237::Catalan();
        println!("Catalan:\n{c} = {:?}", c.decode(true));
    }
}

#[cfg(test)]
mod add_sub_tests {
    use rug::{ops::CompleteRound, Complete};

    use super::*;

    #[test]
    fn test_add() {
        let c = Integer::parse(
            "220855883097298041197912187593213263622162528096038865363210905810239470")
            .unwrap()
            .complete();
        let e = Float::parse("-376").unwrap().complete(P);
        let t = e.exp2();
        let (f, o) = Float::with_val_round(P, &t * &c, Round::Nearest);
        let x = FP237 { f, o };
        println!("{:?}", x.decode(false));
        let e = Float::parse("-376").unwrap().complete(P);
        let t = e.exp2();
        let (f, o) = Float::with_val_round(P, &t, Round::Nearest);
        let y = FP237 { f, o };
        println!("{:?}", y.decode(false));
        let z = &x + &y;
        println!("{:?}", z.decode(false));
        assert_eq!(
            z.decode(false),
            (0, -375, (324518553658426726783156020576768, 65528))
        );
    }

    #[test]
    fn test_sub() {
        let mut c = Integer::parse(
            "207052390403716913623049481515649182342802536657260232019104846713985880")
            .unwrap()
            .complete();
        let e = Float::parse("-262376").unwrap().complete(P);
        let t = e.exp2();
        let (f, o) = Float::with_val_round(P, &t * &c, Round::Nearest);
        let y = FP237 { f, o };
        println!("{:?}", y.decode(false));
        c += 1;
        let (f, o) = Float::with_val_round(P, &t * &c, Round::Nearest);
        let x = FP237 { f, o };
        println!("{:?}", x.decode(false));
        let z = &x - &y;
        println!("{:?}", z.decode(true));
        assert_eq!(z.decode(true), (0, -262376, (0, 1)));
    }

    #[test]
    fn test_subnormal_add_subnormal_giving_subnormal() {
        let e = Float::parse("-262378").unwrap().complete(P);
        let t = e.exp2();
        let c = Integer::parse(
            "2555352441053941610851869298425305547681183005143821848",
        )
        .unwrap()
        .complete();
        let (f, o) = Float::with_val_round(P, &t * &c, Round::Nearest);
        let x = FP237 { f, o };
        println!("{:?}", x.decode(false));
        let c = Integer::parse("21747048302197486").unwrap().complete();
        let (f, o) = Float::with_val_round(P, &t * &c, Round::Nearest);
        let y = FP237 { f, o };
        println!("{:?}", y.decode(false));
        let z = &x + &y;
        println!("{:?}", z.decode(false));
        assert_eq!(
            z.decode(false),
            (
                0,
                -262378,
                (7509505897047126, 339876022494367207146887125588680943878)
            )
        );
    }
}

#[cfg(test)]
mod mul_tests {
    use rug::{ops::CompleteRound, Complete};

    use super::*;

    #[test]
    fn test_normal() {
        let c = Integer::parse("555").unwrap().complete();
        let e = Float::parse("-23718").unwrap().complete(P);
        let t = e.exp2();
        let (f, o) = Float::with_val_round(P, &t * &c, Round::Nearest);
        let x = FP237 { f, o };
        println!("{:?}", x.decode(false));
        let c = Integer::parse("21747048302197486").unwrap().complete();
        let e = Float::parse("29").unwrap().complete(P);
        let t = e.exp2();
        let (f, o) = Float::with_val_round(P, &t * &c, Round::Nearest);
        let y = FP237 { f, o };
        println!("{:?}", y.decode(false));
        let z = &x * &y;
        println!("{:?}", z.decode(false));
        assert_eq!(
            z.decode(false),
            ((0, -23862, (424661712810566800616627487375360, 0)))
        );
    }

    #[test]
    fn test_overflow() {
        let e = Float::parse("262140").unwrap().complete(P);
        let t = e.exp2();
        let (f, o) = Float::with_val_round(P, &t, Round::Nearest);
        let x = FP237 { f, o };
        println!("{:?}", x.decode(false));
        let e = Float::parse("4").unwrap().complete(P);
        let t = e.exp2();
        let (f, o) = Float::with_val_round(P, &t, Round::Nearest);
        let y = FP237 { f, o };
        println!("{:?}", y.decode(false));
        let z = &x * &y;
        println!("{:?}", z.decode(false));
        assert_eq!(z.decode(false), ((0, 262144, (0, 0))));
    }
}

#[cfg(test)]
mod div_tests {
    use rug::{ops::CompleteRound, Complete};

    use super::*;

    #[test]
    fn test_normal() {
        let c = Integer::parse("555").unwrap().complete();
        let e = Float::parse("-23718").unwrap().complete(P);
        let t = e.exp2();
        let (f, o) = Float::with_val_round(P, &t * &c, Round::Nearest);
        let x = FP237 { f, o };
        // println!("{:?}", x.decode(false));
        let c = Integer::parse("7777").unwrap().complete();
        let e = Float::parse("-23720").unwrap().complete(P);
        let t = e.exp2();
        let (f, o) = Float::with_val_round(P, &t * &c, Round::Nearest);
        let y = FP237 { f, o };
        // println!("{:?}", y.decode(false));
        let z = &x / &y;
        // println!("{:?}", z.decode(true));
        assert_eq!(
            z.decode(false),
            ((
                0,
                -238,
                (
                    370544523143478119304928052297435,
                    56225130705079841269183023087285862379
                )
            ))
        );
    }

    #[test]
    fn test_overflow() {
        let e = Float::parse("262140").unwrap().complete(P);
        let t = e.exp2();
        let (f, o) = Float::with_val_round(P, &t, Round::Nearest);
        let x = FP237 { f, o };
        println!("{:?}", x.decode(false));
        let e = Float::parse("4").unwrap().complete(P);
        let t = e.exp2();
        let (f, o) = Float::with_val_round(P, &t, Round::Nearest);
        let y = FP237 { f, o };
        println!("{:?}", y.decode(false));
        let z = &x * &y;
        println!("{:?}", z.decode(false));
        assert_eq!(z.decode(false), ((0, 262144, (0, 0))));
    }
}

#[cfg(test)]
mod rem_tests {
    use rug::ops::CompleteRound;

    use super::*;

    #[test]
    fn test_normal_1() {
        let (f, o) = Float::with_val_round(P, 3.297338e302, Round::Nearest);
        let x = FP237 { f, o };
        let (f, o) = Float::with_val_round(P, 1.008297e-297, Round::Nearest);
        let y = FP237 { f, o };
        let (f, o) =
            Float::with_val_round(P, 7.06898110067969e-298, Round::Nearest);
        let z = FP237 { f, o };
        // println!("{x:e} % {y:e} = {z:e}");
        assert_eq!((&x % &y).f, z.f);
    }

    #[test]
    fn test_normal_2() {
        let f = Float::parse("1.4e78118").unwrap().complete(P);
        let x = FP237 {
            f,
            o: Ordering::Equal,
        };
        let f = Float::parse("1.7e-25009").unwrap().complete(P);
        let y = FP237 {
            f,
            o: Ordering::Equal,
        };
        let z = &x % &y;
        println!("{x:e} % {y:e} = {z:e}");
        assert_eq!(z.f, x.f % y.f);
    }
}

#[cfg(test)]
mod sqrt_tests {
    use rug::{ops::CompleteRound, Complete};

    use super::*;

    fn print_test_item(x: &FP237, z: &FP237) {
        let rx = x.decode(true);
        let rz = z.decode(true);
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            rx.0, rx.1, rx.2 .0, rx.2 .1, rz.0, rz.1, rz.2 .0, rz.2 .1,
        );
    }

    #[test]
    fn test_normal_1() {
        let (f, o) = Float::with_val_round(P, 7., Round::Nearest);
        let x = FP237 { f, o };
        let z = x.clone().sqrt();
        println!("√{x:e} = {z:e}");
        println!("{:?}", x.decode(false));
        println!("{:?}", z.decode(false));
        assert_eq!(z.f, x.f.clone().sqrt());
    }

    #[test]
    fn test_normal_2() {
        let c = Integer::parse("73913349228891354865085158512847")
            .unwrap()
            .complete();
        let e = Float::parse("-262021").unwrap().complete(P);
        let t = e.exp2();
        let (f, o) = Float::with_val_round(P, &t * &c, Round::Nearest);
        let x = FP237 { f, o };
        let z = x.clone().sqrt();
        println!("√{x:e} = {z:e}");
        println!("{:?}", x.decode(true));
        println!("{:?}", z.decode(true));
        assert_eq!(z.f, x.f.clone().sqrt());
    }

    #[test]
    fn test_normal_3() {
        let (e, (h, l)) =
            (-262030, (0_u128, 217574416850779740425388533631545_u128));
        let mut m = Integer::from(h);
        m <<= 128;
        m += Integer::from(l);
        let t = Float::with_val(P, e).exp2();
        let f = FP237 {
            f: m * t,
            o: Ordering::Equal,
        };
        let r = f.clone().sqrt();
        println!("√{f:e} = {r:e}");
        println!("{:?}", f.decode(true));
        println!("{:?}", f.decode(false));
        println!("{:?}", r.decode(true));
        println!("{:?}", r.decode(false));
        print_test_item(&f, &r);
        assert_eq!(r.f, f.f.clone().sqrt());
    }

    #[test]
    fn test_normal_4() {
        let (e, (h, l)) =
            (-262234, (0_u128, 201645646862054831249887812997549_u128));
        let mut m = Integer::from(h);
        m <<= 128;
        m += Integer::from(l);
        let t = Float::with_val(P, e).exp2();
        let f = FP237 {
            f: m * t,
            o: Ordering::Equal,
        };
        let r = f.clone().sqrt();
        println!("√{f:e} = {r:e}");
        println!("{:?}", f.decode(true));
        println!("{:?}", f.decode(false));
        println!("{:?}", r.decode(true));
        println!("{:?}", r.decode(false));
        print_test_item(&f, &r);
        assert_eq!(r.f, f.f.clone().sqrt());
    }

    #[test]
    fn test_subnormal_1() {
        let c = Integer::parse("6864413414364755142662776660834762333374747222713421199918761912559").unwrap().complete();
        let e = Float::parse("-262375").unwrap().complete(P);
        let t = e.exp2();
        let (f, o) = Float::with_val_round(P, &t * &c, Round::Nearest);
        let x = FP237 { f, o };
        let z = x.clone().sqrt();
        println!("√{x:e} = {z:e}");
        println!("{:?}", x.decode(true));
        println!("{:?}", z.decode(true));
        assert_eq!(z.f, x.f.clone().sqrt());
    }
}

#[cfg(test)]
mod fma_tests {
    use rug::ops::CompleteRound;

    use super::*;

    #[test]
    fn test_no_diff_to_non_fused() {
        let t = Float::parse("1.5").unwrap().complete(P);
        let (f, o) = Float::with_val_round(P, &t, Round::Nearest);
        let one_and_a_half = FP237 { f, o };
        let t = Float::parse("2.0").unwrap().complete(P);
        let (f, o) = Float::with_val_round(P, &t, Round::Nearest);
        let two = FP237 { f, o };
        let t = Float::parse("3.0").unwrap().complete(P);
        let (f, o) = Float::with_val_round(P, &t, Round::Nearest);
        let three = FP237 { f, o };
        assert_eq!(
            one_and_a_half.fma(&two, &FP237::Pi()),
            &FP237::Pi() + &three
        );
    }

    #[test]
    fn test_small_diff_to_non_fused() {
        let t = Float::parse("1.0").unwrap().complete(P);
        let (f, o) = Float::with_val_round(P, &t, Round::Nearest);
        let one = FP237 { f, o };
        let e = Float::parse("-237").unwrap().complete(P);
        let t = e.exp2();
        let (f, o) = Float::with_val_round(P, &t, Round::Nearest);
        let d = FP237 { f, o };
        let x = &one - &d;
        let y = &x;
        let a = &(&d + &d) - &one;
        let z = x.fma(y, &a);
        // println!(
        //     " d: {d:e}\n x: {x:e}\n y: {y:e}\nxy: {:e}\n a: {a:e}\n z: \
        //      {z:e}\n r: {:e}",
        //     &(&x * y),
        //     &(&x * y) + &a
        // );
        assert_eq!(z, &d * &d);
    }

    #[test]
    fn test_product_near_inf() {
        let e = Float::parse("59475").unwrap().complete(P);
        let t = e.exp2();
        let m = Float::parse("-40901480905045544498406800675204866629143691002339835783757486257606735").unwrap().complete(P);
        let (f, o) = Float::with_val_round(P, &m * &t, Round::Nearest);
        let x = FP237 { f, o };
        println!("{:?}", x.decode(true));
        let e = Float::parse("202197").unwrap().complete(P);
        let t = e.exp2();
        let m = Float::parse("190854343998886546791476171399145128666427780394331938864477511960147119").unwrap().complete(P);
        let (f, o) = Float::with_val_round(P, &m * &t, Round::Nearest);
        let y = FP237 { f, o };
        println!("{:?}", y.decode(true));
        let e = Float::parse("-7930").unwrap().complete(P);
        let t = e.exp2();
        let m = Float::parse("-12842618913023200758447616413804922508126617643079319769168854255306905").unwrap().complete(P);
        let (f, o) = Float::with_val_round(P, &m * &t, Round::Nearest);
        let a = FP237 { f, o };
        println!("{:?}", a.decode(true));
        let z = x.fma(&y, &a);
        println!("{:?}", z.decode(true));
        println!(
            " x: {x:e}\n y: {y:e}\nxy: {:e}\n a: {a:e}\n z: {z:e}\n r: {:e}",
            &(&x * &y),
            &(&x * &y) + &a
        );
    }
}

#[cfg(test)]
mod sos_tests {
    use rug::ops::CompleteRound;

    use super::*;

    #[test]
    fn test_small_diff_to_non_fused() {
        let t = Float::parse("1.0").unwrap().complete(P);
        let (f, o) = Float::with_val_round(P, &t, Round::Nearest);
        let one = FP237 { f, o };
        let e = Float::parse("-117").unwrap().complete(P);
        let t = e.exp2();
        let (f, o) = Float::with_val_round(P, &t, Round::Nearest);
        let d = FP237 { f, o };
        let d2 = &d * &d;
        let x = &one + &d;
        let y = &one - &d;
        let z = x.sos(&y);
        let mut t = (&d.f * &d.f).complete(2 * P + 2);
        t *= 2;
        t += 2;
        let (f, o) = Float::with_val_round(P, t, Round::Nearest);
        let r = FP237 { f, o };
        println!(" d: {:?}\nd2: {:?}", d.decode(false), d2.decode(false));
        println!(" 1: {:?}", one.decode(false));
        println!(" x: {:?}\n y: {:?}", x.decode(false), y.decode(false));
        println!(
            "x2: {:?}\ny2: {:?}",
            &(&x * &x).decode(false),
            &(&y * &y).decode(false)
        );
        println!(" z: {:?}\n r: {:?}", z.decode(false), r.decode(false));
        // println!(
        //     " d: {d:?}\n x: {x:?}\n y: {y:?}\n z: {z:?}\n r: {r:?},\n f: \
        //      {:?}",
        //     &two + &(&d2 + &d2)
        // );
        assert_eq!(z.f, r.f);
    }
}

#[cfg(test)]
mod sin_tests {
    use super::*;

    #[test]
    fn test_small_val() {
        let s = "2.4172956479058681897522768256558457838";
        let (f, o) = Float::with_val_round(
            P,
            Float::parse(s).unwrap(),
            Round::Nearest,
        );
        let a = FP237 { f, o };
        let ph = Float::with_val(250, Constant::Pi).div(2);
        let ad = Float::with_val(250, a.clone().f);
        // let ph = FP237::Pi().div(&FP237::from_str("2.0").unwrap());
        let rd = ad % ph;
        let r = FP237 {
            f: Float::with_val(P, rd),
            o: Ordering::Equal,
        };
        println!("{:?}", a.decode(false));
        println!("{:?}", r.decode(false));
        println!("{:?}", a.sin().decode(false));
        println!("{:?}", r.cos().decode(false));
        // println!("{:e}\n{:e}\n{:e}", a.f, ph.f, r.f);
        let c = Integer::from(2).div(Float::with_val(250, Constant::Pi));
        let (f, o) = Float::with_val_round(P, &c, Round::Nearest);
        let ch = FP237 { f, o };
        let (f, o) = Float::with_val_round(P, &c - &ch.f, Round::Nearest);
        let cl = FP237 { f, o };
        println!("{}", c);
        println!("{}", ch.f);
        println!("{}", cl.f);
        println!("{:?}", ch.decode(true));
        println!("{:?}", cl.decode(true));
    }

    #[test]
    fn test_large_val() {
        let s = "3.1172956479058681897522768256558457838091042210721692171453017613851364e59";
        let (f, o) = Float::with_val_round(
            P,
            Float::parse(s).unwrap(),
            Round::Nearest,
        );
        let a = FP237 { f, o };
        println!("{:?}", a.decode(false));
        println!("{:?}", a.sin().decode(true));
        let ph = FP237::Pi().div(&FP237::from_str("2.0").unwrap());
        let r = &a % &ph;
        println!("{:?}", r.decode(false));
        println!("{:?}", r.sin().decode(true));
    }
}
