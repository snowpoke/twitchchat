/* // TODO: Change tag parsing formats
tmi-sent-ts: from u64 to Timestamp (wraps NaiveDateTime)
user-id: u64 to &str
msg-id: &str to NoticeType? (or custom enum), or UUID, msg-id seems to refer to all sorts of things
id: &str to UUID (uuid crate)
msg-param-recipient-id: u64 to &str
msg-param-streak-months: u64 to BoundedU16
msg-param-viewerCount: u64 to u32
ban-duration: u64 to enum BanDuration{ Timeout(BoundedU32<1,1_209_600>) or Permanent}
target-msg-id: &str to UUID
emote-sets: Vec<&str> to Vec<u32>
room-id: u64 to &str */
use crate::messages::SubPlan;
use crate::twitch::{BadgeVec, Color, EmoteVec, FlagVec, EmoteSet};
use crate::messages::{NoticeType, FollowersOnly};
use twitchchat_macros::generate_tag_traits as init_tags;

/// Trait that should be applied to all message struct that can contain tags.
pub trait HasTags<'a> {
    fn tags(&'a self) -> crate::irc::Tags<'a>;
}

init_tags![
    "badge-info" as BadgeVec,
    "badges" as BadgeVec,
    "ban-duration" as u64,
    "bits" as u64,
    "color" as Color,
    "display-name",
    "emote-only" as bool,
    "emote-sets" as EmoteSet,
    "emotes" as EmoteVec,
    "flags" as FlagVec,
    "followers-only" as FollowersOnly,
    "id",
    "login",
    "mod" as bool,
    "msg-id" as NoticeType,
    "r9k" as bool,
    "room-id" as u64,
    "slow" as u64,
    "subs-only" as bool,
    "subscriber",
    "system-msg" as String,
    "target-msg-id",
    "tmi-sent-ts" as u64,
    "turbo",
    "user-id" as u64,
    "user-type",
    "msg-param-cumulative-months" as u64,
    "msg-param-displayName",
    "msg-param-login",
    "msg-param-months" as u64,
    "msg-param-promo-gift-total" as u64,
    "msg-param-promo-name",
    "msg-param-recipient-display-name",
    "msg-param-recipient-id" as u64,
    "msg-param-recipient-user-name",
    "msg-param-sender-login",
    "msg-param-sender-name",
    "msg-param-should-share-streak" as bool,
    "msg-param-streak-months" as u64,
    "msg-param-sub-plan" as SubPlan,
    "msg-param-sub-plan-name",
    "msg-param-viewerCount" as u64,
    "msg-param-ritual-name",
    "msg-param-threshold" as u64,
    "msg-param-gift-months" as u64,
];
