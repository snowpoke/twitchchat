use crate::irc::tags::ParsedTag;
use crate::{irc::*, MaybeOwned, MaybeOwnedIndex, Validator};
use crate::messages::tags::HasTags;
use twitchchat_macros::irc_tags;
use std::time::Duration;
use pipe_trait::Pipe;
use wrap_result::WrapOk;


/// The parameters for a room being in follower-only mode
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub enum FollowersOnly {
    /// The mode is disabled
    Disabled,
    /// All followers are allowed to speak
    All,
    /// Only those following for `n` minutes are allowed to speak
    Limit(Duration),
}

// std::time::Duration doesn't implement FromStr, so we can't use parse_display::FromStr
impl std::str::FromStr for FollowersOnly {
    type Err = std::num::ParseIntError;
    fn from_str(s: &str) -> Result<FollowersOnly, Self::Err> {
        let minutes_to_duration = |x| Duration::from_secs(x*86400);
        let duration_to_limit = |x| FollowersOnly::Limit(x);
        match s {
            "-1" => FollowersOnly::Disabled.wrap_ok(),
            "0" => FollowersOnly::All.wrap_ok(),
            s => u64::from_str(s)?.pipe(minutes_to_duration).pipe(duration_to_limit).wrap_ok()
        }
    }
}

impl FollowersOnly {
    /// Transforms FollowersOnly into an Option containing the time restriction duration.
    pub fn optional(&self) -> Option<Duration> {
        match self {
            Self::Disabled => None,
            Self::All => Some(Duration::new(0,0)),
            Self::Limit(duration) => Some(*duration),
        }
    }
}

/// Identifies the channel's chat settings (e.g., slow mode duration).
#[irc_tags["emote-only", "followers-only", "r9k", "slow", "subs-only"]]
#[derive(Clone, PartialEq)]
pub struct RoomState<'a> {
    raw: MaybeOwned<'a>,
    tags: TagIndices,
    channel: MaybeOwnedIndex,
}

impl<'a> HasTags<'a> for RoomState<'a> {
    fn tags(&'a self) -> Tags<'a>{
        Tags {
            data: &self.raw,
            indices: &self.tags,
        }
    }
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

    //TODO: Bring this method back
    // /// Whether this room is in followers only mode
    // pub fn is_followers_only(&self) -> bool {
    //     !self.followers_only().is_disabled()
    // }

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
    //use pipe_trait::Pipe;
    use assert2::assert;

    #[test]
    #[cfg(feature = "serde")]
    fn room_state_serde() {
        let input = ":tmi.twitch.tv ROOMSTATE #museun\r\n";
        crate::serde::round_trip_json::<RoomState>(input);
        crate::serde::round_trip_rmp::<RoomState>(input);
    }

    /// Tests whether ROOMSTATE messages are parsed into RoomState structs correctly.
    #[test]
    fn room_state_stability() {
        let input = ":tmi.twitch.tv ROOMSTATE #museun\r\n";

        for msg in parse(input).map(|s| s.unwrap()) {
            let msg = RoomState::from_irc(msg).unwrap();
        }
    }

    /// Tests whether the parts of a full ROOMSTATE message can be accessed as expected. 
    #[test]
    fn room_state_integrity() {
        let input = "@emote-only=0;followers-only=0;r9k=0;slow=0;subs-only=0 :tmi.twitch.tv ROOMSTATE #dallas\r\n";
        let msg = parse(input).next().unwrap().unwrap().pipe(RoomState::from_irc);
        assert!(msg.emote_only().unwrap() == false);
        assert!(msg.followers_only().unwrap() == FollowersOnly::All);
        assert!(msg.r9k().unwrap() == true);
        assert!(msg.slow().unwrap() == 0);
        assert!(msg.subs_only().unwrap() == false);
        assert!(msg.channel().unwrap() == "#dallas");
    }

    #[test]
    fn test_followers_only_parsing() {
        const EXPECTED: &[(&str, FollowersOnly)] = &[
            ("-1", FollowersOnly::Disabled),
            ("0", FollowersOnly::All),
            ("4", FollowersOnly::Limit(Duration::from_secs(4*86400))),
            ("31415", FollowersOnly::Limit(Duration::from_secs(31415*86400))),
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
