use crate::irc::tags::ParsedTag;
use crate::twitch::{Attribution, AttributionVec, BadgeVec, Color, EmoteVec, FlagVec};
use crate::{irc::*, MaybeOwned, MaybeOwnedIndex, Validator};
use parse_display::FromStr;
use std::str::FromStr;

/// A paid subscription ot the channel
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Hash, FromStr)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub enum SubPlan {
    /// A `Prime` subscription
    Prime,

    /// A Tier-1 subscription (currently $4.99)
    #[from_str(regex = "1000")]
    Tier1,

    /// A Tier-2 subscription (currently $9.99)
    #[from_str(regex = "2000")]
    Tier2,

    /// A Tier-3 subscription (currently $24.99)
    #[from_str(regex = "3000")]
    Tier3,

    /// An unknown tier -- this will catch and future tiers if they are added.
    #[display("{0}")]
    Unknown(String),
}

/// The kind of notice it was, retrieved via [UserNotice::msg_id()]
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Hash, FromStr)]
#[display(style = "lowercase")]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub enum NoticeType {
    /// This was a subscription notice
    Sub,
    /// This was a re-subscription notice
    Resub,
    /// This was a gifted subscription
    SubGift,
    /// This was an anonymous gifted subscription
    AnonSubGift,
    /// This was a mystery gift for the subscription
    SubMysteryGift,
    /// Gift for a paid upgrade
    GiftPaidUpgrade,
    /// A reward gift
    RewardGift,
    /// An anonymous gift for paid upgrade
    AnonGiftPaidUpgrade,
    /// A raid
    Raid,
    /// A canceled raid
    Unraid,
    /// A ritual
    Ritual,
    /// A the tier that the bits were part of
    BitsBadgeTier,
    /// An unknown notice type (a catch-all)
    #[display("{0}")]
    Unknown(String),
}

/// Announces Twitch-specific events to the channel (e.g., a user's subscription notification).
#[derive(Clone, PartialEq)]
pub struct UserNotice<'a> {
    raw: MaybeOwned<'a>,
    tags: TagIndices,
    channel: MaybeOwnedIndex,
    message: Option<MaybeOwnedIndex>,
}

impl<'a> UserNotice<'a> {
    raw!();
    tags!();
    str_field!(
        /// The channel that this event is happening on
        channel
    );
    opt_str_field!(
        /// Optional message attached to the event
        message
    );

    /// Helper function to return information that can be parsed as AttributionVec. (copied from Privmsg)
    fn tag_to_attribution_vec<Ref, Attr, T>(
        &'a self,
        tag: impl AsRef<str>,
    ) -> AttributionVec<Ref, Attr, T>
    where
        Ref: FromStr,
        Attr: FromStr,
        T: Attribution<Ref, Attr>,
    {
        self.tags()
            .get(tag.as_ref())
            .map(AttributionVec::<Ref, Attr, T>::from_str)
            .map(Result::ok)
            .flatten()
            .unwrap_or_else(|| vec![].into())
    }

    /// Metadata related to the chat badges
    ///
    /// Currently used only for `subscriber`, to indicate the exact number of months the user has been a subscriber
    pub fn badge_info(&'a self) -> BadgeVec {
        self.tag_to_attribution_vec("badge-info")
    }

    /// Badges attached to this message
    pub fn badges(&'a self) -> BadgeVec {
        self.tag_to_attribution_vec("badges")
    }

    /// The user's color, if set
    pub fn color(&self) -> Option<ParsedTag<Color>> {
        self.tags().get_parsed("color")
    }

    /// The user's display name, if set
    pub fn display_name(&self) -> Option<&str> {
        self.tags().get("display-name")
    }

    /// Emotes attached to this message
    pub fn emotes(&self) -> EmoteVec {
        self.tag_to_attribution_vec("emotes")
    }

    /// Flags attached to this message
    pub fn flags(&self) -> FlagVec {
        self.tag_to_attribution_vec("flags")
    }

    /// A unique id (UUID) attached to this message
    ///
    /// (this is used for message localization)
    pub fn id(&self) -> Option<&str> {
        self.tags().get("id")
    }

    /// The name of the user who sent this notice
    pub fn login(&self) -> Option<&str> {
        self.tags().get("login")
    }

    /// Whether this user is a moderator
    pub fn is_moderator(&self) -> bool {
        self.tags().get_as_bool("mod")
    }

