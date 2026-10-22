//! Text conversions that follow the XML value readers of the retail scenario loader.
//!
//! `BScenario_load` @ `0x140377be0` and its helpers read each value with a
//! small set of reader functions:
//!
//! - Integers: `sub_140621290` (text: `strtol` base 10) and `sub_14068FEB0`
//!   (XMB string variant: `atoi`).
//! - Floats: `sub_140621390` (text: `strtod`), `sub_140621880` (element
//!   text) and `sub_14068FCC0` (XMB variant).
//! - Booleans: `sub_1406216D0`, which calls `sub_140688070`.
//! - Vectors: `sub_1406217A0`, which calls `sub_140623AC0`
//!   (`sscanf(text, "%f,%f,%f")`).
//!
//! Each reader leaves the destination unchanged when the value text is empty.
//! The functions in this module give `None` for that case. The callers then
//! keep the engine default.

use serde::{Deserialize, Deserializer};

/// Characters that the C `isspace` function accepts.
fn is_c_space(character: char) -> bool {
    matches!(character, ' ' | '\t' | '\n' | '\x0b' | '\x0c' | '\r')
}

/// Convert text to an integer like the C `strtol` function with base 10.
///
/// The conversion skips leading white space, reads an optional sign and then
/// reads decimal digits. Text without digits gives `0`. A value out of the
/// `i32` range gives `i32::MAX` or `i32::MIN`, as the MSVC runtime does.
#[must_use]
pub(crate) fn strtol(text: &str) -> i32 {
    let text = text.trim_start_matches(is_c_space);
    let (negative, digits) = match text.as_bytes().first() {
        Some(b'-') => (true, &text[1..]),
        Some(b'+') => (false, &text[1..]),
        _ => (false, text),
    };
    let mut value: i64 = 0;
    for digit in digits.bytes().take_while(u8::is_ascii_digit) {
        value = (value * 10 + i64::from(digit - b'0')).min(i64::from(i32::MAX) + 1);
    }
    let value = if negative { -value } else { value };
    i32::try_from(value).unwrap_or(if negative { i32::MIN } else { i32::MAX })
}

/// Convert text to a float like the C `strtod` function, then narrow it to `f32`.
///
/// Text without a number gives `0.0`. See [`strtod_prefix`] for the syntax.
#[must_use]
pub(crate) fn strtod(text: &str) -> f32 {
    strtod_prefix(text).map_or(0.0, |(value, _)| value)
}

/// Read the float at the start of `text` like the C `strtod` function.
///
/// The conversion skips leading white space and reads the longest decimal
/// prefix (`[+-]digits[.digits][e[+-]digits]`). It also accepts `inf`,
/// `infinity` and `nan` in any case. The result holds the value, narrowed to
/// `f32`, and the number of bytes read. Text without a number gives `None`.
#[must_use]
pub(crate) fn strtod_prefix(text: &str) -> Option<(f32, usize)> {
    let start = text.len() - text.trim_start_matches(is_c_space).len();
    let bytes = text.as_bytes();
    let negative = bytes.get(start) == Some(&b'-');
    let mut end = start + usize::from(matches!(bytes.get(start), Some(b'+' | b'-')));
    for word in ["infinity", "inf", "nan"] {
        if text
            .get(end..end + word.len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(word))
        {
            let magnitude = if word == "nan" {
                f32::NAN
            } else {
                f32::INFINITY
            };
            let value = if negative { -magnitude } else { magnitude };
            return Some((value, end + word.len()));
        }
    }
    let digits_from = |from: usize| {
        bytes[from..]
            .iter()
            .take_while(|byte| byte.is_ascii_digit())
            .count()
    };
    let integer_digits = digits_from(end);
    end += integer_digits;
    let mut fraction_digits = 0;
    if bytes.get(end) == Some(&b'.') {
        fraction_digits = digits_from(end + 1);
        if integer_digits > 0 || fraction_digits > 0 {
            end += 1 + fraction_digits;
        }
    }
    if integer_digits == 0 && fraction_digits == 0 {
        return None;
    }
    if matches!(bytes.get(end), Some(b'e' | b'E')) {
        let sign = usize::from(matches!(bytes.get(end + 1), Some(b'+' | b'-')));
        let exponent_digits = digits_from(end + 1 + sign);
        if exponent_digits > 0 {
            end += 1 + sign + exponent_digits;
        }
    }
    let value: f64 = text[start..end].parse().ok()?;
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the engine stores the strtod double in a float"
    )]
    let narrowed = value as f32;
    Some((narrowed, end))
}

/// Read a vector like `sscanf(text, "%f,%f,%f")` in `sub_140623AC0`.
///
/// Each `%f` skips leading white space. Each comma must follow its number
/// directly. Text after the third number is ignored. If one of the three
/// numbers is missing, the result is `None`.
#[must_use]
pub(crate) fn scan_vec3(text: &str) -> Option<[f32; 3]> {
    let mut vector = [0.0; 3];
    let mut rest = text;
    for (index, component) in vector.iter_mut().enumerate() {
        let (value, length) = strtod_prefix(rest)?;
        *component = value;
        rest = &rest[length..];
        if index < 2 {
            rest = rest.strip_prefix(',')?;
        }
    }
    Some(vector)
}

