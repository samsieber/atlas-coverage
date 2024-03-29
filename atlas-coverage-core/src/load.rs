use std::fmt;
use crate::model::PuppeteerData;
use serde::de::SeqAccess;
use serde::de::Visitor;
use serde::Deserializer;
use serde_json::Deserializer as JsonDeserializer;
use std::collections::HashMap;

use crate::load;
use crate::model;
use crate::range::Range;
use std::path::Path;
use crate::util;
use serde_json::Error;

pub struct RawCoveragePart {
    text: String,
    ranges: Vec<Range>,
}
pub type RawCoverage = HashMap<String, RawCoveragePart>;

pub fn load_items<P: AsRef<Path>>(paths: Vec<P>) -> Loader{
    let mut loader = load::Loader::new();

    for p in paths {
        let raw_content = match util::fast_read(p.as_ref()){
            Ok(value) => {
                eprintln!("Loading {}", p.as_ref().to_string_lossy());
                value
            },
            Err(err) => {
                eprintln!("Cannot load {} - {:#?}", p.as_ref().to_string_lossy(), err);
                continue
            },
        };
        match loader.add_json_data(&mut JsonDeserializer::from_slice(&raw_content.as_bytes())) {
            Ok(_) => {
                eprintln!("Loaded {}", p.as_ref().to_string_lossy());
            },
            Err(error) => {
                eprintln!("Could not parse json for {} - {:#?}", p.as_ref().to_string_lossy(), error);
            },
        };
    }

    loader
}

pub struct Loader {
    parts: RawCoverage,
}

struct VisitorAppender<'a> {
    parts: &'a mut RawCoverage,
}

impl<'a> VisitorAppender<'a> {
    #[inline]
    fn add_data(&mut self, mut data: PuppeteerData) {
        let inserted = if let Some(existing_data) = self.parts.get_mut(&data.url) {
            existing_data.ranges.append(&mut data.ranges.ranges);
            true
        } else {
            false
        };

        if !inserted {
            self.parts.insert(
                data.url,
                RawCoveragePart {
                    text: data.text,
                    ranges: data.ranges.ranges,
                },
            );
        }
    }
}

impl Loader {
    #[inline]
    pub fn new() -> Loader {
        Loader {
            parts: HashMap::new(),
        }
    }

    #[inline]
    pub fn add_json_data<'de>(
        &mut self,
        deserializer: &mut JsonDeserializer<impl ::serde_json::de::Read<'de>>,
    ) -> Result<(), Error>{
        let visitor: VisitorAppender<'_> = VisitorAppender {
            parts: &mut self.parts,
        };
        Ok(deserializer.deserialize_seq(visitor)?)
    }
}

fn convert_to_puppeteer_data((key, value): (String, RawCoveragePart)) -> PuppeteerData {
    PuppeteerData {
        url: key,
        text: value.text,
        ranges: value.ranges.into_iter().collect(),
    }
}

impl IntoIterator for Loader {
    type Item = PuppeteerData;
    // This is SOOOOOOOO ugly, but at least usable
    type IntoIter = ::std::iter::Map<
        ::std::collections::hash_map::IntoIter<::std::string::String, load::RawCoveragePart>,
        fn((String, load::RawCoveragePart)) -> model::PuppeteerData,
    >;

    fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
        self.parts.into_iter().map(convert_to_puppeteer_data)
    }
}

impl<'va, 'de: 'va> Visitor<'de> for VisitorAppender<'va> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "expected a list of puppeteer coverage data")
    }

    fn visit_seq<S>(self, mut seq: S) -> Result<(), S::Error>
    where
        S: SeqAccess<'de>,
    {
        let mut collapser = self;

        while let Some(value) = seq.next_element()? {
            collapser.add_data(value);
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::load::Loader;
    use crate::model::PuppeteerData;
    use serde_json::Deserializer as JsonDeserializer;

    #[test]
    fn test_load_iter() {
        let data = r#"[
            {
                "text": "Testing\n1 2 1 2",
                "url": "1",
                "ranges": [{
                    "start": 1,
                    "end": 2
                },{
                    "start": 4,
                    "end": 5
                }]
            },
            {
                "text": "Testing\n3 4 3 4",
                "url": "2",
                "ranges": [{
                    "start": 1,
                    "end": 2
                },{
                    "start": 4,
                    "end": 5
                }]
            },
            {
                "text": "Testing\n5 6 7 8",
                "url": "1",
                "ranges": [{
                    "start": 0,
                    "end": 2
                },{
                    "start": 4,
                    "end": 7
                }]
            }
        ]"#;

        let combined_data = r#"[
            {
                "text": "Testing\n1 2 1 2",
                "url": "1",
                "ranges": [{
                    "start": 0,
                    "end": 2
                },{
                    "start": 4,
                    "end": 7
                }]
            },
            {
                "text": "Testing\n3 4 3 4",
                "url": "2",
                "ranges": [{
                    "start": 1,
                    "end": 2
                },{
                    "start": 4,
                    "end": 5
                }]
            }
        ]"#;

        let mut deserializer = JsonDeserializer::from_slice(data.as_bytes());
        let mut loader = Loader::new();

        loader.add_json_data(&mut deserializer).unwrap();

        let mut combined: Vec<PuppeteerData> = ::serde_json::from_str(combined_data).unwrap();
        let mut loaded = loader.into_iter().collect::<Vec<PuppeteerData>>();

        combined.sort_by_key(|v| v.url.clone());
        loaded.sort_by_key(|v| v.url.clone());

        assert_eq!(combined, loaded)
    }
}
