//! GDI-compatible arithmetic for monochrome glyph point movement.
//!
//! Distances are raw F26.6 and normalized vectors are raw F2.14. When the
//! checked dot product is unavailable, the caller uses the normal arithmetic.

const ONE: i64 = 16384;
const MIN_ABS_DOT: i64 = 4096;

/// Round one movement component with the unit factor set by SFVTPV.
/// Half ties round toward positive infinity on either axis.
#[inline(always)]
pub(super) fn sfvtpv_delta(distance: i32, component: i32) -> i32 {
    let product = i64::from(distance) * i64::from(component);
    ((product + 8192) >> 14) as i32
}

#[inline(always)]
fn local_dot(pvx: i32, pvy: i32, fvx: i32, fvy: i32) -> i64 {
    let x = (i64::from(pvx) * i64::from(fvx) + 8192) >> 14;
    let y = (i64::from(pvy) * i64::from(fvy) + 8192) >> 14;
    x + y
}

#[inline(always)]
fn from_dot(distance: i32, fvx: i32, k: i64) -> i32 {
    let product = i64::from(distance) * i64::from(fvx);
    if k == ONE {
        // Exact existing mul14 nearest-away arithmetic, including negative ties.
        ((product + 8192 + (product >> 63)) >> 14) as i32
    } else if i64::from(fvx) == k {
        distance
    } else {
        // Bias the signed raw numerator, then Rust/C truncation toward zero.
        // Do not normalize denominator sign or replace with nearest division.
        ((product + k.abs() / 2) / k) as i32
    }
}

#[inline(always)]
pub(super) fn checked_dot(pvx: i32, pvy: i32, fvx: i32, fvy: i32) -> Option<i32> {
    // The interpreter supplies normalized vectors. Reject out-of-range raw
    // components without abs(i32::MIN); no extra normalization is performed.
    if [pvx, pvy, fvx, fvy]
        .iter()
        .any(|&value| !(-16384..=16384).contains(&value))
    {
        return None;
    }
    let k = local_dot(pvx, pvy, fvx, fvy);
    // Unguarded local dot: do not apply or mutate the global near-zero clamp.
    if k.abs() < MIN_ABS_DOT {
        return None;
    }
    Some(k as i32)
}

#[inline(always)]
pub(super) fn x_delta(distance: i32, pvx: i32, pvy: i32, fvx: i32, fvy: i32) -> Option<i32> {
    checked_dot(pvx, pvy, fvx, fvy).map(|k| from_dot(distance, fvx, i64::from(k)))
}

#[cfg(test)]
mod tests {
    use super::{from_dot, local_dot, sfvtpv_delta, x_delta, ONE};

    #[test]
    fn sfvtpv_rounds_signed_components_and_negative_half_ties() {
        assert_eq!(sfvtpv_delta(-44, 13120), -35);
        assert_eq!(sfvtpv_delta(-44, 9813), -26);
        assert_eq!(sfvtpv_delta(-9, 11364), -6);
        assert_eq!(sfvtpv_delta(-9, -11802), 6);
        assert_eq!(sfvtpv_delta(-3, 8192), -1);
        assert_eq!(sfvtpv_delta(3, -8192), -1);
    }

    #[test]
    fn local_dot_uses_separate_positive_half_rounding() {
        assert_eq!(local_dot(11585, 11585, 11585, 11585), ONE);
        assert_eq!(local_dot(7327, 14654, 7327, 14654), ONE);
        assert_eq!(local_dot(9830, 13107, 9830, 13107), ONE - 1);
        assert_eq!(local_dot(0, 16384, 11585, 11585), 11585);
        assert_eq!(local_dot(-16384, 0, 9830, 13107), -9830);
    }

    #[test]
    fn raw_component_bounds_reject_without_overflow() {
        for bad in [i32::MIN, -16385, 16385, i32::MAX] {
            assert_eq!(x_delta(1, bad, 0, 16384, 0), None);
            assert_eq!(x_delta(1, 16384, bad, 16384, 0), None);
            assert_eq!(x_delta(1, 16384, 0, bad, 0), None);
            assert_eq!(x_delta(1, 16384, 0, 16384, bad), None);
        }
    }

    #[test]
    fn outside_local_dot_domain_returns_baseline_fallback_signal() {
        assert_eq!(x_delta(1, 16384, 0, 4095, 15864), None);
        assert_eq!(x_delta(1, -16384, 0, 4095, 15864), None);
        assert_eq!(x_delta(1, 16384, 0, 0, 16384), None);
        assert_eq!(x_delta(1, 16384, 0, 4096, 15864), Some(1));
        assert_eq!(x_delta(-1, -16384, 0, 4096, 15864), Some(0));
    }

    #[test]
    fn positive_one_fastpath_keeps_original_nearest_away_ties() {
        assert_eq!(from_dot(1, 8192, ONE), 1);
        assert_eq!(from_dot(-1, 8192, ONE), -1);
        assert_eq!(from_dot(3, 8192, ONE), 2);
        assert_eq!(from_dot(-3, 8192, ONE), -2);
    }

    #[test]
    fn exact_ratio_fastpath_preserves_negative_distance_and_denominator() {
        assert_eq!(x_delta(-8, 0, 16384, 11585, 11585), Some(-8));
        assert_eq!(x_delta(-7, 16384, 0, -16384, 0), Some(-7));
        assert_eq!(from_dot(7, -9830, -9830), 7);
        assert_eq!(from_dot(-7, -9830, -9830), -7);
    }

    #[test]
    fn signed_raw_bias_is_not_sign_normalized() {
        assert_eq!(x_delta(7, -16384, 0, 9830, 13107), Some(-7));
        assert_eq!(x_delta(-7, -16384, 0, 9830, 13107), Some(6));
        assert_eq!(from_dot(-9, 9830, 16383), -4);
        assert_eq!(from_dot(-9, -9830, -16383), -5);
    }

    #[test]
    fn one_neighbours_keep_distinct_branch_order() {
        assert_eq!(from_dot(-1, 8192, ONE - 1), 0);
        assert_eq!(from_dot(-1, 8192, ONE), -1);
        assert_eq!(from_dot(-1, 8192, ONE + 1), 0);
        assert_eq!(from_dot(1, 8192, ONE - 1), 1);
        assert_eq!(from_dot(1, 8192, ONE + 1), 0);
    }

    #[test]
    fn zero_distance_is_zero_across_eligible_branches() {
        for k in [-16384, -9830, -4096, 4096, 9830, 16383, 16384, 16385] {
            assert_eq!(from_dot(0, 9830, k), 0);
        }
    }

    #[test]
    fn wide_products_and_final_cast_keep_i32_wrapping_semantics() {
        assert_eq!(from_dot(i32::MAX, 16384, ONE), i32::MAX);
        assert_eq!(from_dot(i32::MIN, 16384, ONE), i32::MIN);
        assert_eq!(from_dot(i32::MIN, -16384, ONE), i32::MIN);
        assert_eq!(from_dot(i32::MIN, -16384, 4096), 0);
        assert_eq!(from_dot(i32::MAX, 16384, 4096), -4);
    }

    #[test]
    fn captured_negative_movement_controls_use_only_numeric_inputs() {
        // Existing diagnostic measurements, not fresh holdout assertions.
        assert_eq!(x_delta(-8, 0, 16384, 11585, 11585), Some(-8));
        assert_eq!(x_delta(-9, 9830, 13107, 9830, 13107), Some(-4));
        assert_eq!(x_delta(-19, -13107, 9830, -13107, 9830), Some(15));
    }
}
