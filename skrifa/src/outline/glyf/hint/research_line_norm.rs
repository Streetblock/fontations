//! Pure integer line-normalization research candidate.
//!
//! No font, glyph, ppem, hash, angle class or fitted parameter selects behavior.
//! This module does not select the research mode or change caller null handling.

const ONE: u128 = 16_384;

fn nearest_component(value: i32, squared_length: u128) -> i32 {
    let numerator = u128::from(value.unsigned_abs()) * ONE;
    let squared_numerator = numerator * numerator;

    // floor(N / sqrt(S)), computed without float, libm or an approximate sqrt.
    let mut lo = 0_u128;
    let mut hi = ONE + 1;
    while lo + 1 < hi {
        let midpoint = (lo + hi) / 2;
        if midpoint * midpoint * squared_length <= squared_numerator {
            lo = midpoint;
        } else {
            hi = midpoint;
        }
    }
    let mut magnitude = lo;
    let midpoint = 2 * magnitude + 1;
    if 4 * squared_numerator >= midpoint * midpoint * squared_length {
        magnitude += 1;
    }
    let result = magnitude as i32;
    if value < 0 { -result } else { result }
}

/// Nearest raw F2.14 unit vector for signed i32 line deltas.
///
/// None means that the caller must retain its existing coincident-point policy.
/// The caller also retains its existing perpendicular rotation and axis paths.
///
/// For every i32 input: S<=2^63, N^2<=2^90 and all midpoint comparison products
/// are strictly below 2^94. Thus ordinary u128 operations are exact and safe.
/// Signs are restored after magnitude rounding; exact half ties go away from 0.
pub fn normalize_nonzero_nearest(x: i32, y: i32) -> Option<(i32, i32)> {
    let ax = u128::from(x.unsigned_abs());
    let ay = u128::from(y.unsigned_abs());
    let squared_length = ax * ax + ay * ay;
    if squared_length == 0 {
        return None;
    }
    Some((nearest_component(x, squared_length), nearest_component(y, squared_length)))
}

#[cfg(test)]
mod tests {
    use super::{normalize_nonzero_nearest as normalize, ONE};

    #[test]
    fn zero_is_an_explicit_caller_decision() {
        assert_eq!(normalize(0, 0), None);
    }

    #[test]
    fn signed_axes_and_minimum_integer_are_exact() {
        for value in [1, 2, 12345, i32::MAX] {
            assert_eq!(normalize(value, 0), Some((16384, 0)));
            assert_eq!(normalize(-value, 0), Some((-16384, 0)));
            assert_eq!(normalize(0, value), Some((0, 16384)));
            assert_eq!(normalize(0, -value), Some((0, -16384)));
        }
        assert_eq!(normalize(i32::MIN, 0), Some((-16384, 0)));
        assert_eq!(normalize(0, i32::MIN), Some((0, -16384)));
    }

    #[test]
    fn familiar_ratios_use_nearest_component_magnitudes() {
        assert_eq!(normalize(1, 1), Some((11585, 11585)));
        assert_eq!(normalize(3, 4), Some((9830, 13107)));
        assert_eq!(normalize(1, 2), Some((7327, 14654)));
        assert_eq!(normalize(-3, 4), Some((-9830, 13107)));
    }

    #[test]
    fn already_measured_line_discriminators_are_reproduced() {
        // Known diagnostic inputs, not claimed as fresh validation cases.
        assert_eq!(normalize(488, -177), Some((15402, -5586)));
        assert_eq!(normalize(280, -463), Some((8478, -14020)));
        assert_eq!(normalize(461, 266), Some((14191, 8188)));
        assert_eq!(normalize(179, 502), Some((5503, 15432)));
    }

    #[test]
    fn source_derived_non_native_vector_controls_are_explicit() {
        assert_eq!(normalize(1363, 1051), Some((12975, 10005)));
    }

    #[test]
    fn positive_integer_scaling_and_sign_reversal_are_invariant() {
        for (x, y) in [(173,419), (257,-113), (1,1023), (419,173)] {
            let expected = normalize(x,y).unwrap();
            for factor in [1,3,7,64] {
                assert_eq!(normalize(x*factor,y*factor), Some(expected));
                assert_eq!(normalize(-x*factor,-y*factor), Some((-expected.0,-expected.1)));
            }
        }
    }

    #[test]
    fn largest_i32_magnitudes_do_not_overflow() {
        assert_eq!(normalize(i32::MIN,i32::MIN), Some((-11585,-11585)));
        assert_eq!(normalize(i32::MIN,i32::MAX), Some((-11585,11585)));
        assert_eq!(normalize(i32::MAX,i32::MAX), Some((11585,11585)));
        let s = 2_u128 * (1_u128 << 62);
        assert!(s <= 1_u128 << 63);
        assert!((2*ONE+1)*(2*ONE+1)*s < 1_u128 << 94);
    }

    fn check_nearest_interval(component: i32, result: i32, s: u128) {
        let n = u128::from(component.unsigned_abs()) * ONE;
        let r = u128::from(result.unsigned_abs());
        assert!(r <= ONE);
        if result != 0 {
            assert_eq!(result.signum(), component.signum());
        }
        if r > 0 {
            assert!(4*n*n >= (2*r-1)*(2*r-1)*s);
        }
        assert!(4*n*n < (2*r+1)*(2*r+1)*s);
    }

    #[test]
    fn independent_integer_interval_law_on_small_grid() {
        for x in -17_i32..=17 {
            for y in -17_i32..=17 {
                if let Some((nx,ny)) = normalize(x,y) {
                    let s = u128::from(x.unsigned_abs()).pow(2) + u128::from(y.unsigned_abs()).pow(2);
                    check_nearest_interval(x,nx,s);
                    check_nearest_interval(y,ny,s);
                }
            }
        }
    }
}
