use serde::{de::DeserializeOwned, Serialize};

use super::TypeFallibleMapper;

/// Mapper between [serde] types using [json](serde_json) serializer/deserializer
pub struct Mapper;
impl<T: Serialize, Z: DeserializeOwned> TypeFallibleMapper<T, Z> for Mapper {
    type Error = serde_json::Error;

    fn try_map(from: T) -> Result<Z, Self::Error> {
        serde_json::from_value(serde_json::to_value(from)?)
    }
}

/// Mapper from a [serde_json::Value] or a json [String] to any [DeserializeOwned]
pub struct FromJsonMapper;
impl<T: DeserializeOwned> TypeFallibleMapper<serde_json::Value, T> for FromJsonMapper {
    type Error = serde_json::Error;

    fn try_map(from: serde_json::Value) -> Result<T, Self::Error> {
        serde_json::from_value(from)
    }
}
impl<T: DeserializeOwned> TypeFallibleMapper<String, T> for FromJsonMapper {
    type Error = serde_json::Error;

    fn try_map(from: String) -> Result<T, Self::Error> {
        serde_json::from_str(&from)
    }
}
impl<T: DeserializeOwned> TypeFallibleMapper<&String, T> for FromJsonMapper {
    type Error = serde_json::Error;

    fn try_map(from: &String) -> Result<T, Self::Error> {
        serde_json::from_str(from)
    }
}
impl<T: DeserializeOwned> TypeFallibleMapper<&str, T> for FromJsonMapper {
    type Error = serde_json::Error;

    fn try_map(from: &str) -> Result<T, Self::Error> {
        serde_json::from_str(from)
    }
}

/// Mapper from any [Serialize] to a [serde_json::Value] or json [String]
pub struct ToJsonMapper;
impl<T: Serialize> TypeFallibleMapper<T, serde_json::Value> for ToJsonMapper {
    type Error = serde_json::Error;

    fn try_map(from: T) -> Result<serde_json::Value, Self::Error> {
        serde_json::to_value(from)
    }
}
impl<T: Serialize> TypeFallibleMapper<T, String> for ToJsonMapper {
    type Error = serde_json::Error;

    fn try_map(from: T) -> Result<String, Self::Error> {
        serde_json::to_string(&from)
    }
}

/// Mapper from any [Serialize] to a pretty json [String]
pub struct ToPrettyJsonMapper;
impl<F: Serialize> TypeFallibleMapper<F, String> for ToPrettyJsonMapper {
    type Error = serde_json::Error;

    fn try_map(from: F) -> Result<String, Self::Error> {
        serde_json::to_string_pretty(&from)
    }
}
