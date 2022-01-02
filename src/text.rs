pub trait Text {
    fn split_when_balanced(&self, on: char, quote: char) -> Vec<&str>;
}

impl<T> Text for T
where
    T: std::convert::AsRef<str>,
{
    fn split_when_balanced(&self, on: char, quote: char) -> Vec<&str> {
        let text = self.as_ref();

        let mut parts = vec![];
        let mut start = 0;
        let mut quoted = false;
        for (offset, c) in text.char_indices() {
            if c == quote {
                quoted = !quoted;
            }
            if !quoted && c == on {
                parts.push(&text[start..offset]);
                start = offset + 1;
            }
        }
        parts.push(&text[start..]);
        parts.into_iter().filter(|s| !s.is_empty()).collect()
    }
}
