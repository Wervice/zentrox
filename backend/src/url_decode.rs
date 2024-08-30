/// Decodes URL encoded value to plain string.
/// `input` - &str to decode
pub fn url_decode(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            // Extract the next two characters to decode
            let mut hex = String::new();
            if let Some(c1) = chars.next() {
                hex.push(c1);
            }
            if let Some(c2) = chars.next() {
                hex.push(c2);
            }

            // Parse the hex string into a u8 value
            if let Ok(decoded_byte) = u8::from_str_radix(&hex, 16) {
                result.push(decoded_byte as char);
            }
        } else if c == '+' {
            // '+' in URL encoding is space
            result.push(' ');
        } else {
            result.push(c);
        }
    }

    result
}
