pub trait Text {
    fn split_when_balanced(&self, on: char, quote: char) -> Vec<&str>;
    fn split_when_balanced_with_offsets(&self, on: char, quote: char) -> Vec<(usize, &str)>;
    fn or_quotes(&self) -> String;
}

impl<T> Text for T
where
    T: std::convert::AsRef<str>,
{
    fn split_when_balanced_with_offsets(&self, on: char, quote: char) -> Vec<(usize, &str)> {
        let text = self.as_ref();

        let mut parts = vec![];
        let mut start = 0;
        let mut quoted = false;
        for (offset, c) in text.char_indices() {
            if c == quote {
                quoted = !quoted;
            }
            if !quoted && c == on {
                parts.push((start, &text[start..offset]));
                start = offset + 1;
            }
        }
        parts.push((start, &text[start..]));
        parts.into_iter().filter(|(_, s)| !s.is_empty()).collect()
    }

    fn split_when_balanced(&self, on: char, quote: char) -> Vec<&str> {
        let splits = self.split_when_balanced_with_offsets(on, quote);
        splits.into_iter().map(|(_, s)| s).collect()
    }

    fn or_quotes(&self) -> String {
        let text = self.as_ref();
        String::from(match text == "" {
            true => "''",
            false => text,
        })
    }
}
