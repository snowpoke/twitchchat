use crate::irc::tags::ParsedTag;
use crate::twitch::{Attribution, AttributionVec, BadgeVec, Color, EmoteVec, FlagVec};
use crate::{irc::*, MaybeOwned, MaybeOwnedIndex, Validator};
use std::str::FromStr;

/// Identifies a user's chat settings or properties (e.g., chat color)..
#[derive(Clone, PartialEq)]
pub struct UserState<'a> {
    raw: MaybeOwned<'a>,
    tags: TagIndices,
    channel: MaybeOwnedIndex,
}

impl<'a> UserState<'a> {
    raw!();
    tags!();
    str_field!(
        /// Channel this event happened on
        channel
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
    /// Currently used only for `subscriber`, to indicate the exact number of
    /// months the user has been a subscriber
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

    /// Whether this user is a moderator
    pub fn is_moderator(&self) -> bool {
        self.tags().get_as_bool("mod")
    }
}

impl<'a> FromIrcMessage<'a> for UserState<'a> {
    type Error = MessageError;

    fn from_irc(msg: IrcMessage<'a>) -> Result<Self, Self::Error> {
        msg.expect_command(IrcMessage::USER_STATE)?;

        let this = Self {
            tags: msg.parse_tags(),
            channel: msg.expect_arg_index(0)?,
            raw: msg.raw,
        };

        Ok(this)
    }

    into_inner_raw!();
}

into_owned!(UserState { raw, tags, channel });
impl_custom_debug!(UserState { raw, tags, channel });
serde_struct!(UserState { raw, tags, channel });

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "serde")]
    fn user_state_serde() {
        let input = "@badges=bits/1000;badge-info=moderator :tmi.twitch.tv USERSTATE #museun\r\n";
        crate::serde::round_trip_json::<UserState>(input);
        crate::serde::round_trip_rmp::<UserState>(input);
    }

    #[test]
    fn user_state() {
        let input = ":tmi.twitch.tv USERSTATE #museun\r\n";
        for msg in parse(input).map(|s| s.unwrap()) {
            let msg = UserState::from_irc(msg).unwrap();
            assert_eq!(msg.channel(), "#museun");
        }
    }
}
