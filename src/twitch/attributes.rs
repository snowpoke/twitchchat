//! Helper functions and a trait for data of a structure similar to Emotes or Flag.
//! Those structures provide additional information about parts of a message, and conveys them using the range that the information applies to.
use std::ops::Range;
use std::iter::FilterMap;
    use std::str::Split;


pub(crate) enum RangePosition{
    Left,
    Right,
}   

/// We need to supply separators based on which the string will be split apart:
pub(crate) struct SeparatorInfo {
    /// Separator between elements.
    pub(crate) element_separator: char,
    /// Separator between range and attribute data.
    pub(crate) range_attribute_separator: char,
    /// Separator between attributes.
    pub(crate) attribute_separator: char,
    /// Separator between ranges.
    pub(crate) range_separator: char,
    /// Position of range relative to attribute.
    pub(crate) range_position: RangePosition,
}


pub(crate) trait Attribute<T>: Sized{
    /// Given a vector of ranges and attributes, creates an element of itself.
    fn new(
        ranges: impl Iterator<Item = Range<u16>>,
        attributes: impl Iterator<Item = T>,
    ) -> Option<Self>;

    /// When implementing this attribute, this function gives all necessery information on how to parse the original string.
    fn get_separator_info() -> SeparatorInfo;
    
    /// Parses a single attribute (Like an emote, or a score)
    fn parse_attribute(input: impl AsRef<str>) -> Option<T>;


    fn get_element_separator() -> char{
        Self::get_separator_info().element_separator
    }
    
    fn get_range_attribute_separator() -> char{
        Self::get_separator_info().range_attribute_separator
    }
    
    fn get_attribute_separator() -> char{
        Self::get_separator_info().attribute_separator
    }
    
    fn get_range_separator() -> char{
        Self::get_separator_info().range_separator
    }

    fn get_range_position() -> RangePosition{
        Self::get_separator_info().range_position
    }

    fn parse_ranges(input: impl AsRef<str>) -> Vec<Range<u16>> {
        input.as_ref().split(Self::get_range_separator())
            .filter_map(|s| split_pair(s, '-'))
            .filter_map(move |(start, end)| {
                let (start, end) = (start.parse().ok()?, end.parse().ok()?);
                Range { start, end }.into()
            })
            .collect()
    }

    fn parse_attributes(input: impl AsRef<str>) -> Vec<T>{
        input.as_ref().split(Self::get_attribute_separator())
            .filter_map(Self::parse_attribute)
            .collect()
    }

    // Parse single emote
    fn parse_item(item: &str) -> Option<Self> {
        split_pair(item, Self::get_range_attribute_separator()).and_then(|(left, right)| {
            // ranges are on the side denoted by the Either element
            let (ranges, attributes) = match Self::get_range_position() {
                RangePosition::Left => (left, right),
                RangePosition::Right => (right, left),
            };
            let (ranges, attributes) = (Self::parse_ranges(ranges), Self::parse_attributes(attributes));
            Self::new(ranges.into_iter(), attributes.into_iter())
        })
    }

    /// Parse is generated based on the functions provided above.
    /// The input type here has to be &str to assure knowledge about the lifetime and ownership.
    #[allow(clippy::type_complexity)]
    fn parse(input: &str) -> FilterMap<Split<'_, char>, fn(&str) -> Option<Self>>{
        input.split(Self::get_element_separator()).filter_map(Self::parse_item)
    }
}

/// Splits a string into a pair of strings based on a separator.
/// If the separator is not found, then full string is first element, second element is an empty string.
pub(crate) fn split_pair<'a>(input: impl AsRef<str> + 'a, sep: char) -> Option<(String, String)> {
    let mut split = input.as_ref().split(sep);
    (split.next()?.into(), split.next().unwrap_or_default().into()).into()
}