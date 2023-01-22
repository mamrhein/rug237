// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use rand::prelude::*;
use rug::float::{ParseFloatError, Round};
use rug::ops::Pow;
use rug::{Assign, Float, Integer};
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::ops::RangeInclusive;
use std::str::FromStr;

pub const P: u32 = 237;
pub const EMAX: i32 = 262143;
pub const EMIN: i32 = 1 - EMAX;
pub const MIN_EXP_SUBNORMAL: i32 = EMIN - P as i32 + 1;

pub struct FP237 {
    pub(crate) f: Float,
    pub(crate) o: Ordering,
}

impl FP237 {
    pub fn decode(&self) -> (u32, i32, (u128, u128)) {
        let b: Integer = Integer::from(u128::MAX) + 1;
        match self.f.to_integer_exp() {
            Some((mut i, mut e)) => {
                let s = self.f.is_sign_negative() as u32;
                i.abs_mut();
                while i.is_even() {
                    i >>= 1;
                    e += 1;
                }
                if e > EMAX {
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
                    return (s, 0, (0, 0));
                }
                let h = Integer::from(&i / &b).to_u128().unwrap();
                let l = Integer::from(&i % &b).to_u128().unwrap();
                (s, e, (h, l))
            }
            _ => panic!("Value is NaN or infinite."),
        }
    }

    pub fn rnd_from_exp_range(exp_range: &RangeInclusive<i32>) -> Self {
        let mut rng = thread_rng();
        let s = rng.gen_range(0..=1_u32);
        let t: i32 = rng.gen_range(exp_range.clone());
        let h = rng.gen_range(0..=P - 128);
        let l = rng.gen_range(0..=u128::MAX);
        let mut c = (Integer::from(h) << 128) + l;
        if t >= EMIN {
            c += Integer::from(1) << (P - 1);
        }
        let mut p = Float::new(256);
        p.assign(Float::i_exp(2, t));
        let (mut f, o) = Float::with_val_round(P, &c * &p, Round::Nearest);
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
            self.f.fmt(f)
        }
    }
}

#[cfg(test)]
mod decode_tests {
    use super::*;

    #[test]
    fn test_from_str() {
        let s = "17.625";
        let f = FP237::from_str(s).unwrap();
        println!("{}", f);
        assert_eq!(f.decode(), (0, -3, (0, 141)));
    }

    #[test]
    fn test_min_pos_subnormal() {
        let e = Float::with_val(P, Float::parse("-262378.").unwrap());
        let t = e.exp2();
        let f = FP237 {
            f: t.clone(),
            o: Ordering::Equal,
        };
        assert_eq!(f.decode(), (0, -262378, (0, 1)));
    }

    #[test]
    fn test_subnormal() {
        let s = "-0.9818036132127703363504450836394764653184121e-78913";
        let f = FP237::from_str(s).unwrap();
        assert_eq!(
            f.decode(),
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
        assert_eq!(f.decode(), (1, -262378, (0, 1)));
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
        println!("{f}");
        assert_eq!(
            f.decode(),
            (0, 261907, ((1 << (237 - 128)) - 1, u128::MAX))
        );
    }

    #[test]
    fn test_normal_gt1() {
        let s = "320.1000009";
        let f = FP237::from_str(s).unwrap();
        println!("{}", f);
        assert_eq!(f.decode(), (0, -3, (0, 141)));
    }
}

#[cfg(test)]
mod rnd_tests {
    use super::*;

    #[test]
    fn test_normal_lt1() {
        let exp_range: RangeInclusive<i32> = -304..=-236;
        let f = FP237::rnd_from_exp_range(&exp_range);
        assert_eq!(f.f.prec(), P);
        let (s, e, (h, _)) = f.decode();
        assert!(s == 0 || s == 1);
        assert!(exp_range.contains(&e));
        assert!(h.leading_zeros() >= 256 - P);
    }
}
