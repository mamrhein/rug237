// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use rand::prelude::*;
use rug::float::{Constant, ParseFloatError, Round};
use rug::ops::Pow;
use rug::{Assign, Float, Integer};
use std::cmp::Ordering;
use std::fmt::{Display, Formatter, LowerExp};
use std::ops::{Add, Div, Mul, RangeInclusive, Rem, Sub};
use std::str::FromStr;

pub const P: u32 = 237;
pub const PM1: i32 = P as i32 - 1;
pub const EMAX: i32 = 262143;
pub const EMIN: i32 = 1 - EMAX;
pub const MIN_EXP_SUBNORMAL: i32 = EMIN - PM1;

#[derive(Clone)]
pub struct FP237 {
    pub(crate) f: Float,
    pub(crate) o: Ordering,
}

impl FP237 {
    #[allow(non_snake_case)]
    pub fn Log2() -> Self {
        FP237 {
            f: Float::with_val(P, Constant::Log2),
            o: Ordering::Equal,
        }
    }

    #[allow(non_snake_case)]
    pub fn Pi() -> Self {
        FP237 {
            f: Float::with_val(P, Constant::Pi),
            o: Ordering::Equal,
        }
    }

    #[allow(non_snake_case)]
    pub fn Euler() -> Self {
        FP237 {
            f: Float::with_val(P, Constant::Euler),
            o: Ordering::Equal,
        }
    }

    #[allow(non_snake_case)]
    pub fn Catalan() -> Self {
        FP237 {
            f: Float::with_val(P, Constant::Catalan),
            o: Ordering::Equal,
        }
    }

    pub fn trunc(&self) -> Self {
        FP237 {
            f: self.f.clone().trunc(),
            o: Ordering::Equal,
        }
    }

    pub fn decode(&self, reduce: bool) -> (u32, i32, (u128, u128)) {
        let b: Integer = Integer::from(u128::MAX) + 1;
        match self.f.to_integer_exp() {
            Some((mut i, mut e)) => {
                let s = self.f.is_sign_negative() as u32;
                i.abs_mut();
                if reduce && i != 0 {
                    while i.is_even() {
                        i >>= 1;
                        e += 1;
                    }
                }
                if e > EMAX - PM1 {
                    return (s, EMAX + 1, (0, 0));
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
            Float::with_val_round(prec, &c * &p, Round::Nearest)
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
    use super::*;
    use rug::ops::CompleteRound;
    use rug::Complete;

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
    use super::*;
    use rug::ops::CompleteRound;
    use rug::Complete;

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
    use super::*;
    use rug::ops::CompleteRound;
    use rug::Complete;

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
            ((0, -238, (370544523143478119304928052297435, 56225130705079841269183023087285862379)))
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
        let (f,o) = Float::with_val_round(P, 7.06898110067969e-298, Round::Nearest);
        let z = FP237 { f, o };
        // println!("{x:e} % {y:e} = {z:e}");
        assert_eq!((&x % &y).f, z.f);
    }

    #[test]
    fn test_normal_2() {
        let f = Float::parse("1.4e78118").unwrap().complete(P);
        let x = FP237 { f, o: Ordering::Equal };
        let f = Float::parse("1.7e-25009").unwrap().complete(P);
        let y = FP237 { f, o: Ordering::Equal };
        let z = &x % &y;
        println!("{x:e} % {y:e} = {z:e}");
        assert_eq!(z.f, x.f % y.f);
    }
}
