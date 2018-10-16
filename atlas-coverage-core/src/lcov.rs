use lcov_parser:: { LCOVRecord };

/// Iterator yielding every line in a string. The line includes newline character(s).
pub struct LinesWithEndings<'a> {
    input: &'a str,
}

impl<'a> LinesWithEndings<'a> {
    pub fn from(input: &'a str) -> LinesWithEndings<'a> {
        LinesWithEndings {
            input: input,
        }
    }
}

impl<'a> Iterator for LinesWithEndings<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<&'a str> {
        if self.input.is_empty() {
            return None;
        }
        let split = self.input.find('\n').map(|i| i + 1).unwrap_or(self.input.len());
        let (line, rest) = self.input.split_at(split);
        self.input = rest;
        Some(line)
    }
}


pub struct LcovFilesLines<'a> {
    lines: LinesWithEndings<'a>,
    next_file: Option<String>,
}

pub struct FileLines {
    pub file_path: String,
    pub lines: Vec<usize>,
}

impl <'a> LcovFilesLines<'a> {
    fn empty(lines: LinesWithEndings<'a>) -> LcovFilesLines<'a> {
        LcovFilesLines {
            lines, next_file: None,
        }
    }

    pub fn new(input: &'a str) -> LcovFilesLines<'a>{
        let mut lines = LinesWithEndings::from(input);

        loop {
            let line = lines.next();

            match line {
                Some(line) => {
//                    eprintln!("{}", &line);
                    match LCOVRecord::from(line) {
                        LCOVRecord::SourceFile(src_file) => {
                            return LcovFilesLines {
                                lines,
                                next_file: Some(src_file),
                            }
                        },
                        _ => {}
                    }
                },
                None => return Self::empty(lines)
            }
        }
    }

    #[inline]
    fn read_to_next_file(&mut self) -> Vec<usize>{
        let mut lines = vec!();
        loop {
            let line = self.lines.next();
            match line {
                Some(line) => {
//                    eprintln!("{}", &line);
                    match LCOVRecord::from(line) {
                        LCOVRecord::SourceFile(src_file) => {
                            self.next_file = Some(src_file);
                            return lines
                        },
                        LCOVRecord::Data(line_data) => {
                            lines.push(line_data.line as usize);
                        },
                        _ => {}
                    }
                },
                None => {
                    self.next_file = None;
                    return lines;
                }
            }
        }
    }
}

impl <'a> Iterator for LcovFilesLines<'a> {
    type Item = FileLines;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        let next_file = self.next_file.take();

        next_file.map(|next_file| {
            FileLines {
                file_path: next_file,
                lines: self.read_to_next_file(),
            }
        })
    }
}