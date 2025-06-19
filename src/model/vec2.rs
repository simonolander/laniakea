use crate::model::position::Position;
use std::f64::consts::PI;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, Neg, Sub};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    x: f64,
    y: f64,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };

    pub fn new(x: f64, y: f64) -> Vec2 {
        Vec2 { x, y }
    }

    pub fn is_zero(&self) -> bool {
        self == &Self::ZERO
    }

    pub fn length_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y
    }

    pub fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }

    pub fn normalize(&mut self) {
        let length = self.length();
        if length != 0.0 {
            *self /= length
        }
    }

    pub fn normalized(&self) -> Vec2 {
        let mut clone = self.clone();
        clone.normalize();
        clone
    }

    pub fn angle(&self) -> f64 {
        self.y.atan2(self.x)
    }

    /// Returns the angle from this vector to another given vector.
    /// Angles are normalized to be in the range -π (inclusive) to +π (exclusive).
    /// Returns zero if either vector has length zero.
    pub fn angle_to(&self, other: &Vec2) -> f64 {
        if self.is_zero() || other.is_zero() {
            0.0
        } else {
            let angle = other.y.atan2(other.x) - self.y.atan2(self.x);
            if angle > PI {
                angle - 2.0 * PI
            } else if angle <= -PI {
                angle + 2.0 * PI
            } else {
                angle
            }
        }
    }
}

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x + other.x, self.y + other.y)
    }
}

impl AddAssign<Vec2> for Vec2 {
    fn add_assign(&mut self, rhs: Vec2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x - other.x, self.y - other.y)
    }
}

impl Mul<f64> for Vec2 {
    type Output = Vec2;
    fn mul(self, other: f64) -> Vec2 {
        Vec2::new(self.x * other, self.y * other)
    }
}

impl Div<f64> for Vec2 {
    type Output = Vec2;
    fn div(self, other: f64) -> Vec2 {
        Vec2::new(self.x / other, self.y / other)
    }
}

impl DivAssign<f64> for Vec2 {
    fn div_assign(&mut self, rhs: f64) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl Neg for Vec2 {
    type Output = Vec2;
    fn neg(self) -> Vec2 {
        Vec2::new(-self.x, -self.y)
    }
}

impl From<(f64, f64)> for Vec2 {
    fn from((x, y): (f64, f64)) -> Self {
        Vec2::new(x, y)
    }
}

impl From<&Position> for Vec2 {
    fn from(p: &Position) -> Self {
        Vec2::new(p.column as f64, p.row as f64)
    }
}

impl Sum<Vec2> for Vec2 {
    fn sum<I: Iterator<Item = Vec2>>(iter: I) -> Self {
        iter.fold(Self::ZERO, |a, b| a + b)
    }
}

#[cfg(test)]
mod tests {
    use crate::model::vec2::Vec2;
    use proptest::arbitrary::Arbitrary;
    use proptest::prelude::{BoxedStrategy, Strategy};
    use std::ops::Range;

    #[derive(Debug, Clone, PartialEq)]
    pub struct Vec2Parameters {
        pub x_range: Range<f64>,
        pub y_range: Range<f64>,
        pub non_zero: bool,
    }

    impl Default for Vec2Parameters {
        fn default() -> Self {
            Vec2Parameters {
                x_range: -100.0..100.0,
                y_range: -100.0..100.0,
                non_zero: false,
            }
        }
    }

    impl Arbitrary for Vec2 {
        type Parameters = Vec2Parameters;

        fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
            (params.x_range, params.y_range)
                .prop_map(Vec2::from)
                .prop_filter("Must not be zero", move |v| {
                    !params.non_zero || !v.is_zero()
                })
                .boxed()
        }

        type Strategy = BoxedStrategy<Self>;
    }

    impl Vec2 {
        pub fn non_zero() -> impl Strategy<Value = Vec2> {
            let mut params = Vec2Parameters::default();
            params.non_zero = true;
            Self::arbitrary_with(params)
        }
    }

    mod is_zero {
        use crate::model::vec2::Vec2;
        use proptest::proptest;

        #[test]
        fn should_be_true_when_zero() {
            assert!(Vec2::ZERO.is_zero());
        }

        #[test]
        fn should_be_false_when_not_zero() {
            proptest!(|(v in Vec2::non_zero())| {
                assert!(!v.is_zero());
            })
        }
    }

