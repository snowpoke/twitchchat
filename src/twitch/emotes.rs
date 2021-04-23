/*!
Emotes are little pictograms used in-line in Twitch messages

They are presented (to the irc connection) in a `id:range1,range2/id2:range1,..` form which marks the byte position that the emote is located.

# example:
`"testing Kappa"` would be `25:8-13`

`"Kappa testing Kappa"` would be `25:0-5,14-19`
*/

use std::ops::Range;
use crate::twitch::attributes::{RangePosition, SeparatorInfo, Attribute};

/// Emotes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct Emotes {
    /// This emote id, e.g. `Kappa = 25`    
    pub id: usize,
    /// A list of [Range] in the message where this emote is found
    ///
    /// [Range]: https://doc.rust-lang.org/std/ops/struct.Range.html
    pub ranges: Vec<Range<u16>>,
}

impl Attribute<usize> for Emotes {
    fn new(
        ranges: impl Iterator<Item = Range<u16>>,
        mut attributes: impl Iterator<Item = usize>,
    ) -> Option<Self> {
        Emotes {
            id: attributes.next()?, // attributes will only ever have one element
            ranges: ranges.collect(),
        }.into()
    }

    fn get_separator_info() -> SeparatorInfo {
        SeparatorInfo {
            element_separator: '/',
            range_attribute_separator: ':',
            attribute_separator: '\n', //never matches
            range_separator: ',',
            range_position: RangePosition::Right,
        }
    }

    // emotes are represented as just numbers, so we just use &str::parse
    fn parse_attribute(input: impl AsRef<str>) -> Option<usize>{
        input.as_ref().parse::<usize>().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        macro_rules! emote {
            ($id:expr, $($r:expr),* $(,)?) => {
                Emotes {
                    id: $id,
                    ranges: vec![$($r),*]
                }
            };
        }

        let inputs = &[
            (
                "25:0-4,6-10,12-16",
                vec![emote!(25, (0..4), (6..10), (12..16))],
            ),
            (
                "25:0-4", //
                vec![emote!(25, (0..4))],
            ),
            (
                "1077966:0-6/25:8-12",
                vec![emote!(1_077_966, (0..6)), emote!(25, (8..12))],
            ),
            (
                "25:0-4,6-10/33:12-19",
                vec![emote!(25, (0..4), (6..10)), emote!(33, (12..19))],
            ),
            (
                "25:0-4,15-19/33:6-13",
                vec![emote!(25, (0..4), (15..19)), emote!(33, (6..13))],
            ),
            (
                "33:0-7/25:9-13,15-19",
                vec![emote!(33, (0..7)), emote!(25, (9..13), (15..19))],
            ),
        ];

        for (input, expect) in inputs {
            let emotes = Emotes::parse(input).collect::<Vec<_>>();
            assert_eq!(emotes.len(), expect.len());
            assert_eq!(emotes, *expect);
        }
    }
}
