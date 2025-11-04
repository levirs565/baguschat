pub mod base64_field {
    use base64::prelude::*;
    use serde::Deserialize;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        BASE64_STANDARD.decode(s).map_err(serde::de::Error::custom)
    }

    pub fn serialize<S>(v: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = BASE64_STANDARD.encode(v);
        serializer.serialize_str(&s)
    }
}
