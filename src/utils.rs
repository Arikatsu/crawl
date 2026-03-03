pub fn normalize_url(base_url: &str, extracted_path: &str) -> String {
    let trimmed = extracted_path.trim();

    if trimmed.contains('{') || trimmed.contains('}')
        || trimmed.contains("undefined")
        || trimmed.starts_with("data:")
        || trimmed.starts_with('#')
    {
        return String::new();
    }

    let url = if trimmed.starts_with("http") {
        trimmed.to_string()
    } else if trimmed.starts_with("//") {
        format!("https:{}", trimmed)
    } else if trimmed.starts_with('/') {
        format!("{}{}", base_url.trim_end_matches('/'), trimmed)
    } else {
        format!("{}/{}", base_url.trim_end_matches('/'), trimmed)
    };

    url
}

pub fn parse_max_icon_size(sizes: &str) -> u32 {
    sizes
        .split_whitespace()
        .filter_map(|s| {
            if s.eq_ignore_ascii_case("any") {
                return Some(u32::MAX);
            }
            s.split('x')
                .next()
                .and_then(|w| w.parse().ok())
        })
        .max()
        .unwrap_or(0)
}