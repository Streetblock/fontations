//! Isolated empirical S45 arithmetic research, original code, MIT.
//!
//! Exact integer translation of the frozen Python candidate. The base period
//! 0x2D47 and six quotient bits are compatibility hypotheses, not claims about
//! the TrueType specification or the internal arithmetic used by Windows GDI.

/// Decode the selector's byte fields, leaving reserved period11 to upstream.
pub(super) fn parameters(selector: i32) -> Option<(i32, i32, i32)> {
    let selector = i64::from(selector & 0xFF);
    let period14: i64 = match selector & 0xC0 {
        0 => 0x2D47 / 2,
        0x40 => 0x2D47,
        0x80 => 0x2D47 * 2,
        _ => return None,
    };
    let rounded_period26 = (period14 + 128).div_euclid(256);
    let phase26 = (((selector & 0x30) >> 4) * rounded_period26 + 2).div_euclid(4);
    let low = selector & 15;
    let threshold26 = if low == 0 {
        rounded_period26 - 1
    } else {
        ((low - 4) * rounded_period26 + 4).div_euclid(8)
    };
    Some((period14 as i32, phase26 as i32, threshold26 as i32))
}

/// Parameters are exclusively produced by `parameters` above: positive period.
pub(super) fn round(distance26: i32, period14: i32, phase26: i32, threshold26: i32) -> i32 {
    let distance = i64::from(distance26);
    let period = i64::from(period14);
    let phase = i64::from(phase26);
    let threshold = i64::from(threshold26);
    let numerator14 = (distance.abs() + threshold - phase) * 256;
    // Python truncate_ratio: signed division towards zero FIRST, not floor.
    let quotient26 = (numerator14 * 64) / period;
    // Python //64 and final //256 are floor, including negative values.
    let index = quotient26.div_euclid(64);
    let mut value14 = index * period + phase * 256;
    if distance != 0 && value14 < 0 {
        value14 = phase * 256;
    }
    let result26 = (value14 + 128).div_euclid(256);
    let signed = if distance < 0 { -result26 } else { result26 };
    // The VM's F26.6 container is i32; explicit Rust casts preserve its wrapping
    // representation at extreme inputs. All arithmetic before this cast is i64.
    signed as i32
}

#[cfg(test)]
mod tests {
    use super::{parameters, round};

    fn calculate(distance: i32, selector: i32) -> i32 {
        let (period, phase, threshold) = parameters(selector).unwrap();
        round(distance, period, phase, threshold)
    }

    #[test]
    fn research_s45_parameters_use_floor_for_negative_thresholds() {
        assert_eq!(parameters(0x00), Some((5795, 0, 22)));
        assert_eq!(parameters(0x01), Some((5795, 0, -9)));
        assert_eq!(parameters(0x02), Some((5795, 0, -6)));
        assert_eq!(parameters(0x03), Some((5795, 0, -3)));
        assert_eq!(parameters(0x48), Some((11591, 0, 23)));
        assert_eq!(parameters(0xB2), Some((23182, 68, -23)));
    }

    #[test]
    fn research_s45_zero_has_no_sign_guard() {
        assert_eq!(calculate(0, 0xB2), -23);
        assert_eq!(calculate(-1, 0xB2), -68);
        assert_eq!(calculate(1, 0xB2), 68);
        assert_eq!(calculate(0, 0x01), -23);
        assert_eq!(calculate(-1, 0x01), 0);
        assert_eq!(calculate(1, 0x01), 0);
    }

    #[test]
    fn research_s45_large_signed_inputs_preserve_exact_integer_semantics() {
        assert_eq!(calculate(-4097, 0x00), -4097);
        assert_eq!(calculate(4097, 0x00), 4097);
        assert_eq!(calculate(-4097, 0x48), -4075);
        assert_eq!(calculate(4097, 0x48), 4075);
        assert_eq!(calculate(-4097, 0xB2), -4052);
        assert_eq!(calculate(4097, 0xB2), 4052);
    }

    #[test]
    fn research_s45_selected_transition_neighbours_and_raster_inputs() {
        assert_eq!(calculate(158, 0x48), 136);
        assert_eq!(calculate(746, 0x48), 724);
        assert_eq!(calculate(475, 0x48), 453);
        assert_eq!(calculate(476, 0x48), 498);
        assert_eq!(calculate(-475, 0x48), -453);
        assert_eq!(calculate(-476, 0x48), -498);
    }

    #[test]
    fn research_s45_reserved_period_retains_upstream_fallback() {
        for selector in 0xC0..=0xFF {
            assert_eq!(parameters(selector), None);
        }
    }

    #[test]
    fn research_s45_i32_extremes_do_not_overflow_intermediates() {
        for selector in 0..192 {
            let _ = calculate(i32::MIN, selector);
            let _ = calculate(i32::MAX, selector);
        }
        assert_eq!(calculate(i32::MIN, 0x48), -2147483632);
        assert_eq!(calculate(i32::MAX, 0x48), 2147483632);
        assert_eq!(calculate(i32::MIN, 0x00), 2147483632);
        assert_eq!(calculate(i32::MAX, 0x00), -2147483632);
    }
}