    mod length {
        use crate::model::vec2::Vec2;
        use approx::assert_abs_diff_eq;
        use proptest::proptest;

        #[test]
        fn should_have_correct_length_when_one_component_is_zero() {
            proptest!(|(len in -100.0..100.0)| {
                    for v in &[
                        Vec2::new(0.0, len),
                        Vec2::new(len, 0.0),
                        Vec2::new(0.0, -len),
                        Vec2::new(-len, 0.0),
                    ] {
                        assert_eq!(v.length(), len.abs());
                    }
            })
        }

        proptest! {
            #[test]
            fn scaling_the_vector_should_scale_the_length(v: Vec2, scale in -100f64..100.0) {
                assert_abs_diff_eq!(v.length() * scale.abs(), (v * scale).length(), epsilon = 1e-8)
            }
        }
    }

    mod normalize {
        use crate::model::vec2::Vec2;
        use approx::assert_relative_eq;
        use proptest::proptest;

        proptest! {
            #[test]
            fn should_have_length_one_after_normalization(v in Vec2::non_zero()) {
                assert_relative_eq!(v.normalized().length(), 1.0)
            }

            #[test]
            fn should_retain_angle(v: Vec2) {
                assert_relative_eq!(v.normalized().angle(), v.angle());
            }
        }

        #[test]
        fn should_do_nothing_when_zero() {
            assert_eq!(Vec2::ZERO.normalized(), Vec2::ZERO);
        }
    }

    mod angle {
        use crate::model::vec2::Vec2;
        use approx::assert_relative_eq;
        use proptest::proptest;
        use std::f64::consts::PI;

        proptest! {
            #[test]
            fn should_retain_angle_after_scaling(v: Vec2, scale in 0.1f64..100.0) {
                assert_relative_eq!((v * scale).angle(), v.angle());
            }
        }

        #[test]
        fn should_have_correct_angle() {
            assert_relative_eq!(Vec2::new(1.0, 0.0).angle(), 0.0);
            assert_relative_eq!(Vec2::new(1.0, 1.0).angle(), PI / 4.0);
            assert_relative_eq!(Vec2::new(0.0, 1.0).angle(), PI / 2.0);
            assert_relative_eq!(Vec2::new(-1.0, 1.0).angle(), 3.0 * PI / 4.0);
            assert_relative_eq!(Vec2::new(-1.0, 0.0).angle(), PI);
            assert_relative_eq!(Vec2::new(-1.0, -1.0).angle(), -3.0 * PI / 4.0);
            assert_relative_eq!(Vec2::new(0.0, -1.0).angle(), -PI / 2.0);
            assert_relative_eq!(Vec2::new(1.0, -1.0).angle(), -PI / 4.0);
        }
    }

    mod angle_between {
        use crate::model::vec2::Vec2;
        use approx::assert_relative_eq;
        use more_asserts::{assert_gt, assert_le};
        use proptest::proptest;
        use std::f64::consts::PI;

        proptest! {
            #[test]
            fn should_be_zero_if_one_vector_is_zero(v: Vec2) {
                assert_relative_eq!(v.angle_to(&Vec2::ZERO), 0.0);
                assert_relative_eq!(Vec2::ZERO.angle_to(&v), 0.0);
            }

            #[test]
            fn angle_between_itself_should_be_zero(v: Vec2) {
                assert_relative_eq!(v.angle_to(&v), 0.0);
            }

            #[test]
            fn should_always_be_between_minus_and_plus_pi(v1: Vec2, v2: Vec2) {
                let angle = v1.angle_to(&v2);
                assert_gt!(angle, -PI);
                assert_le!(angle, PI);
            }

            #[test]
            fn swapping_the_vectors_should_negate_the_angle(v1: Vec2, v2: Vec2) {
                assert_relative_eq!(v1.angle_to(&v2), -v2.angle_to(&v1));
            }
        }

        #[test]
        fn should_have_correct_angle() {
            assert_relative_eq!(
                Vec2::new(1.0, 0.0).angle_to(&Vec2::new(0.0, 1.0)),
                PI / 2.0
            );
            assert_relative_eq!(
                Vec2::new(1.0, -1.0).angle_to(&Vec2::new(1.0, 0.0)),
                PI / 4.0
            );
            assert_relative_eq!(
                Vec2::new(1.0, -1.0).angle_to(&Vec2::new(0.0, 1.0)),
                3.0 * PI / 4.0
            );
        }
    }
}
