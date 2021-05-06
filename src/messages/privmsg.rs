use crate::irc::tags::ParsedTag;
use crate::twitch::attributes::{Attribution, AttributionVec};
use crate::twitch::{Badge, BadgeVec, Color, EmoteVec, FlagVec};
use crate::{irc::*, MaybeOwned, MaybeOwnedIndex, Validator};
use std::str::FromStr;

// IDEA: Use tendril crate for parsing

/// Some PRIVMSGs are considered 'CTCP' (client-to-client protocol)
///
/// This is a tag-type for determining what kind of CTCP it was
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub enum Ctcp<'a> {
    /// An action CTCP, sent by the user when they do `/me` or `/action`
    Action,
    /// An unknown CTCP
    Unknown {
        /// The unknown CTCP command
        command: &'a str,
    },
}

/// Message sent by a user
#[derive(Clone, PartialEq)]
pub struct Privmsg<'a> {
    raw: MaybeOwned<'a>,
    tags: TagIndices,
    name: MaybeOwnedIndex,
    channel: MaybeOwnedIndex,
    data: MaybeOwnedIndex,
    ctcp: Option<MaybeOwnedIndex>,
}

impl<'a> Privmsg<'a> {
    raw!();
    tags!();
    str_field!(
        /// User who sent this messages
        name
    );
    str_field!(
        /// Channel this message was sent on
        channel
    );
    str_field!(
        /// Data that the user provided
        data
    );

    /// Iterator alternative to `Privmsg::badges()`
    // pub fn iter_badges(&self) -> BadgesIter {
    //     BadgesIter::new(
    //         self.tags().get("badges").map(|s| s.split(',')),
    //     )
    // }

    /// Iterator alternative to `Privmsg::emotes()`
    // pub fn iter_emotes(&self) -> EmotesIter {
    //     EmotesIter::new(
    //         self.tags().get("emotes").map(|s| s.split_terminator('/'))
    //     )
    // }

