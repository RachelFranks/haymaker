//
// Haymaker
//

#[derive(Clone, Debug, Default)]
pub struct LineInfo<'a> {
    pub line: &'a str,
    pub sans_flags: &'a str,
    pub full_line: &'a str,
    pub shell: bool,
    pub debug: bool,
    pub silence: bool,
    pub neglect: bool,
    pub split: usize,
}

impl<'a> From<&'a str> for LineInfo<'a> {
    fn from(line: &'a str) -> Self {
        let mut info = LineInfo::default();

        for (index, (offset, c)) in line.char_indices().enumerate() {
            if c.is_whitespace() {
                info.shell = true;
                continue;
            }
            match c {
                '+' => info.debug = true,
                '-' => info.silence = true,
                '^' => info.neglect = true,
                _ => {
                    info.sans_flags = &line[offset..];
                    info.split = index;
                    break;
                }
            }
        }
        info.full_line = line;
        info
    }
}

#[test]
fn test_lines() {
    #[rustfmt::skip]
    let cases = [
        ("nothing + should - happen ^", "nothing + should - happen ^", false, false, false, false),
        ("+debug -not silent", "debug -not silent", false, true, false, false),
        (" -+ ^debug -silent", "debug -silent", true, true, true, true),
    ];

    for (case, sans_flags, shell, debug, silence, neglect) in cases {
        let line = LineInfo::from(case);
        assert_eq!(line.shell, shell);
        assert_eq!(line.debug, debug);
        assert_eq!(line.silence, silence);
        assert_eq!(line.neglect, neglect);
        assert_eq!(line.sans_flags, sans_flags);
    }
}
