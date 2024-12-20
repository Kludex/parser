use std::collections::HashMap;

/// Parse `Content-Type` like headers.
///
/// Function name inspired by `werkzeug.parse_options_header` in Python.
/// See.: https://tedboy.github.io/flask/generated/werkzeug.parse_options_header.html
pub fn parse_options_header(value: String) -> Result<(String, HashMap<String, String>), String> {
    let mut parts = value.split(';');

    let name = parts.next().ok_or("Missing header name")?.trim();
    let mut parameters = HashMap::new();

    for part in parts {
        let mut parameter_parts = part.splitn(2, '=');
        let key = parameter_parts.next().ok_or("Missing parameter key")?.trim();
        let value = parameter_parts.next().ok_or("Missing parameter value")?.trim();
        parameters.insert(key.to_string(), value.to_string());
    }
    Ok((name.to_string(), parameters))
}
