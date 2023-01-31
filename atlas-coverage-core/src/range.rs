use std::cmp::Ordering;
use std::iter::FromIterator;
use std::{cmp, fmt};

type Value = usize;

#[derive(PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Range {
    start: Value,
    end: Value,
}

impl Range {
    fn overlaps(&self, other: &Range) -> bool {
        (other.start >= self.start && other.start <= self.end)
            || (other.end >= self.start && other.end <= self.end)
    }

    fn merge(&mut self, other: &Range) {
        self.start = cmp::min(self.start, other.start);
        self.end = cmp::max(self.end, other.end);
    }
}

impl Range {
    fn compare_to_item(self, value: Value) -> Ordering {
        match self.start.cmp(&value) {
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => Ordering::Equal,
            Ordering::Less => match self.end.cmp(&value) {
                Ordering::Greater => Ordering::Equal,
                Ordering::Equal => Ordering::Equal,
                Ordering::Less => Ordering::Less,
            },
        }
    }
}

impl fmt::Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{},{}]", self.start, self.end)
    }
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct RangeStack {
    pub ranges: Vec<Range>,
}

impl RangeStack {
    fn add(&mut self, range: &Range) {
        if let Some(last) = self.ranges.last_mut() {
            if last.overlaps(range) {
                last.merge(range);
                return;
            }
        }

        self.ranges.push(*range);
    }

    pub fn contains_value(&self, value: Value) -> bool {
        self.ranges
            .binary_search_by(|ref range| range.compare_to_item(value))
            .is_ok()
    }
}

impl fmt::Display for RangeStack {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for range in &self.ranges {
            write!(f, " {}", range)?;
        }
        Ok(())
    }
}

impl FromIterator<Range> for RangeStack {
    fn from_iter<I>(iterator: I) -> Self
    where
        I: IntoIterator<Item = Range>,
    {
        let mut raw_ranges: Vec<_> = iterator.into_iter().collect();
        raw_ranges.sort_by(|a, b| a.start.cmp(&b.start));

        let mut range_stack = RangeStack { ranges: Vec::new() };

        for range in &raw_ranges {
            range_stack.add(range);
        }

        range_stack
    }
}

impl<'a> FromIterator<&'a Range> for RangeStack {
    fn from_iter<I>(iterator: I) -> Self
    where
        I: IntoIterator<Item = &'a Range>,
    {
        iterator.into_iter().cloned().collect()
    }
}

#[cfg(test)]
mod test {
    use super::Range;
    use std::cmp::Ordering;

    #[test]
    fn test_compare_to_item() {
        let range = Range { start: 5, end: 10 };
        assert_eq!(range.compare_to_item(0), Ordering::Greater);
        assert_eq!(range.compare_to_item(4), Ordering::Greater);
        assert_eq!(range.compare_to_item(5), Ordering::Equal);
        assert_eq!(range.compare_to_item(10), Ordering::Equal);
        assert_eq!(range.compare_to_item(11), Ordering::Less);
        assert_eq!(range.compare_to_item(200), Ordering::Less);
    }
}
