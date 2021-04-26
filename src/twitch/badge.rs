#![allow(missing_docs)]
/// The kind of the [badges] that are associated with messages.
///
/// Any unknown (e.g. custom badges/sub events, etc) are placed into the [Unknown] variant.
///
/// [badges]: Badge
/// [Unknown]: BadgeKind::Unknown

use derive_more::IsVariant;
use crate::twitch::attributes::{Attribution, SeparatorInfo, AttributionVec};
use parse_display::{Display, FromStr};
use std::str::FromStr;

/// Describes the kind of badge owned by the user.
#[non_exhaustive]
#[derive(Display, FromStr, Debug, Clone, PartialEq, Eq, Hash, IsVariant)]
#[display(style = "kebab-case")] // this also defines the FromStr style
#[display("{}/1")]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub enum Badge{
    /// Admin badge
    Admin,
    /// Broadcaster badge
    Broadcaster,
    /// Moderator badge
    Moderator,
    /// Staff badge
    Staff,
    /// Turbo badge
    Turbo,
    /// Premium badge
    Premium,
    /// VIP badge
    Vip,
    /// Partner badge
    Partner,

    /// Bits badge
    #[display("bits/{0}")]
    Bits(u64), //u64: number of bits

    /// GlobalMod badge
    #[display("global_mod/1")] // legacy badge in snake_case
    GlobalMod,

    /// Subscriber badge with tier info
    /// This is being parsed if the data number matches the format [num]0[num]
    #[display("subscriber/{0}0{1:>02}")]
    TierSubscriber(u8, #[from_str(regex = "[0-9]{2,}")] u32), //u8: Subscription tier, u32: Subscription months (at least two characters)

    /// Subscriber badge without tier info.
    /// This is being parsed if the data number didn't match the pattern in TierSubscriber.
    #[display("subscriber/{0}")]
    NoTierSubscriber(u32), //u32: Subscription months

    /// Unknown badge. Likely a custom badge
    #[display("{0}/{1}")] // displays only the inside data
    Unknown(String, u64),
}

impl Badge{
    // all other is_variant() functions are derived automatically
    /// Returns whether this badge is any kind of subscriber badge.
    pub(crate) fn is_subscriber(&self) -> bool{
        self.is_tier_subscriber() || self.is_no_tier_subscriber()
    }
}
/// Metadata to the chat badges
pub type BadgeInfo = Badge;


/// We implement Attribution, but define a custom parse function.
/// This is a roundabout way of still being able to use AttributionVec<Badge>.
impl Attribution<Badge, u64> for Badge {
    fn new(reference: Badge, _attributes: impl Iterator<Item=u64>) -> Self {
        reference
    }

    fn get_separator_info() -> SeparatorInfo {
        SeparatorInfo {
            attribution_separator: ',',
            range_attribute_separator: '\0', // does not matter
            attribute_separator: '\0', // does not matter
        }
    }

    fn parse(item: &str) -> Option<Self> {
        <Badge as FromStr>::from_str(item).ok()
    }
}

/// Vector containing user badges
pub type BadgeVec = AttributionVec<Badge, u64, Badge>;

/* 
 *//* 
/// An iterator over badges
#[derive(Debug, Constructor)]
pub struct BadgesIter<'a> {
    items: Option<std::str::Split<'a, char>>,
}

impl<'a> Iterator for BadgesIter<'a> {
    type Item = Badge<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.items.as_mut()?.next() {
            Badge::parse(item)
        } else {
            None
        }
    }
}

#[allow(dead_code)]
pub(crate) fn parse_badges_iter(input: &str) -> impl Iterator<Item = Badge<'_>> + '_ {
    input.split(',').filter_map(Badge::parse)
}
 */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_known_badges() {
        // ("input", expected value)
        let badge_set: &[(&str, Badge)] = &[
            ("admin/1", Badge::Admin),
            ("bits/100", Badge::Bits(100)),
            ("broadcaster/1", Badge::Broadcaster),
            ("global_mod/1", Badge::GlobalMod),
            ("moderator/1", Badge::Moderator),
            ("subscriber/1", Badge::NoTierSubscriber(1)),
            ("subscriber/103", Badge::NoTierSubscriber(103)),
            ("subscriber/3001", Badge::TierSubscriber(3,1)),
            ("staff/1", Badge::Staff),
            ("turbo/1", Badge::Turbo),
            ("premium/1", Badge::Premium),
            ("vip/1", Badge::Vip),
            ("partner/1", Badge::Partner),
            ("unknown/1", Badge::Unknown("unknown".into(), 1)),
        ];

        for (raw, badge) in badge_set {
            println!("{}", raw);
            let parsed_badge = Badge::from_str(raw).expect("Malformed badge test");
            assert_eq!(*badge, parsed_badge);
        }
    }

    #[test]
    fn parse_invalid() {
        let badge_str = "this_badge_is_invalid";
        let badge_result = Badge::from_str(badge_str);
        assert!(badge_result.is_err())
    }
}
