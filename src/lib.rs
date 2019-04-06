#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Q15(i16);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Q31(i32);

impl Q15 {
    pub const ULP: Self = Q15(1);

    pub fn from_float(mut f: f32) -> Self {
        f *= 32768.0;
        if f > 32767.0 {
            f = 32767.0;
        } else if f < -32768.0 {
            f = -32768.0;
        }

        f = f.round();

        Q15(f as i16)
    }

    pub fn exact_mul(self, rhs: Self) -> Q31 {
        let a = i32::from(self.0);
        let b = i32::from(rhs.0);

        let c = a * b;

        Q31(c)
    }

    const ROUND: i32 = 1 << 14;
    fn sat(x: i32) -> Self {
        if x > 0x7FFF {
            Q15(32767)
        } else if x < -0x8000 {
            Q15(-32768)
        } else {
            Q15(x as i16)
        }
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl std::ops::Mul for Q15 {
    type Output = Q15;

    fn mul(self, rhs: Self) -> Self::Output {
        let a = i32::from(self.0);
        let b = i32::from(rhs.0);

        let c = a * b;
        let c = c + Q15::ROUND;
        let c = c >> 15;

        Q15::sat(c)
    }
}

impl std::ops::Add for Q15 {
    type Output = Q15;

    fn add(self, rhs: Self) -> Self::Output {
        Q15(self.0.saturating_add(rhs.0))
    }
}

impl std::ops::Sub for Q15 {
    type Output = Q15;

    fn sub(self, rhs: Self) -> Self::Output {
        Q15(self.0.saturating_sub(rhs.0))
    }
}

impl std::ops::Neg for Q15 {
    type Output = Q15;

    fn neg(self) -> Self {
        Q15(0i16.saturating_sub(self.0))
    }
}

impl std::fmt::Display for Q15 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", f32::from(*self))
    }
}

impl From<Q15> for f32 {
    fn from(x: Q15) -> f32 {
        let f = f32::from(x.0 as i16);
        f / 32768.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn normalize(x: f32) -> f32 {
        f32::from(Q15::from_float(x))
    }

    fn close(x: Q15, mut y: f32) -> bool {
        if y <= -1.0 {
            y = -1.0;
        } else if y >= 0.999_969_5 {
            y = 0.999_969_5;
        }
        (f32::from(x) - y).abs() < f32::from(Q15::ULP)
    }

    #[test]
    fn mult_boundaries() {
        let a = Q15::from_float(1.0);
        let b = Q15::from_float(-1.0);

        assert_eq!(a * b, -a);
        assert_eq!(a * a, Q15::from_float(f32::from(a) * f32::from(a)));
        assert_eq!(b * b, a);
    }

    #[test]
    fn neg_min() {
        let a = Q15::from_float(-1.0);
        let b = Q15::from_float(1.0);

        assert_eq!(-a, b);
    }

    proptest! {
        #[test]
        fn from_to_id(x: i16) {
            let a = Q15(x);
            assert_eq!(a, Q15::from_float(f32::from(a)));
        }

        #[test]
        fn mult(a in -1.0f32..0.999_969_5, b in -1.0f32..0.999_969_5) {
            let qa = Q15::from_float(a);
            let qb = Q15::from_float(b);

            assert!(close(qa*qb, normalize(a)*normalize(b)));
        }

        #[test]
        fn add(a in -1.0f32..0.999_969_5, b in -1.0f32..0.999_969_5) {
            let qa = Q15::from_float(a);
            let qb = Q15::from_float(b);

            assert!(close(qa+qb, normalize(a) + normalize(b)));
        }
    }
}
