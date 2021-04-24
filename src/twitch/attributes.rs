//! Helper functions and a trait for data of a structure similar to Emotes or Flag.
//! Those structures provide additional information about parts of a message, and conveys them using the range that the information applies to.
use std::iter::FilterMap;
use std::ops::Range;
use std::str::FromStr;
use std::str::Split;

/// Relative position of the range in an attribute string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub enum RangePosition {
    /// Range is left to the attribute.
    Left,
    /// Range is right to the attribute.
    Right,
}

/// We need to supply separators based on which the string will be split apart:
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct SeparatorInfo {
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

/// Trait that applies information stored in a tag that adds attribute information to specific parts of a message. (like emote interpretation)
pub trait Attribute<T>: Sized
where
    T: FromStr,
{
    /// Given a vector of ranges and attributes, creates an element of itself.
    fn new(
        ranges: impl Iterator<Item = Range<u16>>,
        attributes: impl Iterator<Item = T>,
    ) -> Option<Self>;

    /// When implementing this attribute, this function gives all necessery information on how to parse the original string.
    fn get_separator_info() -> SeparatorInfo;

    /// Returns separator between elements.
    fn get_element_separator() -> char {
        Self::get_separator_info().element_separator
    }

    /// Returns separator between a range and an attribute.
    fn get_range_attribute_separator() -> char {
        Self::get_separator_info().range_attribute_separator
    }

    /// Returns separator between attributes.
    fn get_attribute_separator() -> char {
        Self::get_separator_info().attribute_separator
    }

    /// Returns separator between ranges.
    fn get_range_separator() -> char {
        Self::get_separator_info().range_separator
    }

    /// Returns the relative position of the range.
    fn get_range_position() -> RangePosition {
        Self::get_separator_info().range_position
    }

    /// Parses the range information.
    fn parse_ranges(input: impl AsRef<str>) -> Vec<Range<u16>> {
        input
            .as_ref()
            .split(Self::get_range_separator())
            .filter_map(|s| split_pair(s, '-'))
            .filter_map(move |(start, end)| {
                let (start, end) = (start.parse().ok()?, end.parse().ok()?);
                Range { start, end }.into()
            })
            .collect()
    }

    /// Parses the attribute information.
    fn parse_attributes(input: impl AsRef<str>) -> Vec<T> {
        input
            .as_ref()
            .split(Self::get_attribute_separator())
            .filter_map(|x| T::from_str(x).ok())
            .collect()
    }

    /// Parses a single attribute.
    fn parse_item(item: &str) -> Option<Self> {
        split_pair(item, Self::get_range_attribute_separator()).and_then(|(left, right)| {
            // ranges are on the side denoted by the Either element
            let (ranges, attributes) = match Self::get_range_position() {
                RangePosition::Left => (left, right),
                RangePosition::Right => (right, left),
            };
            let (ranges, attributes) = (
                Self::parse_ranges(ranges),
                Self::parse_attributes(attributes),
            );
            Self::new(ranges.into_iter(), attributes.into_iter())
        })
    }

    /// Parse is generated based on the functions provided above.
    /// The input type here has to be &str to assure knowledge about the lifetime and ownership.
    #[allow(clippy::type_complexity)]
    fn parse(input: &str) -> FilterMap<Split<'_, char>, fn(&str) -> Option<Self>> {
        input
            .split(Self::get_element_separator())
            .filter_map(Self::parse_item)
    }
}

/// Splits a string into a pair of strings based on a separator.
/// If the separator is not found, then full string is first element, second element is an empty string.
pub(crate) fn split_pair<'a>(input: impl AsRef<str> + 'a, sep: char) -> Option<(String, String)> {
    let mut split = input.as_ref().split(sep);
    (
        split.next()?.into(),
        split.next().unwrap_or_default().into(),
    )
        .into()
}

/// We can use Attribute::parse as a from_str function for a vector of attributes, but we have to define a custom vector format to do so.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct AttributeVec<S: FromStr, T: Attribute<S>>(Vec<T>, std::marker::PhantomData<S>);

impl<S, T> AttributeVec<S, T>
where
    S: FromStr,
    T: Attribute<S>,
{
    /// Returns the vector stored in AttributeVec.
    pub fn take(self) -> Vec<T> {
        self.0
    }

    /// Generates an AttributeVec from a vector.
    pub fn from_vec(vec: Vec<T>) -> Self {
        Self(vec, std::marker::PhantomData)
    }
}

impl<S, T> FromStr for AttributeVec<S, T>
where
    S: FromStr,
    T: Attribute<S>,
{
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(AttributeVec(
            <T as Attribute<S>>::parse(s).collect::<Vec<_>>(),
            std::marker::PhantomData,
        ))
    }
}
