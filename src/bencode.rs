use anyhow;
use serde_bencode;

pub fn decode_bencoded_value(encoded_value: &str) -> anyhow::Result<serde_json::Value> {
    fn convert(value: serde_bencode::value::Value) -> anyhow::Result<serde_json::Value> {
        match value {
            serde_bencode::value::Value::Bytes(b) => {
                let string = String::from_utf8(b)?;
                Ok(serde_json::Value::String(string))
            }
            serde_bencode::value::Value::Int(i) => {
                Ok(serde_json::Value::Number(serde_json::Number::from(i)))
            }
            serde_bencode::value::Value::List(l) => {
                let array = l
                    .into_iter()
                    .map(|item| convert(item))
                    .collect::<anyhow::Result<Vec<serde_json::Value>>>()?;
                Ok(serde_json::Value::Array(array))
            }
            serde_bencode::value::Value::Dict(d) => {
                let mut dict = serde_json::Map::new();
                for (key, value) in d {
                    let decoded_key = convert(serde_bencode::value::Value::Bytes(key))?;
                    let decoded_value = convert(value)?;
                    if let Some(key_str) = decoded_key.as_str() {
                        dict.insert(key_str.to_owned(), decoded_value);
                    } else {
                        anyhow::bail!( "Dictionary key is not a valid UTF-8 (byte)string");
                    }
                }
                Ok(serde_json::Value::Object(dict))
            }
        }
    }

    let value: serde_bencode::value::Value = serde_bencode::from_str(encoded_value)?;
    convert(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn decode_string_valid() {
        assert_eq!(decode_bencoded_value("5:hello").unwrap(), json!("hello"));
    }

    #[test]
    fn decode_string_empty() {
        assert_eq!(decode_bencoded_value("0:").unwrap(), json!(""));
    }

    #[test]
    #[should_panic]
    fn decode_string_non_numeric_length() {
        decode_bencoded_value(":hello").unwrap();
    }

    #[test]
    #[should_panic]
    fn decode_string_missing_colon() {
        decode_bencoded_value("5hello").unwrap();
    }

    #[test]
    #[should_panic]
    fn decode_string_invalid_length() {
        decode_bencoded_value("invalid:hello").unwrap();
    }

    #[test]
    #[should_panic]
    fn decode_string_invalid_length_is_bigger() {
        decode_bencoded_value("6:hello").unwrap();
    }

    #[test]
    fn decode_string_length_is_smaller() {
        assert_eq!(decode_bencoded_value("5:helloo").unwrap(), json!("hello"));
    }

    #[test]
    fn decode_number_valid() {
        assert_eq!(decode_bencoded_value("i123456e").unwrap(), json!(123456));
    }

    #[test]
    fn decode_number_valid_negative() {
        assert_eq!(decode_bencoded_value("i-789e").unwrap(), json!(-789));
    }

    #[test]
    #[should_panic]
    fn decode_number_invalid_missing_end() {
        decode_bencoded_value("i123456").unwrap();
    }

    #[test]
    #[should_panic]
    fn decode_number_invalid() {
        decode_bencoded_value("ixyzee").unwrap();
    }

    #[test]
    #[should_panic]
    fn decode_number_invalid_empty() {
        decode_bencoded_value("ie").unwrap();
    }

    // specs say this shouldn't be allowed
    /*
    #[test]
    #[should_panic]
    fn test_decode_number_invalid_leading_zero() {
        decode_bencoded_value("i01e").unwrap();
    }

    #[test]
    #[should_panic]
    fn test_decode_number_invalid_negative_zero() {
        decode_bencoded_value("i-0e").unwrap();
    }
    */

    #[test]
    fn decode_list_valid_empty() {
        assert_eq!(decode_bencoded_value("le").unwrap(), serde_json::json!([]));
    }

    #[test]
    fn decode_list_valid_single_element() {
        assert_eq!(
            decode_bencoded_value("li42ee").unwrap(),
            serde_json::json!([42])
        );
    }

    #[test]
    fn decode_list_valid_multiple_elements() {
        assert_eq!(
            decode_bencoded_value("li42e5:helloi123ee").unwrap(),
            serde_json::json!([42, "hello", 123])
        );
    }

    #[test]
    fn decode_list_valid_nested() {
        assert_eq!(
            decode_bencoded_value("lli42eee").unwrap(),
            serde_json::json!([[42]])
        );
    }

    #[test]
    #[should_panic]
    fn decode_list_invalid_empty() {
        decode_bencoded_value("l").unwrap();
    }

    #[test]
    #[should_panic]
    fn decode_list_invalid_missing_end() {
        decode_bencoded_value("li42e").unwrap();
    }
    #[test]
    fn decode_dictionary_valid_empty() {
        assert_eq!(decode_bencoded_value("de").unwrap(), json!({}));
    }

    #[test]
    fn decode_dictionary_valid_single_element() {
        assert_eq!(
            decode_bencoded_value("d3:key5:valuee").unwrap(),
            json!({"key": "value"})
        );
    }

    #[test]
    fn decode_dictionary_valid_multiple_elements() {
        assert_eq!(
            decode_bencoded_value("d3:key5:value5:helloi123ee").unwrap(),
            json!({"key": "value", "hello": 123})
        );
    }

    #[test]
    fn decode_dictionary_valid_nested() {
        assert_eq!(
            decode_bencoded_value("d1:ad1:b3:fooee").unwrap(),
            json!({"a": {"b": "foo"}})
        );
    }

    #[test]
    fn decode_dictionary_valid_with_nested_list() {
        assert_eq!(
            decode_bencoded_value("d1:ali1ei2ei3eee").unwrap(),
            json!({"a": [1, 2, 3]})
        );
    }

    #[test]
    fn decode_dictionary_valid_nested_with_list() {
        assert_eq!(
            decode_bencoded_value("d1:ad1:bli1ei2eeee").unwrap(),
            json!({"a": {"b": [1, 2]}})
        );
    }

    #[test]
    #[should_panic]
    fn decode_dictionary_invalid_missing_end() {
        decode_bencoded_value("d3:key5:value").unwrap();
    }
}
