use vlq::decode;

type ResolvedOffset = i64;

#[derive(Debug)]
pub struct MappingEntry {
    pub gen_line: ResolvedOffset,
    pub gen_column: ResolvedOffset,
    pub source: Option<SourceEntry>,
}
#[derive(Debug)]
pub struct SourceEntry {
    pub source_idx: ResolvedOffset,
    pub source_line: ResolvedOffset,
    pub source_col: ResolvedOffset,
    pub name_idx: Option<ResolvedOffset>,
}

pub struct MappingData<'a> {
    internal_iterator: OffsetIterator<'a>,
    gen_column: ResolvedOffset,
    source_idx: ResolvedOffset,
    source_line: ResolvedOffset,
    source_col: ResolvedOffset,
    name_idx: ResolvedOffset,
}

enum Encountered {
    Comma,
    Semicolon,
    End,
}

struct OffsetIterator<'a> {
    data: &'a [u8],
    idx: usize,
    encountered: Encountered,
    line: ResolvedOffset,
}

impl<'a> Iterator for OffsetIterator<'a> {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<u8> {
        let idx = self.idx;
        self.idx = self.idx + 1;

        if idx == self.data.len() {
            self.encountered = Encountered::End;
            self.idx = self.idx - 1;
            None
        } else if self.data[idx] == ';' as u8 {
            self.encountered = Encountered::Semicolon;
            self.line += 1;
            None
        } else if self.data[idx] == ',' as u8 {
            self.encountered = Encountered::Comma;
            None
        } else {
            Some(self.data[idx])
        }
    }
}

impl<'a> MappingData<'a> {
    pub fn new(data: &'a [u8]) -> MappingData<'a> {
        MappingData {
            internal_iterator: OffsetIterator {
                data,
                idx: 0,
                encountered: Encountered::Semicolon,
                line: 0,
            },
            gen_column: 0,
            source_idx: 0,
            source_line: 0,
            source_col: 0,
            name_idx: 0,
        }
    }
}

impl<'a> Iterator for MappingData<'a> {
    type Item = MappingEntry;

    #[inline]
    fn next(&mut self) -> Option<MappingEntry> {
        match { decode(&mut self.internal_iterator) } {
            Err(_) => match self.internal_iterator.encountered {
                Encountered::Comma => panic!("Needs a column number in a segment"),
                Encountered::Semicolon => self.next(),
                Encountered::End => None,
            },
            Ok(gen_col_offset) => {
                {
                    self.gen_column = self.gen_column + gen_col_offset;
                }
                Some(MappingEntry {
                    gen_line: self.internal_iterator.line,
                    gen_column: self.gen_column,
                    source: { decode(&mut self.internal_iterator) }.ok().map(|source_idx_offset| {
                        self.source_idx += source_idx_offset;

                        self.source_line += {
                            decode(&mut self.internal_iterator).expect("A source list offset is always accompanied by a source line offset")
                        };

                        self.source_col += {
                            decode(&mut self.internal_iterator).expect("A source list offset is always accompanied by a source column offset")
                        };

                        SourceEntry {
                            source_idx: self.source_idx,
                            source_line: self.source_line,
                            source_col: self.source_col,
                            name_idx: decode(&mut self.internal_iterator).ok().map(|v| {
                                // consume the next character (, or ;)
                                self.internal_iterator.next() ;
                                self.name_idx += v;
                                self.name_idx
                            }),
                        }
                    })
                })
            }
        }
    }
}
#[cfg(test)]
mod test {
    use vlq::decode;
    use crate::vlq_decode::MappingData;

