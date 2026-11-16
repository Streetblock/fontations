//! Isolated research projection arithmetic, not a production compatibility mode.
//!
//! P1H: independently round each product toward positive infinity at a half,
//! then add. Inputs are signed F26.6 coordinates and F2.14 vector components.
//! The final wrapping i32 conversion is retained from the baseline dot14.

#[inline(always)]
pub(super) fn dot14(ax: i32, ay: i32, bx: i32, by: i32) -> i32 {
    let x = (i64::from(ax) * i64::from(bx) + 0x2000) >> 14;
    let y = (i64::from(ay) * i64::from(by) + 0x2000) >> 14;
    (x + y) as i32
}

#[cfg(test)]
mod tests {
    use super::dot14;

    #[test]
    fn positive_and_negative_half_products_round_toward_positive_infinity() {
        assert_eq!(dot14(1, 0, 8192, 0), 1);
        assert_eq!(dot14(-1, 0, 8192, 0), 0);
        assert_eq!(dot14(3, 0, 8192, 0), 2);
        assert_eq!(dot14(-3, 0, 8192, 0), -1);
        assert_eq!(dot14(8192, 0, 11585, 11585), 5793);
        assert_eq!(dot14(-8192, 0, 11585, 11585), -5792);
    }

    #[test]
    fn products_are_rounded_before_the_sum() {
        assert_eq!(dot14(1, 1, 8192, 8192), 2);
        assert_eq!(dot14(-1, -1, 8192, 8192), 0);
        assert_eq!(dot14(100, 100, 11585, 11585), 142);
        assert_eq!(dot14(-100, -100, 11585, 11585), -142);
    }

    #[test]
    fn forty_five_degree_known_coordinate_controls() {
        assert_eq!(dot14(800, 800, 11585, 11585), 1132);
        assert_eq!(dot14(808, 808, 11585, 11585), 1142);
        assert_eq!(dot14(-800, -800, 11585, 11585), -1132);
    }

    #[test]
    fn non_forty_five_vector_and_axis_controls() {
        assert_eq!(dot14(800, 800, 9830, 13107), 1120);
        assert_eq!(dot14(-800, -800, 9830, 13107), -1120);
        assert_eq!(dot14(800, 800, 7327, 14654), 1074);
        assert_eq!(dot14(-800, -800, 7327, 14654), -1074);
        assert_eq!(dot14(800, -808, 16384, 0), 800);
        assert_eq!(dot14(800, -808, 0, 16384), -808);
        assert_eq!(dot14(800, -808, -16384, 0), -800);
    }

    #[test]
    fn exact_product_half_neighbours_and_negative_vectors() {
        for (input, expected) in [(8191, 4096), (8192, 4096), (8193, 4097)] {
            assert_eq!(dot14(input, 0, 8192, 0), expected);
        }
        assert_eq!(dot14(1, 0, -8192, 0), 0);
        assert_eq!(dot14(-1, 0, -8192, 0), 1);
        assert_eq!(dot14(0, 0, -11585, 11585), 0);
    }

    #[test]
    fn wide_products_do_not_overflow_and_final_i32_cast_wraps() {
        assert_eq!(dot14(i32::MAX, 0, 16384, 0), i32::MAX);
        assert_eq!(dot14(i32::MIN, 0, 16384, 0), i32::MIN);
        assert_eq!(dot14(i32::MAX, i32::MAX, 16384, 16384), -2);
        assert_eq!(dot14(i32::MIN, i32::MIN, 16384, 16384), 0);
        assert_eq!(dot14(i32::MAX, i32::MIN, 16384, 16384), -1);
    }
}