    /// The kind of notice this message is
    pub fn msg_id(&'a self) -> Option<ParsedTag<NoticeType>> {
        self.tags().get_parsed("msg-id")
    }

    /// The id of the room for this notice
    pub fn room_id(&self) -> Option<ParsedTag<u64>> {
        self.tags().get_parsed("room-id")
    }

    /// The timestamp which twitch received this message
    pub fn tmi_sent_ts(&self) -> Option<ParsedTag<u64>> {
        self.tags().get_parsed("tmi-sent-ts")
    }

    /// User id of the user who sent this notice
    pub fn user_id(&self) -> Option<ParsedTag<u64>> {
        self.tags().get_parsed("user-id")
    }

    /// The message printed in chat along with this notice
    pub fn system_msg(&self) -> Option<String> {
        self.tags()
            .get("system-msg")?
            .replace("\\s", " ")
            .replace("\\r", "\r")
            .replace("\\n", "\n")
            .replace("\\\\", "\\")
            .replace("\\:", ":")
            .into()
    }

    /// (Sent only on sub, resub) The total number of months the user has
    /// subscribed.
    ///
    /// This is the same as msg-param-months but sent for different
    /// types of user notices.
    pub fn msg_param_cumulative_months(&self) -> Option<ParsedTag<u64>> {
        self.tags().get_parsed("msg-param-cumulative-months")
    }

    /// (Sent only on raid) The display name of the source user raiding this
    /// channel.
    pub fn msg_param_display_name(&self) -> Option<&str> {
        self.tags().get("msg-param-displayName")
    }

    /// (Sent on only raid) The name of the source user raiding this channel.

    pub fn msg_param_login(&self) -> Option<&str> {
        self.tags().get("msg-param-login")
    }

    /// (Sent only on subgift, anonsubgift) The total number of months the user
    /// has subscribed.
    ///
    /// This is the same as msg-param-cumulative-months but sent
    /// for different types of user notices.
    pub fn msg_param_months(&self) -> Option<ParsedTag<u64>> {
        self.tags().get_parsed("msg-param-months")
    }

    /// (Sent only on anongiftpaidupgrade, giftpaidupgrade) The number of gifts
    /// the gifter has given during the promo indicated by msg-param-promo-name.
    pub fn msg_param_promo_gift_total(&self) -> Option<ParsedTag<u64>> {
        self.tags().get_parsed("msg-param-promo-gift-total")
    }

    /// (Sent only on anongiftpaidupgrade, giftpaidupgrade) The subscriptions
    /// promo, if any, that is ongoing; e.g. Subtember 2018.
    pub fn msg_param_promo_name(&self) -> Option<&str> {
        self.tags().get("msg-param-promo-name")
    }

    /// (Sent only on subgift, anonsubgift) The display name of the subscription
    /// gift recipient.
    pub fn msg_param_recipient_display_name(&self) -> Option<&str> {
        self.tags().get("msg-param-recipient-display-name")
    }

    /// (Sent only on subgift, anonsubgift) The user ID of the subscription gift
    /// recipient.
    pub fn msg_param_recipient_id(&self) -> Option<ParsedTag<u64>> {
        self.tags().get_parsed("msg-param-recipient-id")
    }

    /// (Sent only on subgift, anonsubgift) The user name of the subscription
    /// gift recipient.
    pub fn msg_param_recipient_user_name(&self) -> Option<&str> {
        self.tags().get("msg-param-recipient-user-name")
    }

    /// (Sent only on giftpaidupgrade) The login of the user who gifted the
    /// subscription.
    pub fn msg_param_sender_login(&self) -> Option<&str> {
        self.tags().get("msg-param-sender-login")
    }

    /// (Sent only on giftpaidupgrade) The display name of the user who gifted
    /// the subscription.
    pub fn msg_param_sender_name(&self) -> Option<&str> {
        self.tags().get("msg-param-sender-name")
    }

    /// (Sent only on sub, resub) Boolean indicating whether users want their
    /// streaks to be shared.
    pub fn msg_param_should_share_streak(&self) -> Option<ParsedTag<bool>> {
        self.tags().get_parsed("msg-param-should-share-streak")
    }

    /// (Sent only on sub, resub) The number of consecutive months the user has
    /// subscribed.
    ///
    /// This is 0 if msg-param-should-share-streak is 0.
    pub fn msg_param_streak_months(&self) -> Option<ParsedTag<u64>> {
        self.tags().get_parsed("msg-param-streak-months")
    }

