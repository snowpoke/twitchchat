/* // TODO: Change tag parsing formats
tmi-sent-ts: from u64 to Timestamp (wraps NaiveDateTime)
user-id: u64 to &str
msg-id: &str to NoticeType? (or custom enum), or UUID, msg-id seems to refer to all sorts of things
id: &str to UUID (uuid crate)
msg-param-recipient-id: u64 to &str
msg-param-streak-months: u64 to BoundedU16
msg-param-viewerCount: u64 to u32
ban-duration: u64 to BoundedU32<1,1_209_600>
target-msg-id: &str to UUID
emote-sets: Vec<&str> to Vec<u32>
room-id: u64 to &str */
use crate::messages::SubPlan;
use twitchchat_macros::generate_tag_parser_getter as include_tags;

trait HasTags<'a> {
    fn tags(&self) -> crate::irc::Tags<'a>;
}

trait HasMsgParam<'a>: HasTags<'a> {
    include_tags![
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
}
