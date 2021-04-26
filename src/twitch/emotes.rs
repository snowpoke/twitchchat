/*!
Emotes are little pictograms used in-line in Twitch messages

They are presented (to the irc connection) in a `id:range1,range2/id2:range1,..` form which marks the byte position that the emote is located.

# example:
`"testing Kappa"` would be `25:8-13`

`"Kappa testing Kappa"` would be `25:0-5,14-19`
*/

use crate::twitch::attributes::{Attribution, SeparatorInfo, MsgRange, AttributionVec};
use std::str::FromStr;

/// Emotes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct Emote {
    /// This emote id, e.g. `Kappa = 25`    
    pub id: usize,
    /// A list of [Range] in the message where this emote is found
    ///
    /// [Range]: https://doc.rust-lang.org/std/ops/struct.Range.html
    pub ranges: Vec<MsgRange>,
}

impl Attribution<usize, MsgRange> for Emote {
    fn new(
        reference: usize,
        attributes: impl Iterator<Item = MsgRange>,
    ) -> Self {
        Self {
            id: reference,
            ranges: attributes.collect(),
        }
    }

    fn get_separator_info() -> SeparatorInfo {
        SeparatorInfo {
            attribution_separator: '/',
            range_attribute_separator: ':',
            attribute_separator: ',', 
        }
    }
}

impl FromStr for Emote {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        <Emote as Attribution<usize, MsgRange>>::parse(s).ok_or(())
    }
}

/// Vector containing emote attribution data.
pub type EmoteVec = AttributionVec<usize, MsgRange, Emote>;

/// An iterator over emotes
// #[derive(Debug, Constructor)]
// pub struct EmotesIter<'a> {
//     items: Option<std::str::SplitTerminator<'a, char>>,
// }

// impl<'a> Iterator for EmotesIter<'a> {
//     type Item = Emote;

//     fn next(&mut self) -> Option<Self::Item> {
//         if let Some(item) = self.items.as_mut()?.next() {
//             Emote::parse_item(item)
//         } else {
//             None
//         }
//     }
// }

// /// Parse emotes into iterator.
// #[allow(dead_code)]
// pub fn parse_emotes_iter(input: &str) -> impl Iterator<Item = Emote> + '_ {
//     Emote::parse(input)
// }



#[cfg(test)]
mod tests {
    use super::*;
    use crate::twitch::EmoteVec;
    use std::str::FromStr;

    #[test]
    fn parse() {
        macro_rules! emote {
            ($id:expr, $($r:expr),* $(,)?) => {
                Emote {
                    id: $id,
                    ranges: vec![$($r.into()),*]
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
            let emotes = EmoteVec::from_str(input).unwrap();
            assert_eq!(emotes.len(), expect.len());
            assert_eq!(*emotes, *expect);
        }
    }
}