    #[test]
    fn test_parsing_printer() {
        let arg = "qHAQAA,EAoBI,SAAoBC,GAAAC,KAAAD,mBAPpBC,KAAAC,cAAwB,EAiBpBD,KAAKD,iBAAiBG,sBAAqB,oDCjCnDC,EAAA,WAcI,SAAAC,EAAoBL,EAA4CM,GAA5CL,KAAAD,mBAA4CC,KAAAK,WAJhEL,KAAAM,cAAkC,IAAIC,EAAA,EAEtCP,KAAAQ,gBAA0B,EAGtBR,KAAKK,SAASI,YAAYC,SAASC,gBAAiB,cACpDX,KAAKK,SAASO,SAASF,SAASC,gBAAiB,gBA4BzD,OAzBIP,EAAAS,UAAAC,SAAA,eAAAC,EAAAf,KACQgB,EAAuBhB,KAAKD,iBAAiBkB,mBAE7CD,IACAhB,KAAKkB,cAAgBF,GAGzBhB,KAAKD,iBAAiBoB,aAAc,EAEpCnB,KAAKD,iBAAiBqB,QAAQC,KAC1BC,OAAAC,EAAA,EAAAD,CAAUtB,KAAKM,eACfgB,OAAAE,EAAA,EAAAF,CAAO,SAAAG,GAAO,QAAEA,KAEfC,UAAU,SAACD,GACRV,EAAKG,cAAgBO,EACrBV,EAAKP,gBAAiB,KAIlCJ,EAAAS,UAAAc,YAAA,WACI3B,KAAKM,cAAcsB,MAAK,GACxB5B,KAAKD,iBAAiBoB,aAAc,EACpCnB,KAAKK,SAASI,YAAYC,SAASC,gBAAiB,gBACpDX,KAAKK,SAASO,SAASF,SAASC,gBAAiB,eAEzDP,EA5CA,eCMIyB,EAA2BC,EAAA,cAAUC,cAAe,EAAGC,SCL7C,KDK2EC,UAEnF,SAAAC,EAA+BC,GAAM,OAAOL,EAAA,aAAQ,KAAO,KAAM,MAEvE,IAAIM,EAAyBN,EAAA,aAAQ,UAAW3B,EAD1C,SAAoCgC,GAAM,OAAOL,EAAA,aAAQ,IAAKK,MAAQL,EAAA,aAAQ,EAAG,EAAG,KAAM,KAAM,EAAG,aAAe,KAAM,KAAM,KAAMI,EAAsBL,IAA4BC,EAAA,aAAQ,EAAG,OAAQ,KAAM,EAAG3B,GAAmBkC,EAAA,EAAqBP,EAAA,WAAe,KAAM,OAAQ,SAAUQ,EAAKC,GAAMD,EAAIC,EAAI,EAAG,IAAO,yCEepUC,EAAA,aC7BAC,EAAAC,EAAAC,EAAA,6CAAAC,IAeA,IAAIA,EAA2Bd,EAAA,aAAQhC,KAAwB,SAAUqC,GAAM,OAAOL,EAAA,cAASA,EAAA,aAAQ,IAAKA,EAAA,yBAA6BA,EAAA,2CAAuC,GAAIe,EAAA,EAAmCT,KAA8B,EAAGN,EAAA,0BAA8BA,EAAA,cAAkBA,EAAA,aAAQ,KAAMgB,EAAA,EAAmBA,EAAA,GAA0BhB,EAAA,WAAe,EAAGgB,EAAA,KAAyChB,EAAA,aAAQ,WAAYgB,EAAA,EAAiBA,EAAA,MAAsBhB,EAAA,aAAQ,WAAYiB,EAAA,EAAiBA,EAAA,IAAmB,EAAGA,EAAA,IAAwC,EAAGA,EAAA,KAAcjB,EAAA,aAAQ,WAAYU,EAA2BA,MAAgCV,EAAA,aAAQ,WAAYhC,EAAoBA,GAAqBuC,EAAA,IAAuBP,EAAA,aAAQ,KAAMiB,EAAA,EAAW,WAAc,SAAWC,KAAM,WAAYC,aAAc,kDAAqDD,KAAM,kBAAmBC,aAAc,8DAAiED,KAAM,iBAAkBC,aAAc,6DAAgED,KAAM,mBAAoBC,aAAc,iEAAoED,KAAM,eAAgBC,aAAc,2DAA8DD,KAAM,cAAeC,aAAc,wDAA2DD,KAAM,mBAAoBC,aAAc,kEAAqED,KAAM,sBAAuBC,aAAc,uEAA0ED,KAAM,YAAaC,aAAc,oDAAuDD,KAAM,eAAgBC,aAAc,0DAA6DD,KAAM,eAAgBC,aAAc,0DAA6DD,KAAM,YAAaC,aAAc,oDAAuDD,KAAM,aAAcC,aAAc,sDAAyDD,KAAM,KAAME,UAAW\\/C";
        let mut line = 0;
        let mut orig_line = 0;
        let mut orig_column = 0;
        let mut source_number = 0;
        let mut name_number = 0;

        for group in arg.split(';') {
            println!("================\nLine {}", line);

            if group.is_empty() {
                line += 1;
                continue;
            }

            let mut column = 0;
            for segment in group.split(',') {
                let mut input = segment.as_bytes().iter().cloned();

                let col_delta = decode(input.by_ref()).expect("column needed");
                column += col_delta;
                println!("   column {}", column);

                match decode(input.by_ref()) {
                    Err(_) => {}
                    Ok(s) => {
                        source_number += s;
                        println!("   source #{}", source_number);

                        let line_delta = decode(input.by_ref()).unwrap();
                        orig_line += line_delta;
                        println!("   orig line {}", orig_line);

                        let col_delta = decode(input.by_ref()).unwrap();
                        orig_column += col_delta;
                        println!("   orig column {}", orig_column);

                        match decode(input.by_ref()) {
                            Err(_) => {}
                            Ok(n) => {
                                name_number += n;
                                println!("   name #{}", name_number);
                            }
                        }
                    }
                };

                println!("");
            }

            println!("");
            line += 1;
        }
    }

