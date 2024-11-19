pub fn parse_http_response(response: &str) -> Option<String> {
    // Split the response into lines
    let lines: Vec<&str> = response.lines().collect();

    // Check if response is empty
    if lines.is_empty() {
        return None;
    }

    // Parse status line
    let status_line = lines[0];
    if !is_successful_response(status_line) {
        return None;
    }

    // Find the separation between headers and content (empty line)
    let mut content_start = 0;
    for (i, line) in lines.iter().enumerate() {
        if line.trim().is_empty() {
            content_start = i + 1;
            break;
        }
    }

    // If we found a content section, return it
    if content_start > 0 && content_start < lines.len() {
        Some(lines[content_start..].join("\n"))
    } else {
        None
    }
}

fn is_successful_response(status_line: &str) -> bool {
    if let Some(status_code) = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u16>().ok())
    {
        (200..300).contains(&status_code)
    } else {
        false
    }
}
