//! Twitch runs their automod analysis over every message and assigns flags if necessary.
//! The flags contain a character range that indicates the offending term and a score. (Note: Apparently links and drug related terms also get flagged, but not assigned with a score.)
//! The scores take the shape [type].[severity], examples would be I.4, P.3, S.5, A.3
//!
//! The following types are defined:
//! 'A': Aggression
//! 'I': Identity Language
//! 'P': Profanity
//! 'S': Sexual Language
//!
//! A term can be flagged as multiple types, they are then separated by forward slashes '/'.
//! A message can have multiple flags, they are separated by commas ','.
//!
//! Example flags:
//! Message: "50K LMAOO" -- Flags: "4-8:P.3"
//! Message: "I have a spaz" - Flags: "9-12:A.6/I.6"
//! Message: "SHES HOT AF FR" -- Flags: "9-10:P.5"
//! Message: "THATS A CREEP" -- Flags: "8-12:A.6"
//! Message: "she hottie" -- Flags: "4-9:S.3"
//! Message: "LMAO Poki wtf" -- Flags: "0-3:P.6,10-12:P.6"

use crate::twitch::attributes::{split_pair, Attribute, RangePosition, SeparatorInfo};
use std::ops::Range;
use std::str::FromStr;

/// The four possible types of offensive terms recognized by Twitch
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub enum ScoreType {
    Aggressive,
    Identity,
    Profanity,
    Sexual,
}

/// A score that was assigned to a term by automod. Like A.6, S.3, etc.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
struct Score(ScoreType, u8);

/// Contains information about a flagged term.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct Flag {
    range: Range<u16>,
    scores: Vec<Score>,
}

impl FromStr for Score {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (score_type, score) = split_pair(s, '.').ok_or(())?;
        let score_type = match score_type.as_ref() {
            "A" => ScoreType::Aggressive,
            "I" => ScoreType::Identity,
            "P" => ScoreType::Profanity,
            "S" => ScoreType::Sexual,
            _ => return Err(()),
        };

        Ok(Score(score_type, score.parse::<u8>().map_err(|_| ())?))
    }
}
impl Attribute<Score> for Flag {
    fn new(
        mut ranges: impl Iterator<Item = Range<u16>>,
        attributes: impl Iterator<Item = Score>,
    ) -> Option<Self> {
        Self {
            range: ranges.next()?,
            scores: attributes.collect(),
        }
        .into()
    }

    fn get_separator_info() -> SeparatorInfo {
        SeparatorInfo {
            element_separator: ',',
            range_attribute_separator: ':',
            attribute_separator: '/',
            range_separator: '\n', // never matches
            range_position: RangePosition::Left,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AGGRESSIVE: ScoreType = ScoreType::Aggressive;
    const IDENTITY: ScoreType = ScoreType::Identity;
    const PROFANE: ScoreType = ScoreType::Profanity;
    const SEXUAL: ScoreType = ScoreType::Sexual;

    #[test]
    fn parse() {
        let inputs = &[
            (
                "4-8:P.3",
                vec![Flag {
                    range: 4..8,
                    scores: vec![Score(PROFANE, 3)],
                }],
            ),
            (
                "9-12:A.6/I.6",
                vec![Flag {
                    range: 9..12,
                    scores: vec![Score(AGGRESSIVE, 6), Score(IDENTITY, 6)],
                }],
            ),
            (
                "9-10:P.5",
                vec![Flag {
                    range: 9..10,
                    scores: vec![Score(PROFANE, 5)],
                }],
            ),
            (
                "8-12:A.6",
                vec![Flag {
                    range: 8..12,
                    scores: vec![Score(AGGRESSIVE, 6)],
                }],
            ),
            (
                "4-9:S.3",
                vec![Flag {
                    range: 4..9,
                    scores: vec![Score(SEXUAL, 3)],
                }],
            ),
            (
                "0-3:P.6,10-12:P.6",
                vec![
                    Flag {
                        range: 0..3,
                        scores: vec![Score(PROFANE, 6)],
                    },
                    Flag {
                        range: 10..12,
                        scores: vec![Score(PROFANE, 6)],
                    },
                ],
            ),
            (
                "0-3",
                vec![Flag {
                    range: 0..3,
                    scores: vec![],
                }],
            ),
        ];

        for (input, expect) in inputs {
            let flags = Flag::parse(input).collect::<Vec<_>>();
            assert_eq!(flags.len(), flags.len());
            assert_eq!(flags, *expect);
        }
    }
}