    #[test]
    fn test_parsing() {
        for v in MappingData::new("qHAQAA,EAoBI,SAAoBC,GAAAC,KAAAD,mBAPpBC,KAAAC,cAAwB,EAiBpBD,KAAKD,iBAAiBG,sBAAqB,oDCjCnDC,EAAA,WAcI,SAAAC,EAAoBL,EAA4CM,GAA5CL,KAAAD,mBAA4CC,KAAAK,WAJhEL,KAAAM,cAAkC,IAAIC,EAAA,EAEtCP,KAAAQ,gBAA0B,EAGtBR,KAAKK,SAASI,YAAYC,SAASC,gBAAiB,cACpDX,KAAKK,SAASO,SAASF,SAASC,gBAAiB,gBA4BzD,OAzBIP,EAAAS,UAAAC,SAAA,eAAAC,EAAAf,KACQgB,EAAuBhB,KAAKD,iBAAiBkB,mBAE7CD,IACAhB,KAAKkB,cAAgBF,GAGzBhB,KAAKD,iBAAiBoB,aAAc,EAEpCnB,KAAKD,iBAAiBqB,QAAQC,KAC1BC,OAAAC,EAAA,EAAAD,CAAUtB,KAAKM,eACfgB,OAAAE,EAAA,EAAAF,CAAO,SAAAG,GAAO,QAAEA,KAEfC,UAAU,SAACD,GACRV,EAAKG,cAAgBO,EACrBV,EAAKP,gBAAiB,KAIlCJ,EAAAS,UAAAc,YAAA,WACI3B,KAAKM,cAAcsB,MAAK,GACxB5B,KAAKD,iBAAiBoB,aAAc,EACpCnB,KAAKK,SAASI,YAAYC,SAASC,gBAAiB,gBACpDX,KAAKK,SAASO,SAASF,SAASC,gBAAiB,eAEzDP,EA5CA,eCMIyB,EAA2BC,EAAA,cAAUC,cAAe,EAAGC,SCL7C,KDK2EC,UAEnF,SAAAC,EAA+BC,GAAM,OAAOL,EAAA,aAAQ,KAAO,KAAM,MAEvE,IAAIM,EAAyBN,EAAA,aAAQ,UAAW3B,EAD1C,SAAoCgC,GAAM,OAAOL,EAAA,aAAQ,IAAKK,MAAQL,EAAA,aAAQ,EAAG,EAAG,KAAM,KAAM,EAAG,aAAe,KAAM,KAAM,KAAMI,EAAsBL,IAA4BC,EAAA,aAAQ,EAAG,OAAQ,KAAM,EAAG3B,GAAmBkC,EAAA,EAAqBP,EAAA,WAAe,KAAM,OAAQ,SAAUQ,EAAKC,GAAMD,EAAIC,EAAI,EAAG,IAAO,yCEepUC,EAAA,aC7BAC,EAAAC,EAAAC,EAAA,6CAAAC,IAeA,IAAIA,EAA2Bd,EAAA,aAAQhC,KAAwB,SAAUqC,GAAM,OAAOL,EAAA,cAASA,EAAA,aAAQ,IAAKA,EAAA,yBAA6BA,EAAA,2CAAuC,GAAIe,EAAA,EAAmCT,KAA8B,EAAGN,EAAA,0BAA8BA,EAAA,cAAkBA,EAAA,aAAQ,KAAMgB,EAAA,EAAmBA,EAAA,GAA0BhB,EAAA,WAAe,EAAGgB,EAAA,KAAyChB,EAAA,aAAQ,WAAYgB,EAAA,EAAiBA,EAAA,MAAsBhB,EAAA,aAAQ,WAAYiB,EAAA,EAAiBA,EAAA,IAAmB,EAAGA,EAAA,IAAwC,EAAGA,EAAA,KAAcjB,EAAA,aAAQ,WAAYU,EAA2BA,MAAgCV,EAAA,aAAQ,WAAYhC,EAAoBA,GAAqBuC,EAAA,IAAuBP,EAAA,aAAQ,KAAMiB,EAAA,EAAW,WAAc,SAAWC,KAAM,WAAYC,aAAc,kDAAqDD,KAAM,kBAAmBC,aAAc,8DAAiED,KAAM,iBAAkBC,aAAc,6DAAgED,KAAM,mBAAoBC,aAAc,iEAAoED,KAAM,eAAgBC,aAAc,2DAA8DD,KAAM,cAAeC,aAAc,wDAA2DD,KAAM,mBAAoBC,aAAc,kEAAqED,KAAM,sBAAuBC,aAAc,uEAA0ED,KAAM,YAAaC,aAAc,oDAAuDD,KAAM,eAAgBC,aAAc,0DAA6DD,KAAM,eAAgBC,aAAc,0DAA6DD,KAAM,YAAaC,aAAc,oDAAuDD,KAAM,aAAcC,aAAc,sDAAyDD,KAAM,KAAME,UAAW\\/C".as_bytes()) {
            println!("{:?}", v);
        }
    }
}
