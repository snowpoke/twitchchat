use crate::twitch::{Attribution, AttributionVec, BadgeVec};
use crate::{irc::*, twitch::*, IntoOwned, MaybeOwned, Validator};
use std::str::FromStr;

/// Sent on successful login, if both **TAGS** and **COMMANDS** capabilities have been sent beforehand.
///
/// # NOTE:
///
/// Because Twitch is extremely inconsistent in its documentation you can get this message without any Tags attached.
///
/// If only **COMMANDS** and **MEMBERSHIP** are sent, you'll get this message,
/// but it'll be empty (read: default). You should check the [GlobalUserState::has_tags()] to
/// verify that you actually got the real message
#[derive(Clone, PartialEq)]
pub struct GlobalUserState<'a> {
    raw: MaybeOwned<'a>,
    tags: TagIndices,
    /// Your user-id, if you have Tags enabled
    pub user_id: Option<MaybeOwned<'a>>,
    /// Your display name, if set   
    pub display_name: Option<MaybeOwned<'a>>,
    /// Your color, if set. Defaults to `white`
    pub color: Color,
}

impl<'a> GlobalUserState<'a> {
    raw!();
    tags!();

    /// Determines whether this message actually had tags attached
    pub fn has_tags(&self) -> bool {
        !self.tags.is_empty()
    }

    /// Your available emote sets, always contains atleast '0'
    pub fn emote_sets(&self) -> Vec<&str> {
        self.tags()
            .get("emote-sets")
            .map(|s| s.split(',').collect())
            .unwrap_or_else(|| vec!["0"])
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

    /// Any badges you have
    pub fn badges(&self) -> BadgeVec {
        self.tag_to_attribution_vec("badges")
    }

    /// Your user-id -- only available if you have TAGs enabled
    pub fn user_id(&self) -> Option<&str> {
        self.user_id.as_deref()
    }

    /// Your display name, if set   
    pub fn display_name(&self) -> Option<&str> {
        self.display_name.as_deref()
    }

    /// Your color, if set. Defaults to `white`
    pub fn color(&self) -> Color {
        self.color
    }
}

impl<'a> FromIrcMessage<'a> for GlobalUserState<'a> {
    type Error = MessageError;

    fn from_irc(msg: IrcMessage<'a>) -> Result<Self, Self::Error> {
        msg.expect_command(IrcMessage::GLOBAL_USER_STATE)?;

        let tag_index = msg.parse_tags();
        let tags = Tags {
            data: &msg.raw,
            indices: &tag_index,
        };

        let user_id = tags
            .get("user-id")
            .map(MaybeOwned::from)
            .map(MaybeOwned::into_owned);

        let display_name = tags
            .get("display-name")
            .map(MaybeOwned::from)
            .map(MaybeOwned::into_owned);

        let color = tags
            .get("color")
            .filter(|s| !s.is_empty())
            .map(std::str::FromStr::from_str)
            .transpose()
            .map_err(|err| MessageError::CannotParseTag {
                name: "color".into(),
                error: Box::new(err),
            })?
            .unwrap_or_default();

        let this = Self {
            user_id,
            display_name,
            color,
            tags: tag_index,
            raw: msg.raw,
        };

        Ok(this)
    }

    into_inner_raw!();
}

into_owned!(GlobalUserState {
    raw,
    tags,
    user_id,
    display_name,
    color,
});

impl_custom_debug!(GlobalUserState {
    raw,
    tags,
    emote_sets,
    badges,
    user_id,
    display_name,
    color,
});

serde_struct!(GlobalUserState {
    raw,
    tags,
    user_id,
    display_name,
    color,
    badges,
    emote_sets,
});

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::hashset;
    use assert2::assert;

    #[test]
    #[cfg(feature = "serde")]
    fn global_user_state_serde() {
        let input = "@badge-info=;badges=;color=#FF69B4;display-name=shaken_bot;emote-sets=0;user-id=241015868;user-type= :tmi.twitch.tv GLOBALUSERSTATE\r\n";
        crate::serde::round_trip_json::<GlobalUserState>(input);
        crate::serde::round_trip_rmp::<GlobalUserState>(input);
    }

    #[test]
    fn global_user_state_integrity() {
        let input = "@badge-info=subscriber/8;badges=subscriber/6;color=#0D4200;display-name=dallas;emote-sets=0,33,50,237,793,2126,3517,4578,5569,9400,10337,12239;turbo=0;user-id=1337;user-type=admin :tmi.twitch.tv GLOBALUSERSTATE
        \r\n";
        for msg in parse(input).map(|s| s.unwrap()) {
            let msg = GlobalUserState::from_irc(msg).unwrap();
            assert!(msg.badge_info().unwrap() == vec![BadgeInfo::NoTierSubscriber(8)]);
            assert!(msg.badges().unwrap() == vec![Badge::NoTierSubscriber(6)]);
            let color = "#0D4200".parse().unwrap();
            assert!(msg.color == color);
            assert!(msg.color() == color);
            assert!(msg.display_name().unwrap() == "dallas");

            assert!(msg.emote_sets().unwrap() == hashset!{0,33,50,237,793,2126,3517,4578,5569,9400,10337,12239});
            assert!(msg.turbo().unwrap() == false);
            assert!(msg.user_id().unwrap() == "1337");
            assert!(msg.user_type().unwrap() == "admin");
        }
    }

    #[test]
    fn global_user_state_no_tags() {
        let input = ":tmi.twitch.tv GLOBALUSERSTATE\r\n";
        for msg in parse(input).map(|s| s.unwrap()) {
            let msg = GlobalUserState::from_irc(msg).unwrap();
            assert!(msg.user_id().is_none());
            assert!(msg.display_name().is_none());
            assert_eq!(msg.color(), crate::twitch::Color::default());
            assert_eq!(msg.emote_sets().unwrap(), hashset!{0});
        }
    }

    #[test]
    fn global_user_state_empty_color() {
        let input = "@badge-info=;badges=;color=;display-name=shaken_bot;emote-sets=0;user-id=241015868;user-type= :tmi.twitch.tv GLOBALUSERSTATE\r\n";
        for msg in parse(input).map(|s| s.unwrap()) {
            let msg = GlobalUserState::from_irc(msg).unwrap();
            assert_eq!(msg.user_id().unwrap(), "241015868");
            assert_eq!(msg.display_name().unwrap(), "shaken_bot");
            assert_eq!(msg.color(), crate::twitch::Color::default());
            assert_eq!(msg.emote_sets(), vec!["0"]);
        }
    }
}
