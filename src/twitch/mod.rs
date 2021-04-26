//! Common Twitch types

mod capability;
pub use capability::Capability;

mod userconfig;
pub use userconfig::{UserConfig, UserConfigBuilder, UserConfigError};

pub mod attributes;
pub use attributes::MsgRange;
pub(crate) use attributes::{Attribution, AttributionVec};

mod emotes;
pub use emotes::{Emote, EmoteVec};

mod flags;
pub use flags::{Flag, FlagVec};

mod badge;
pub use badge::{Badge, BadgeInfo, BadgeVec};

pub mod color;
#[doc(inline)]
pub use color::Color;