    /// Gets the 'CTCP' kind associated with this message, if any
    pub fn ctcp(&self) -> Option<Ctcp<'_>> {
        const ACTION: &str = "ACTION";
        let command = &self.raw[self.ctcp?];
        if command == ACTION {
            Some(Ctcp::Action)
        } else {
            Some(Ctcp::Unknown { command })
        }
    }

    /// Whether this message was an Action (a `/me` or `/action`)
    pub fn is_action(&self) -> bool {
        matches!(self.ctcp(), Some(Ctcp::Action))
    }

    /// Helper function to return information that can be parsed as AttributionVec.
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

    /// How many bits were attached to this message
    pub fn bits(&self) -> Option<ParsedTag<u64>> {
        self.tags().get_parsed("bits")
    }

    /// The color of the user who sent this message, if set
    pub fn color(&self) -> Option<ParsedTag<Color>> {
        self.tags().get_parsed("color")
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
    pub fn display_name(&'a self) -> Option<&str> {
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

    /// Whether the user sending this message was a broadcaster
    pub fn is_broadcaster(&self) -> bool {
        self.any_badge(Badge::is_broadcaster)
    }

    /// Whether the user sending this message was a moderator
    pub fn is_moderator(&self) -> bool {
        self.any_badge(Badge::is_moderator)
    }

    /// Whether the user sending this message was a vip
    pub fn is_vip(&self) -> bool {
        self.any_badge(Badge::is_vip)
    }

    /// Whether the user sending this message was a susbcriber
    pub fn is_subscriber(&self) -> bool {
        self.any_badge(Badge::is_subscriber)
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

    /// The id of the room this message was sent to
    pub fn room_id(&self) -> Option<ParsedTag<u64>> {
        self.tags().get_parsed("room-id")
    }

    /// The timestamp of when this message was received by Twitch
    pub fn tmi_sent_ts(&self) -> Option<ParsedTag<u64>> {
        self.tags().get_parsed("tmi-sent-ts")
    }

    /// The id of the user who sent this message
    pub fn user_id(&self) -> Option<ParsedTag<u64>> {
        self.tags().get_parsed("user-id")
    }

    /// `custom-reward-id` is returned on custom rewards set by broadcaster.
    ///
    /// **NOTE** From the new community points rewards.
    ///
    /// With no api from Twitch to retrieve proper name, looks like a UUID.
    pub fn custom_reward_id(&self) -> Option<&str> {
        self.tags().get("custom-reward-id")
    }

    /// Specifies messages with a type of highlighting. Like (re)sub messages, activating host mode, or messages highlighted with channel points.
    pub fn msg_id(&self) -> Option<&str> {
        self.tags().get("msg-id")
    }
}

impl<'a> FromIrcMessage<'a> for Privmsg<'a> {
    type Error = MessageError;

    fn from_irc(msg: IrcMessage<'a>) -> Result<Self, Self::Error> {
        const CTCP_MARKER: char = '\x01';
        msg.expect_command(IrcMessage::PRIVMSG)?;

        let mut index = msg.expect_data_index()?;
        let mut ctcp = None;

        let data = &msg.raw[index];
        if data.starts_with(CTCP_MARKER) && data.ends_with(CTCP_MARKER) {
            let len = data.chars().map(char::len_utf8).sum::<usize>();
            match data[1..len - 1].find(' ') {
                Some(pos) => {
                    // TODO refactor this so the casting is done in 1 canonical place
                    //
                    // skip the first byte
                    let head = index.start + 1;
                    let ctcp_index = MaybeOwnedIndex::raw(head as usize, (head as usize) + pos);

                    // for the byte + space
                    index.start += (pos as u16) + 2;
                    index.end -= 1;
                    ctcp.replace(ctcp_index);
                }
                None => return Err(MessageError::ExpectedData),
            }
        }

        let this = Self {
            tags: msg.parse_tags(),
            name: msg.expect_nick()?,
            channel: msg.expect_arg_index(0)?,
            data: index,
            ctcp,
            raw: msg.raw,
        };
        Ok(this)
    }

    into_inner_raw!();
}

into_owned!(Privmsg {
    raw,
    tags,
    name,
    channel,
    data,
    ctcp,
});

impl_custom_debug!(Privmsg {
    raw,
    tags,
    name,
    channel,
    data,
    ctcp,
    // TODO decide /how/ much should be in the debug, all of this is in the tags
    // is_action,
    // badge_info,
    // badges,
    // bits,
    // color,
    // display_name,
    // emotes,
    // is_broadcaster,
    // is_moderator,
    // is_vip,
    // is_subscriber,
    // is_staff,
    // is_turbo,
    // is_global_moderator,
    // room_id,
    // tmi_sent_ts,
    // user_id,
    // custom_reward_id,
    // msg_id,
});

serde_struct!(Privmsg {
    raw,
    tags,
    name,
    channel,
    data,
});

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! emote {
        ($id:expr, $($r:expr),* $(,)?) => {
            Emote {
                id: $id,
                ranges: vec![$($r.into()),*]
            }
        };
    }

    #[test]
    #[cfg(feature = "serde")]
    fn privmsg_serde() {
        let input = &[
            ":test!user@host PRIVMSG #museun :this is a test\r\n",
            ":test!user@host PRIVMSG #museun :\u{FFFD}\u{1F468}\r\n",
            ":test!user@host PRIVMSG #museun :\x01ACTION this is a test\x01\r\n",
            ":test!user@host PRIVMSG #museun :\x01FOOBAR this is a test\x01\r\n",
        ];

        for input in input {
            crate::serde::round_trip_json::<Privmsg>(input);
            crate::serde::round_trip_rmp::<Privmsg>(input);
        }
    }

    #[test]
    fn privmsg_stability() {
        let input = ":test!user@host PRIVMSG #museun :this is a test\r\n";
        for msg in parse(input).map(|s| s.unwrap()) {
            let msg = Privmsg::from_irc(msg).unwrap();
        }
    }

    #[test]
    fn privmsg_integrity() {
        let input = "@badge-info=;badges=global_mod/1,turbo/1;color=#0D4200;display-name=ronni;emotes=25:0-4,12-16/1902:6-10;id=b34ccfc7-4977-403a-8a94-33c6bac34fb8;mod=0;room-id=1337;subscriber=0;tmi-sent-ts=1507246572675;turbo=1;user-id=1337;user-type=global_mod :ronni!ronni@ronni.tmi.twitch.tv PRIVMSG #ronni :Kappa Keepo Kappa";
        for msg in parse(input).map(|s| s.unwrap()) {
            let msg = Privmsg::from_irc(msg).unwrap();

            assert!(msg.name() == "ronni");
            assert!(msg.channel() == "#ronni");
            assert!(msg.data() == "Kappa Keepo Kappa");
            assert!(msg.ctcp() == None);

            assert!(msg.badge_info().unwrap() == vec![]);
            assert!(msg.badges().unwrap() == vec![Badge::GlobalMod, Badge::Turbo]);
            assert!(msg.color().unwrap() == "#0D4200".parse().unwrap())
            assert!(msg.display_name().unwrap() == "ronni")
            assert!(msg.emotes().unwrap() == vec![emote!(25,(0..4),(12..16)),
            emote!(1902,(6..10))]);
            assert!(msg.id().unwrap() == "b34ccfc7-4977-403a-8a94-33c6bac34fb8");
            assert!(msg.r#mod().unwrap() == true);
            assert!(msg.room_id().unwrap() == 1337);
            assert!(msg.subscriber().unwrap() == false);
            assert!(msg.tmi_sent_ts().unwrap() == 1507246572675);
            assert!(msg.turbo().unwrap() == true);
            assert!(msg.user_id().unwrap() == 1337);
            assert!(msg.user_type().unwrap() == "global_mod");
        }
    }


    #[test]
    fn privmsg_boundary() {
        let input = ":test!user@host PRIVMSG #museun :\u{FFFD}\u{1F468}\r\n";
        for msg in parse(input).map(|s| s.unwrap()) {
            let msg = Privmsg::from_irc(msg).unwrap();

            assert_eq!(msg.name(), "test");
            assert_eq!(msg.channel(), "#museun");
            assert_eq!(msg.data(), "\u{FFFD}\u{1F468}");
            assert_eq!(msg.ctcp(), None);
        }
    }

    #[test]
    fn privmsg_action() {
        let input = ":test!user@host PRIVMSG #museun :\x01ACTION this is a test\x01\r\n";
        for msg in parse(input).map(|s| s.unwrap()) {
            let msg = Privmsg::from_irc(msg).unwrap();

            assert_eq!(msg.name(), "test");
            assert_eq!(msg.channel(), "#museun");
            assert_eq!(msg.data(), "this is a test");
            assert_eq!(msg.ctcp().unwrap(), Ctcp::Action);
        }
    }

    #[test]
    fn privmsg_unknown() {
        let input = ":test!user@host PRIVMSG #museun :\x01FOOBAR this is a test\x01\r\n";
        for msg in parse(input).map(|s| s.unwrap()) {
            let msg = Privmsg::from_irc(msg).unwrap();

            assert_eq!(msg.name(), "test");
            assert_eq!(msg.channel(), "#museun");
            assert_eq!(msg.data(), "this is a test");
            assert_eq!(msg.ctcp().unwrap(), Ctcp::Unknown { command: "FOOBAR" });
        }
    }

    #[test]
    fn privmsg_community_rewards() {
        let input = "@custom-reward-id=abc-123-foo;msg-id=highlighted-message :test!user@host PRIVMSG #museun :Notice me!\r\n";
        for msg in parse(input).map(|s| s.unwrap()) {
            let msg = Privmsg::from_irc(msg).unwrap();
            assert_eq!(msg.name(), "test");
            assert_eq!(msg.channel(), "#museun");
            assert_eq!(msg.data(), "Notice me!");
            assert_eq!(msg.custom_reward_id().unwrap(), "abc-123-foo");
            assert_eq!(msg.msg_id().unwrap(), "highlighted-message");
        }
    }

    // #[test]
    // fn privmsg_badges_iter() {
    //     let input = "@badge-info=;badges=broadcaster/1;color=#FF69B4;display-name=museun;emote-only=1;emotes=25:0-4,6-10/81274:12-17;flags=;id=4e160a53-5482-4764-ba28-f224cd59a51f;mod=0;room-id=23196011;subscriber=0;tmi-sent-ts=1601079032426;turbo=0;user-id=23196011;user-type= :museun!museun@museun.tmi.twitch.tv PRIVMSG #museun :Kappa Kappa VoHiYo\r\n";
    //     for msg in parse(input).map(|s| s.unwrap()) {
    //         let msg = Privmsg::from_irc(msg).unwrap();
    //         assert_eq!(msg.iter_badges().count(), 1);
    //     }
    // }

    // #[test]
    // fn privmsg_emotes_iter() {
    //     let input = "@badge-info=;badges=broadcaster/1;color=#FF69B4;display-name=museun;emote-only=1;emotes=25:0-4,6-10/81274:12-17;flags=;id=4e160a53-5482-4764-ba28-f224cd59a51f;mod=0;room-id=23196011;subscriber=0;tmi-sent-ts=1601079032426;turbo=0;user-id=23196011;user-type= :museun!museun@museun.tmi.twitch.tv PRIVMSG #museun :Kappa Kappa VoHiYo\r\n";
    //     for msg in parse(input).map(|s| s.unwrap()) {
    //         let msg = Privmsg::from_irc(msg).unwrap();
    //         assert_eq!(msg.iter_emotes().count(), 2);
    //     }
    // }
}