    /// (Sent only on sub, resub, subgift, anonsubgift) The type of subscription
    /// plan being used.
    ///
    /// Valid values: Prime, 1000, 2000, 3000. 1000, 2000, and
    /// 3000 refer to the first, second, and third levels of paid subscriptions,
    /// respectively (currently $4.99, $9.99, and $24.99).
    pub fn msg_param_sub_plan(&'a self) -> Option<SubPlan> {
        self.tags().get("msg-param-sub-plan").and_then(|s| {
            match s {
                "Prime" => SubPlan::Prime,
                "Tier1" => SubPlan::Tier1,
                "Tier2" => SubPlan::Tier2,
                "Tier3" => SubPlan::Tier3,
                s => SubPlan::Unknown(s.into()),
            }
            .into()
        })
    }

    /// (Sent only on sub, resub, subgift, anonsubgift) The display name of the
    /// subscription plan.
    ///
    /// This may be a default name or one created by the
    /// channel owner.
    pub fn msg_param_sub_plan_name(&self) -> Option<&str> {
        self.tags().get("msg-param-sub-plan-name")
    }

    /// (Sent only on raid) The number of viewers watching the source channel
    /// raiding this channel.
    pub fn msg_param_viewer_count(&self) -> Option<ParsedTag<u64>> {
        self.tags().get_parsed("msg-param-viewerCount")
    }

    /// (Sent only on ritual) The name of the ritual this notice is for. Valid
    /// value: new_chatter.
    pub fn msg_param_ritual_name(&self) -> Option<&str> {
        self.tags().get("msg-param-ritual-name")
    }

    /// (Sent only on bitsbadgetier) The tier of the bits badge the user just
    /// earned; e.g. 100, 1000, 10000.
    pub fn msg_param_threshold(&self) -> Option<ParsedTag<u64>> {
        self.tags().get_parsed("msg-param-threshold")
    }
}

impl<'a> FromIrcMessage<'a> for UserNotice<'a> {
    type Error = MessageError;

    fn from_irc(msg: IrcMessage<'a>) -> Result<Self, Self::Error> {
        msg.expect_command(IrcMessage::USER_NOTICE)?;

        let this = Self {
            channel: msg.expect_arg_index(0)?,
            message: msg.data,
            tags: msg.parse_tags(),
            raw: msg.raw,
        };

        Ok(this)
    }

    into_inner_raw!();
}

into_owned!(UserNotice {
    raw,
    tags,
    channel,
    message,
});

impl_custom_debug!(UserNotice {
    raw,
    tags,
    channel,
    message,
});

