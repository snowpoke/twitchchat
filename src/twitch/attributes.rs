//! Traits and Structs that can be used to process tags that themselves contain lists of information.
//! Those lists add information or interpretation to messages and senders, and are expressed in somewhat consistent formats.

use std::ops::Range;
use std::str::FromStr;
use std::default::Default;
use derive_more::{Deref, From, Constructor};
use parse_display::{Display, FromStr};
use std::iter::FilterMap;
use std::str::Split;

/// We need to supply separators based on which the string will be split apart:
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct SeparatorInfo {
    /// Separator between attributions.
    pub(crate) attribution_separator: char,
    /// Separator between range and attribute data.
    pub(crate) range_attribute_separator: char,
    /// Separator between attributes.
    pub(crate) attribute_separator: char,
}

/// Like range, but implements FromStr.
/// Indicates character ranges in Twitch messages.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deref, From, Display, FromStr, Default)]
#[from(forward)]
#[display("{0.start}-{0.end}")] // this also defines from_str
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct MsgRange(
    #[from_str(default)]
    Range<u16>
);

/// Trait that applies information stored in a tag that adds attribute information to specific parts of a message. (like emote interpretation)
pub trait Attribution<Ref, Attr>: Sized
where
    Ref: FromStr, // reference to the object that is being described
    Attr: FromStr, // attribute that is given to the referenced object
{
    /// Given a vector of object references and attributes, creates an element of itself.
    fn new(
        reference: Ref,
        attributes: impl Iterator<Item = Attr>,
    ) -> Self;

    /// When implementing this attribute, this function gives all necessery information on how to parse the original string.
    fn get_separator_info() -> SeparatorInfo;

    /// Returns separator between a range and an attribute.
    fn get_range_attribute_separator() -> char {
        Self::get_separator_info().range_attribute_separator
    }

    /// Returns separator between attributes.
    fn get_attribute_separator() -> char {
        Self::get_separator_info().attribute_separator
    }

    /// Returns separator between attributions
    fn get_attribution_separator() -> char {
        Self::get_separator_info().attribution_separator
    }

    /// Parses the attribute information.
    #[allow(clippy::type_complexity)]
    fn parse_attributes(input: &'_ str) -> FilterMap<Split<'_,char>, fn(&str)->Option<Attr>> {
        input
            .split(Self::get_attribute_separator())
            .filter_map(|x| Attr::from_str(x).ok())
    }

    /// Parses a single attribution.
    fn parse(item: &str) -> Option<Self> {
        split_pair(item, Self::get_range_attribute_separator()).and_then(|(left, right)| {
            Self::new(<Ref as FromStr>::from_str(&left).ok()?, Self::parse_attributes(&right)).into()
        })
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

/// We wrap the iterator of attributions so that we can define from_str on it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Deref, Constructor)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct AttributionVec<Ref: FromStr, Attr: FromStr, T: Attribution<Ref, Attr>>{
    #[deref]
    element: Vec<T>,
    _phantom_data: std::marker::PhantomData<Ref>,
    _phantom_data_2: std::marker::PhantomData<Attr>,
}

impl<Ref, Attr, T> From<Vec<T>> for AttributionVec<Ref, Attr, T>
where
    Ref: FromStr,
    Attr: FromStr,
    T: Attribution<Ref, Attr>,
{
    fn from(v: Vec<T>) -> Self {
        AttributionVec::<_,_,_>::new(v, std::marker::PhantomData, std::marker::PhantomData)
    }
}

impl<Ref, Attr, T> FromStr for AttributionVec<Ref, Attr, T>
where
    Ref: FromStr,
    Attr: FromStr,
    T: Attribution<Ref, Attr>,
{
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split_char = <T as Attribution<Ref, Attr>>::get_attribution_separator();
        Ok(
                s.split(split_char)
                .filter_map(<T as Attribution<Ref, Attr>>::parse)
                .collect::<Vec<_>>()
                .into()
        )
    }
}
