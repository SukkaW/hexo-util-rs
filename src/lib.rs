use std::any::type_name;

enum State {
    PlainText,
    Html,
    Comment,
}

fn type_of<T>(_: T) -> &'static str {
    type_name::<T>()
}

pub fn strip_tags(html: &str) -> String {
    // if not string, then safely return an empty string
    if type_of(html) != "&str" {
        "".to_string();
    }

    let mut state = State::PlainText;
    let mut tag_buffer = String::new();
    let mut depth = 0;
    let mut in_quote_char = None;
    let mut output = String::new();

    for char in html.chars() {
        match state {
            State::PlainText => {
                if char == '<' {
                    state = State::Html;
                    tag_buffer += &char.to_string();
                } else {
                    output.push(char);
                }
                continue;
            }
            State::Html => {
                match char {
                    '<' => {
                        // ignore '<' if inside a quote
                        if in_quote_char.is_some() {
                            continue;
                        }
                        // we're seeing a nested '<'
                        depth += 1;
                        continue;
                    }
                    '>' => {
                        // ignore '<' if inside a quote
                        if in_quote_char.is_some() {
                            continue;
                        }

                        if depth != 0 {
                            depth -= 1;
                            continue;
                        }

                        // this is closing the tag in tag_buffer
                        in_quote_char = None;
                        state = State::PlainText;

                        // tag_buffer += '>';
                        tag_buffer = String::new();
                        continue;
                    }
                    '"' | '\'' => {
                        // catch both single and double quotes
                        if in_quote_char.is_some() && char == in_quote_char.unwrap() {
                            in_quote_char = None;
                        } else {
                            in_quote_char = if in_quote_char.is_some() {
                                in_quote_char
                            } else {
                                Some(char)
                            };
                        }

                        tag_buffer.push(char);
                        continue;
                    }
                    '-' => {
                        if tag_buffer == *"<!-" {
                            state = State::Comment;
                        }
                        tag_buffer.push(char);
                        continue;
                    }
                    ' ' | '\t' | '\n' | '\r' => {
                        if tag_buffer == *"<" {
                            state = State::PlainText;
                            output.push_str("< ");
                            tag_buffer = String::new();
                            continue;
                        }

                        tag_buffer.push(char);
                        continue;
                    }
                    _ => {
                        tag_buffer.push(char);
                        continue;
                    }
                }
            }
            State::Comment => {
                if char == '>' {
                    if tag_buffer.ends_with("--") {
                        state = State::PlainText;
                    }
                    tag_buffer = String::new();
                    continue;
                } else {
                    tag_buffer.push(char);
                    continue;
                }
            }
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_not_strip_invalid_tags() {
        let result = strip_tags("lorem ipsum < a> < div>");
        assert_eq!(result, "lorem ipsum < a> < div>");
    }

    #[test]
    fn should_remove_simple_html_tags() {
        let result = strip_tags("<a href=\"\">lorem <strong>ipsum</strong></a>");
        assert_eq!(result, "lorem ipsum");
    }

    #[test]
    fn should_remove_comments() {
        let result = strip_tags("<!-- lorem -- ipsum -- --> dolor sit amet");
        assert_eq!(result, " dolor sit amet");
    }

    #[test]
    fn should_strip_tags_within_comments() {
        let result = strip_tags("<!-- <strong>lorem ipsum</strong> --> dolor sit");
        assert_eq!(result, " dolor sit");
    }

    #[test]
    fn should_not_fail_with_nested_quotes() {
        let result = strip_tags("<article attr=\"foo 'bar'\">lorem</article> ipsum");
        assert_eq!(result, "lorem ipsum");
    }

    #[test]
    fn should_strip_extra_within_tags() {
        let result = strip_tags("<div<>>lorem ipsum</div>");
        assert_eq!(result, "lorem ipsum");
    }

    #[test]
    fn should_strip_within_quotes() {
        let result = strip_tags("<a href=\"<script>\">lorem ipsum</a>");
        assert_eq!(result, "lorem ipsum");
    }
}