serde_struct!(UserNotice {
    raw,
    tags,
    channel,
    message,
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::twitch::Badge;
    use assert2::assert;

    #[test]
    #[cfg(feature = "serde")]
    fn user_notice_serde() {
        let input = &[
            ":tmi.twitch.tv USERNOTICE #museun :This room is no longer in slow mode.\r\n",
            ":tmi.twitch.tv USERNOTICE #museun\r\n",
            "@badge-info=subscriber/8;badges=subscriber/6,bits/100;color=#59517B;display-name=lllAirJordanlll;emotes=;flags=;id=3198b02c-eaf4-4904-9b07-eb1b2b12ba50;login=lllairjordanlll;mod=0;msg-id=resub;msg-param-cumulative-months=8;msg-param-months=0;msg-param-should-share-streak=0;msg-param-sub-plan-name=Channel\\sSubscription\\s(giantwaffle);msg-param-sub-plan=1000;room-id=22552479;subscriber=1;system-msg=lllAirJordanlll\\ssubscribed\\sat\\sTier\\s1.\\sThey\'ve\\ssubscribed\\sfor\\s8\\smonths!;tmi-sent-ts=1580932171144;user-id=44979519;user-type= :tmi.twitch.tv USERNOTICE #giantwaffle\r\n",
        ];

        for input in input {
            crate::serde::round_trip_json::<UserNotice>(input);
            crate::serde::round_trip_rmp::<UserNotice>(input);
        }
    }

    #[test]
    fn user_notice_message() {
        let input = ":tmi.twitch.tv USERNOTICE #museun :This room is no longer in slow mode.\r\n";

        for msg in parse(input).map(|s| s.unwrap()) {
            let msg = UserNotice::from_irc(msg).unwrap();
            assert_eq!(msg.channel(), "#museun");
            assert_eq!(
                msg.message().unwrap(),
                "This room is no longer in slow mode."
            );
        }
    }

    #[test]
    fn user_notice_stability() {
        let input = ":tmi.twitch.tv USERNOTICE #museun\r\n";
        for msg in parse(input).map(|s| s.unwrap()) {
            let msg = UserNotice::from_irc(msg).unwrap();
        }
    }

    #[test]
    fn user_notice_integrity() {
        let input = "@badge-info=;badges=staff/1,broadcaster/1,turbo/1;color=#008000;display-name=ronni;emotes=;id=db25007f-7a18-43eb-9379-80131e44d633;login=ronni;mod=0;msg-id=resub;msg-param-cumulative-months=6;msg-param-streak-months=2;msg-param-should-share-streak=1;msg-param-sub-plan=Prime;msg-param-sub-plan-name=Prime;room-id=1337;subscriber=1;system-msg=ronni\\shas\\ssubscribed\\sfor\\s6\\smonths!;tmi-sent-ts=1507246572675;turbo=1;user-id=1337;user-type=staff :tmi.twitch.tv USERNOTICE #dallas :Great stream -- keep it up!\r\n";

        for msg in parse(input).map(|s| s.unwrap()) {
            let msg = UserNotice::from_irc(msg).unwrap();

            assert!(msg.channel() == "#ronni");
            assert!(msg.message().unwrap() == "Great stream -- keep it up!");

            assert!(msg.badge_info().unwrap().unwrap() == vec![]);
            assert!(msg.badges().unwrap().unwrap() == vec![Badge::Staff, Badge::Broadcaster, Badge::Turbo]);
            assert!(msg.color().unwrap().unwrap() == "#008000".parse().unwrap());
            assert!(msg.display_name().unwrap() == "ronni");
            assert!(msg.emotes().unwrap().unwrap() == vec![]);
            assert!(msg.id().unwrap() == "db25007f-7a18-43eb-9379-80131e44d633");
            assert!(msg.r#mod().unwrap().unwrap() == false);
            assert!(msg.room_id().unwrap().unwrap() == 1337);
            assert!(msg.subscriber().unwrap().unwrap() == true);
            assert!(msg.tmi_sent_ts().unwrap().unwrap() == 1507246572675);
            assert!(msg.turbo().unwrap().unwrap() == true);
            assert!(msg.user_id().unwrap().unwrap() == 1337);
            assert!(msg.user_type().unwrap() == "staff");
            assert!(msg.login().unwrap() == "ronni");
            assert!(msg.msg_id().unwrap().unwrap() == NoticeType::Resub);
            assert!(msg.msg_param_cumulative_months().unwrap().unwrap() == 6);
            assert!(msg.msg_param_streak_months().unwrap().unwrap() == 2);
            assert!(msg.msg_param_should_share_streak().unwrap().unwrap() == true);
            assert!(msg.msg_param_sub_plan().unwrap().unwrap() == SubPlan::Prime);
            assert!(msg.msg_param_sub_plan_name().unwrap() == "Prime");
            assert!(msg.system_msg().unwrap() == "ronni has subscribed for 6 months!");
        }
    }
    #[test]
    fn user_notice_unknown() {
        let input = "@badge-info=subscriber/8;badges=subscriber/6,bits/100;color=#59517B;display-name=lllAirJordanlll;emotes=;flags=;id=3198b02c-eaf4-4904-9b07-eb1b2b12ba50;login=lllairjordanlll;mod=0;msg-id=resub;msg-param-cumulative-months=8;msg-param-months=0;msg-param-should-share-streak=0;msg-param-sub-plan-name=Channel\\sSubscription\\s(giantwaffle);msg-param-sub-plan=1000;room-id=22552479;subscriber=1;system-msg=lllAirJordanlll\\ssubscribed\\sat\\sTier\\s1.\\sThey\'ve\\ssubscribed\\sfor\\s8\\smonths!;tmi-sent-ts=1580932171144;user-id=44979519;user-type= :tmi.twitch.tv USERNOTICE #giantwaffle\r\n";
        for msg in parse(input).map(|s| s.unwrap()) {
            let msg = UserNotice::from_irc(msg).unwrap();
            assert_eq!(msg.channel(), "#giantwaffle");
            assert_eq!(msg.tags().is_empty(), false);
        }
    }
}
