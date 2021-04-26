use crate::irc::tags::ParsedTag;
use crate::{irc::*, MaybeOwned, MaybeOwnedIndex, Validator};
use parse_display::{Display, FromStr};

/// The parameters for a room being in follower-only mode
#[derive(Debug, Copy, Clone, PartialEq, Display, FromStr)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[display("{}")]
pub enum FollowersOnly {
    /// The mode is disabled
    #[from_str(regex = "-1")]
    Disabled,
    /// All followers are allowed to speak
    #[from_str(regex = "0")]
    All,
    /// Only those following for `n` days are allowed to speak
    #[display("{}({0})")]
    #[from_str(regex = "(?P<0>[0-9]+)")]
    Limit(isize),
}

/// Identifies the channel's chat settings (e.g., slow mode duration).
#[derive(Clone, PartialEq)]
pub struct RoomState<'a> {
    raw: MaybeOwned<'a>,
    tags: TagIndices,
    channel: MaybeOwnedIndex,
}

impl<'a> FromIrcMessage<'a> for RoomState<'a> {
    type Error = MessageError;
    fn from_irc(msg: IrcMessage<'a>) -> Result<Self, Self::Error> {
        msg.expect_command(IrcMessage::ROOM_STATE)?;

        let this = Self {
            tags: msg.parse_tags(),
            channel: msg.expect_arg_index(0)?,
            raw: msg.raw,
        };

        Ok(this)
    }

    into_inner_raw!();
}

impl<'a> RoomState<'a> {
    raw!();
    tags!();
    str_field!(
        /// The channel that this event is happening on
        channel
    );

    /// Whether this room is in emote only mode
    pub fn is_emote_only(&self) -> bool {
        self.tags().get_as_bool("emote-only")
    }

    /// Whether this room is in followers only mode
    pub fn is_followers_only(&self) -> Option<ParsedTag<FollowersOnly>> {
        self.tags().get_parsed("followers-only")
    }

    /// Whether this room is in r9k mode
    pub fn is_r9k(&self) -> bool {
        self.tags().get_as_bool("r9k")
    }

    /// The id of the room this message was sent to
    pub fn room_id(&self) -> Option<ParsedTag<u64>> {
        self.tags().get_parsed("room-id")
    }

    /// Whether this room is in slow mode
    ///
    /// This returns the delay in which each message can be sent.
    pub fn is_slow_mode(&self) -> Option<u64> {
        // only return Some(_) if tag was found, correctly parsed, and its value is larger than 0
        self.tags()
            .get_parsed("slow")
            .transpose()
            .ok()
            .flatten()
            .filter(|&s| s > 0)
    }

    /// Whether this room is in subs only mode
    pub fn is_subs_only(&self) -> bool {
        self.tags().get_as_bool("subs-only")
    }
}

into_owned!(RoomState { raw, tags, channel });
impl_custom_debug!(RoomState { raw, tags, channel });
serde_struct!(RoomState { raw, tags, channel });

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    #[cfg(feature = "serde")]
    fn user_state_serde() {
        let input = ":tmi.twitch.tv ROOMSTATE #museun\r\n";
        crate::serde::round_trip_json::<RoomState>(input);
        crate::serde::round_trip_rmp::<RoomState>(input);
    }

    #[test]
    fn user_state() {
        let input = ":tmi.twitch.tv ROOMSTATE #museun\r\n";
        for msg in parse(input).map(|s| s.unwrap()) {
            let msg = RoomState::from_irc(msg).unwrap();
            assert_eq!(msg.channel(), "#museun");
        }
    }

    #[test]
    fn test_followers_only_parsing() {
        const EXPECTED: &[(&str, FollowersOnly)] = &[
            ("-1", FollowersOnly::Disabled),
            ("0", FollowersOnly::All),
            ("4", FollowersOnly::Limit(4)),
            ("31415", FollowersOnly::Limit(31415)),
        ];

        EXPECTED
            .iter()
            .for_each(|(s, mode)| assert_eq!(FollowersOnly::from_str(s), Ok(*mode)));
    }

    #[test]
    fn test_followers_only_invalid_parsing() {
        const INVALID: &[&str] = &["-2", "!", "invalid", ""];
        INVALID.iter().for_each(|s| {
            println!("{}", s);
            assert!(FollowersOnly::from_str(s).is_err())
        });
    }
}
