//! Shared scalar types used by multiple HW1 database schemas.

use alloc::format;
use alloc::string::String;
use core::fmt;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A three-component vector encoded as comma- or whitespace-separated text.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
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
    pub red: u8,
    pub green: u8,
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
