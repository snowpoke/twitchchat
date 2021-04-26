use crate::irc::tags::ParsedTag;
use crate::twitch::{Attribution, AttributionVec, Badge, BadgeVec, Color, EmoteVec, FlagVec};
use crate::{irc::*, MaybeOwned, MaybeOwnedIndex, Validator};
use std::str::FromStr;

/// Message sent by another user to your user (a 'DM')
#[derive(Clone, PartialEq)]
pub struct Whisper<'a> {
    raw: MaybeOwned<'a>,
    tags: TagIndices,
    name: MaybeOwnedIndex,
    data: MaybeOwnedIndex,
}

impl<'a> Whisper<'a> {
    raw!();
    tags!();
    str_field!(
        /// User who sent this messages
        name
    );
    str_field!(
        /// Data that the user provided
        data
    );

    /// The color of the user who sent this message, if set
    pub fn color(&self) -> Option<ParsedTag<Color>> {
        self.tags().get_parsed("color")
    }

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

    /// Returns the display name of the user, if set.
    ///
    /// Users can changed the casing and encoding of their names, if they choose
    /// to.
    ///
    /// By default, their display name is not set. If the user **foo** changes
    /// their display name to **FOO** then this'll return that **FOO**.
    ///
    /// Otherwise it'll return `None`.
    pub fn display_name(&'a self) -> Option<&'a str> {
        self.tags().get("display-name")
    }

    /// Badges attached to this message
    pub fn badge_info(&'a self) -> BadgeVec {
        self.tag_to_attribution_vec("badge-info")
    }

    /// Badges attached to this message
    pub fn badges(&'a self) -> BadgeVec {
        self.tag_to_attribution_vec("badges")
    }

    /// Emotes attached to this message
    pub fn emotes(&self) -> EmoteVec {
        self.tag_to_attribution_vec("emotes")
    }

    /// Flags attached to this message
    pub fn flags(&self) -> FlagVec {
        self.tag_to_attribution_vec("flags")
    }

    /// Whether the user sending this message was a staff member
    pub fn is_staff(&self) -> bool {
        self.any_badge(Badge::is_staff)
    }

    /// Whether the user sending this message had turbo
    pub fn is_turbo(&self) -> bool {
        self.any_badge(Badge::is_turbo)
    }

    /// Whether the user sending this message was a global moderator
    pub fn is_global_moderator(&self) -> bool {
        self.any_badge(Badge::is_global_mod)
    }

    /// Helper function that checks if any badge fulfills a specific requirement. Intended to be used with Badge::is_variant functions.
    fn any_badge(&self, is_badge_fn: impl Fn(&Badge) -> bool) -> bool {
        self.badges().iter().any(is_badge_fn)
    }

    /// The timestamp of when this message was received by Twitch
    pub fn tmi_sent_ts(&self) -> Option<ParsedTag<u64>> {
        self.tags().get_parsed("tmi-sent-ts")
    }

    /// The id of the user who sent this message
    pub fn user_id(&self) -> Option<ParsedTag<u64>> {
        self.tags().get_parsed("user-id")
    }
}

impl<'a> FromIrcMessage<'a> for Whisper<'a> {
    type Error = MessageError;

    fn from_irc(msg: IrcMessage<'a>) -> Result<Self, Self::Error> {
        msg.expect_command(IrcMessage::WHISPER)?;

        // :sender WHISPER target :data
        // we're the target, so ignore it

        let this = Self {
            name: msg.expect_nick()?,
            data: msg.expect_data_index()?,
            tags: msg.parse_tags(),
            raw: msg.raw,
        };

        Ok(this)
    }

    into_inner_raw!();
}

into_owned!(Whisper {
    raw,
    tags,
    name,
    data,
});
impl_custom_debug!(Whisper {
    raw,
    tags,
    name,
    data,
});
serde_struct!(Whisper {
    raw,
    tags,
    name,
    data,
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "serde")]
    fn whisper_serde() {
        let input = ":test!user@host WHISPER museun :this is a test\r\n";
        crate::serde::round_trip_json::<Whisper>(input);
        crate::serde::round_trip_rmp::<Whisper>(input);
    }

    #[test]
    fn whisper() {
        let input = ":test!user@host WHISPER museun :this is a test\r\n";
        for msg in parse(input).map(|s| s.unwrap()) {
            let msg = Whisper::from_irc(msg).unwrap();

            assert_eq!(msg.name(), "test");
            assert_eq!(msg.data(), "this is a test");
        }
    }
}