/// Convert text to a boolean like `sub_140688070`.
///
/// `true` and `1` give `true`. `false` and `0` give `false`. The compare
/// ignores case and does not trim the text. All other text gives `None`.
#[must_use]
pub(crate) fn parse_bool(text: &str) -> Option<bool> {
    if text.eq_ignore_ascii_case("true") || text == "1" {
        Some(true)
    } else if text.eq_ignore_ascii_case("false") || text == "0" {
        Some(false)
    } else {
        None
    }
}

/// Decode an optional integer value with [`strtol`].
///
/// Empty text gives `None`, the same as an absent value.
pub(crate) fn optional_integer<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<i32>, D::Error> {
    let text = String::deserialize(deserializer)?;
    Ok((!text.is_empty()).then(|| strtol(&text)))
}

/// Decode an integer value with [`strtol`]. Empty text gives the engine default `-1`.
pub(crate) fn integer_or_minus_one<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<i32, D::Error> {
    Ok(optional_integer(deserializer)?.unwrap_or(-1))
}

/// Decode an optional float value with [`strtod`].
///
/// Empty text gives `None`, the same as an absent value.
pub(crate) fn optional_float<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<f32>, D::Error> {
    let text = String::deserialize(deserializer)?;
    Ok((!text.is_empty()).then(|| strtod(&text)))
}

/// Decode a float value with [`strtod`]. Empty text gives `0.0`.
pub(crate) fn float_or_zero<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f32, D::Error> {
    Ok(optional_float(deserializer)?.unwrap_or(0.0))
}

/// Decode a float value with [`strtod`]. Empty text gives `1.0`.
pub(crate) fn float_or_one<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f32, D::Error> {
    Ok(optional_float(deserializer)?.unwrap_or(1.0))
}

/// Decode a present value as text, so that empty text stays `Some("")`.
///
/// Use it for a value whose empty text has an effect in the engine.
pub(crate) fn present_text<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    String::deserialize(deserializer).map(Some)
}

/// Decode an optional boolean value with [`parse_bool`].
///
/// Text that [`parse_bool`] rejects gives `None`, the same as an absent value.
pub(crate) fn optional_bool<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<bool>, D::Error> {
    let text = String::deserialize(deserializer)?;
    Ok(parse_bool(&text))
}

/// Decode a boolean value whose engine default is `false`.
pub(crate) fn bool_or_false<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
    Ok(optional_bool(deserializer)?.unwrap_or(false))
}

/// Decode a boolean value whose engine default is `true`.
pub(crate) fn bool_or_true<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
    Ok(optional_bool(deserializer)?.unwrap_or(true))
}

/// Give the engine default `-1` for an absent integer value.
pub(crate) const fn minus_one() -> i32 {
    -1
}

/// Give the engine default `true` for an absent boolean value.
pub(crate) const fn yes() -> bool {
    true
}

/// Give the engine default `1.0` for an absent float value.
pub(crate) const fn one() -> f32 {
    1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strtol_reads_the_decimal_prefix_like_msvc() {
        for (text, expected) in [
            ("42", 42),
            ("  -7", -7),
            ("+3", 3),
            ("25966,Rebels", 25966),
            ("1.9", 1),
            ("abc", 0),
            ("", 0),
            ("-", 0),
            ("99999999999", i32::MAX),
            ("-99999999999", i32::MIN),
        ] {
            assert_eq!(strtol(text), expected, "{text:?}");
        }
    }

    #[test]
    fn strtod_reads_the_decimal_prefix() {
        for (text, expected) in [
            ("1024", 1024.0_f32),
            (" -0.5x", -0.5),
            ("1.", 1.0),
            (".25", 0.25),
            ("2e3", 2000.0),
            ("2e", 2.0),
            ("abc", 0.0),
            (".", 0.0),
            ("-inf", f32::NEG_INFINITY),
        ] {
            assert_eq!(strtod(text).to_bits(), expected.to_bits(), "{text:?}");
        }
        assert!(strtod("NaN").is_nan());
    }

    #[test]
    fn scan_vec3_follows_the_sscanf_format() {
        assert_eq!(
            scan_vec3("767.8011,0,782.4996"),
            Some([767.8011, 0.0, 782.4996])
        );
        assert_eq!(scan_vec3("1, 2, 3 trailing"), Some([1.0, 2.0, 3.0]));
        assert_eq!(scan_vec3("1 ,2,3"), None);
        assert_eq!(scan_vec3("1,2"), None);
        assert_eq!(scan_vec3(""), None);
    }

    #[test]
    fn parse_bool_accepts_only_the_four_engine_words() {
        assert_eq!(parse_bool("TRUE"), Some(true));
        assert_eq!(parse_bool("1"), Some(true));
        assert_eq!(parse_bool("False"), Some(false));
        assert_eq!(parse_bool("0"), Some(false));
        assert_eq!(parse_bool(" true"), None);
        assert_eq!(parse_bool("yes"), None);
        assert_eq!(parse_bool(""), None);
    }
}
