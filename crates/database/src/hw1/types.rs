//! Shared scalar types used by multiple HW1 database schemas.

use alloc::format;
use alloc::string::String;
use core::fmt;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub(crate) fn deserialize_optional_u8<'de, D>(deserializer: D) -> Result<Option<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer)?
        .map(|value| {
            value
                .parse()
                .map_err(|_| D::Error::custom("invalid unsigned byte value"))
        })
        .transpose()
}

pub(crate) fn deserialize_optional_i8<'de, D>(deserializer: D) -> Result<Option<i8>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer)?
        .map(|value| {
            value
                .parse()
                .map_err(|_| D::Error::custom("invalid signed byte value"))
        })
        .transpose()
}

// `serialize_with` callbacks receive a reference to the field value, so this
// signature cannot use Clippy's preferred `Option<&u8>` or pass-by-value form.
#[expect(
    clippy::ref_option,
    clippy::trivially_copy_pass_by_ref,
    reason = "serde `serialize_with` callbacks must take `&Option<u8>`"
)]
pub(crate) fn serialize_optional_u8<S>(value: &Option<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    value.map(|value| format!("{value}")).serialize(serializer)
}

// See `serialize_optional_u8`; serde imposes the same callback signature here.
#[expect(
    clippy::ref_option,
    clippy::trivially_copy_pass_by_ref,
    reason = "serde `serialize_with` callbacks must take `&Option<i8>`"
)]
pub(crate) fn serialize_optional_i8<S>(value: &Option<i8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    value.map(|value| format!("{value}")).serialize(serializer)
}

/// Deserialize an optional boolean child while preserving the game format's
/// presence-only form (`<Flag />` means `true`). Missing fields are supplied
/// as `None` by serde through `default` before this function is invoked.
pub(crate) fn deserialize_optional_bool<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    bool::deserialize(deserializer).map(Some)
}

/// Deserialize an optional float that the engine reads with `BXMLAttribute_getValueAsFloat`.
///
/// Empty text gives `None`, the same as an absent value. The text can have a trailing `f` or `F`
/// (a C float literal, for example `0.38f`).
pub(crate) fn deserialize_optional_float_text<'de, D>(
    deserializer: D,
) -> Result<Option<f32>, D::Error>
where
    D: Deserializer<'de>,
{
    let Some(text) = Option::<String>::deserialize(deserializer)? else {
        return Ok(None);
    };
    let text = text.trim();
    if text.is_empty() {
        return Ok(None);
    }
    let number = text.strip_suffix(['f', 'F']).unwrap_or(text);
    number
        .parse()
        .map(Some)
        .map_err(|_| D::Error::custom(format!("invalid floating-point value `{text}`")))
}

// See `serialize_optional_u8`; serde imposes the same callback signature here.
#[expect(
    clippy::ref_option,
    clippy::trivially_copy_pass_by_ref,
    reason = "serde `serialize_with` callbacks must take `&Option<f32>`"
)]
/// Serialize an optional float so that [`deserialize_optional_float_text`] reads it back.
///
/// `None` gives empty text, the form of the shipped files (for example `u1=""`).
pub(crate) fn serialize_optional_float_text<S>(
    value: &Option<f32>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(value) => serializer.serialize_f32(*value),
        None => serializer.serialize_str(""),
    }
}

/// Deserialize element text as `BString_parseBool` @ `0x140688070` does.
///
/// The engine compares without case: `true` or `1` gives `true`, `false` or `0` gives `false`.
/// The engine keeps the default value (`false`) for other text, so this function also gives
/// `false` for other text.
pub(crate) fn deserialize_text_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let text = String::deserialize(deserializer)?;
    let text = text.trim();
    Ok(text.eq_ignore_ascii_case("true") || text == "1")
}

/// A two-component vector encoded as comma- or whitespace-separated text.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vector2 {
    /// X component.
    pub x: f32,
    /// Y component.
    pub y: f32,
}

impl Vector2 {
    fn parse(value: &str) -> Result<Self, &'static str> {
        let mut components = value
            .split(|character: char| character == ',' || character.is_ascii_whitespace())
            .filter(|component| !component.is_empty());

        let x = parse_f32(components.next())?;
        let y = parse_f32(components.next())?;
        if components.next().is_some() {
            return Err("expected exactly two vector components");
        }

        Ok(Self { x, y })
    }
}

impl<'de> Deserialize<'de> for Vector2 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value).map_err(D::Error::custom)
    }
}

impl Serialize for Vector2 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{},{}", self.x, self.y))
    }
}

/// A three-component vector encoded as comma- or whitespace-separated text.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vector3 {
    /// X component.
    pub x: f32,
    /// Y component.
    pub y: f32,
    /// Z component.
    pub z: f32,
}

impl Vector3 {
    fn parse(value: &str) -> Result<Self, &'static str> {
        let mut components = value
            .split(|character: char| character == ',' || character.is_ascii_whitespace())
            .filter(|component| !component.is_empty());

        let x = parse_f32(components.next())?;
        let y = parse_f32(components.next())?;
        let z = parse_f32(components.next())?;
        if components.next().is_some() {
            return Err("expected exactly three vector components");
        }

        Ok(Self { x, y, z })
    }
}

impl<'de> Deserialize<'de> for Vector3 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value).map_err(D::Error::custom)
    }
}

impl Serialize for Vector3 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{},{},{}", self.x, self.y, self.z))
    }
}

/// An RGB color encoded as three decimal byte values.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RgbColor {
    /// Red channel (0–255).
    pub red: u8,
    /// Green channel (0–255).
    pub green: u8,
    /// Blue channel (0–255).
    pub blue: u8,
}

impl RgbColor {
    fn parse(value: &str) -> Result<Self, &'static str> {
        let mut components = value
            .split(|character: char| character == ',' || character.is_ascii_whitespace())
            .filter(|component| !component.is_empty());

        let red = parse_u8(components.next())?;
        let green = parse_u8(components.next())?;
        let blue = parse_u8(components.next())?;
        if components.next().is_some() {
            return Err("expected exactly three RGB components");
        }

        Ok(Self { red, green, blue })
    }
}

impl<'de> Deserialize<'de> for RgbColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value).map_err(D::Error::custom)
    }
}

impl Serialize for RgbColor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{} {} {}", self.red, self.green, self.blue))
    }
}

fn parse_f32(component: Option<&str>) -> Result<f32, &'static str> {
    component
        .ok_or("missing vector component")?
        .parse()
        .map_err(|_| "invalid floating-point vector component")
}

fn parse_u8(component: Option<&str>) -> Result<u8, &'static str> {
    component
        .ok_or("missing RGB component")?
        .parse()
        .map_err(|_| "invalid RGB byte component")
}

impl fmt::Display for Vector3 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{},{},{}", self.x, self.y, self.z)
    }
}

impl fmt::Display for Vector2 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{},{}", self.x, self.y)
    }
}
