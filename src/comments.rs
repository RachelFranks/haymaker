use itertools::Itertools;

pub fn uncomment(text: &str, blank: &str) -> Vec<String> {
    let lines = text.split('\n');

    let mut output = vec![];
    let mut scopes = 0_usize;

    for source_line in lines {
        let mut line = String::with_capacity(source_line.len());
        let mut ignore_scope_changes = false;
        let mut chars = source_line.chars().chain(std::iter::once(' ')).tuple_windows();

        while let Some((c, n)) = chars.next() {
            if !ignore_scope_changes {
                if c == '/' && n == '*' {
                    scopes += 1;
                    line = line + blank + blank;
                    chars.next(); // skip the star
                    continue;
                }

                if c == '*' && n == '/' {
                    if scopes != 0 {
                        scopes -= 1;
                        line = line + blank + blank;
                        chars.next();
                        continue;
                    }
                }
            }

            if c == '#' || (c == '/' && n == '/') {
                match scopes {
                    0 => break,
                    _ => ignore_scope_changes = true,
                }
            }

            match scopes {
                0 => line.push(c),
                _ => line += blank,
            }
        }
        output.push(line);
    }

    output
}

#[test]
fn test_comments() {
    use crate::console::Color;
    let hayfile = std::fs::read_to_string("tests/comments.hay").unwrap();
    let txtfile = std::fs::read_to_string("tests/comments.txt").unwrap();

    let hay_lines = uncomment(&hayfile, "-");
    let txt_lines = txtfile.split('\n');

    for (offset, (line, correct)) in hay_lines.iter().zip(txt_lines).enumerate() {
        if line != correct {
            println!("line {} is not the same", offset.to_string().red());
            println!("expected: {}\nobserved: {}", correct, line.red());
            panic!();
        }
        println!("line {}\n{}\n{}\n", offset + 1, line, correct);
    }
}
