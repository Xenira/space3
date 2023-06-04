pub fn break_text(text: String, width: f32, font_size: f32, center: bool) -> String {
    let width = (width / (font_size / 2.0)) as usize;
    text.split(' ')
        .fold(vec!["".to_string()], |mut acc: Vec<String>, word| {
            let current_line = acc.last_mut().unwrap();
            if current_line.len() + word.len() > width {
                acc.push(word.to_string());
            } else {
                if !current_line.is_empty() {
                    current_line.push(' ');
                }
                current_line.push_str(word);
            }
            acc
        })
        .iter()
        .map(|line| {
            format!(
                "{}{}",
                " ".repeat(if center && width > line.len() {
                    (width - line.len()) / 2
                } else {
                    0
                }),
                line
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
