//! Serde helper module for handling special float values (NaN, Infinity)
//!
//! JSON doesn't natively support NaN or Infinity values, so we need custom
//! serialization/deserialization logic to handle these cases.

/// Custom serde module for f64 values that may be NaN or Infinity
#[cfg(feature = "serde")]
pub mod nan_safe_f64 {
    use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

    /// Serialize an f64, converting NaN and Infinity to special string representations
    pub fn serialize<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if value.is_nan() {
            "NaN".serialize(serializer)
        } else if value.is_infinite() {
            if value.is_sign_positive() {
                "Infinity".serialize(serializer)
            } else {
                "-Infinity".serialize(serializer)
            }
        } else {
            value.serialize(serializer)
        }
    }

    /// Deserialize an f64, handling special string representations for NaN and Infinity
    pub fn deserialize<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum FloatOrString {
            Float(f64),
            String(String),
        }

        match FloatOrString::deserialize(deserializer)? {
            FloatOrString::Float(f) => Ok(f),
            FloatOrString::String(s) => match s.as_str() {
                "NaN" | "nan" => Ok(f64::NAN),
                "Infinity" | "inf" | "+Infinity" | "+inf" => Ok(f64::INFINITY),
                "-Infinity" | "-inf" => Ok(f64::NEG_INFINITY),
                _ => Err(de::Error::custom(format!("Invalid float string: {}", s))),
            },
        }
    }
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use super::nan_safe_f64;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct TestStruct {
        #[serde(with = "nan_safe_f64")]
        value: f64,
    }

    #[test]
    fn test_nan_serialization() {
        let test = TestStruct { value: f64::NAN };
        let json = serde_json::to_string(&test).unwrap();
        assert_eq!(json, r#"{"value":"NaN"}"#);

        let loaded: TestStruct = serde_json::from_str(&json).unwrap();
        assert!(loaded.value.is_nan());
    }

    #[test]
    fn test_infinity_serialization() {
        let test = TestStruct {
            value: f64::INFINITY,
        };
        let json = serde_json::to_string(&test).unwrap();
        assert_eq!(json, r#"{"value":"Infinity"}"#);

        let loaded: TestStruct = serde_json::from_str(&json).unwrap();
        assert!(loaded.value.is_infinite());
        assert!(loaded.value.is_sign_positive());
    }

    #[test]
    fn test_neg_infinity_serialization() {
        let test = TestStruct {
            value: f64::NEG_INFINITY,
        };
        let json = serde_json::to_string(&test).unwrap();
        assert_eq!(json, r#"{"value":"-Infinity"}"#);

        let loaded: TestStruct = serde_json::from_str(&json).unwrap();
        assert!(loaded.value.is_infinite());
        assert!(loaded.value.is_sign_negative());
    }

    #[test]
    fn test_normal_float_serialization() {
        let test = TestStruct { value: 3.1 };
        let json = serde_json::to_string(&test).unwrap();
        assert_eq!(json, r#"{"value":3.1}"#);

        let loaded: TestStruct = serde_json::from_str(&json).unwrap();
        assert!((loaded.value - 3.1).abs() < 1e-10);
    }
}
