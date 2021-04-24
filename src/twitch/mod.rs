//! Common Twitch types

mod capability;
pub use capability::Capability;

mod userconfig;
pub use userconfig::{UserConfig, UserConfigBuilder, UserConfigError};

pub mod attributes;
pub use attributes::{MsgRange, AttributionVec};

mod emotes;
pub use emotes::Emote;

mod flags;
pub use flags::Flag;

mod badge;
pub use badge::{Badge, BadgeInfo, BadgeKind, BadgesIter};

pub mod color;
#[doc(inline)]
pub use color::Color;



#[allow(dead_code)]
pub(crate) fn parse_badges(input: &str) -> Vec<Badge<'_>> {
    input.split(',').filter_map(Badge::parse).collect()
}



#[allow(dead_code)]
pub(crate) fn parse_badges_iter(input: &str) -> impl Iterator<Item = Badge<'_>> + '_ {
    input.split(',').filter_map(Badge::parse)
}
