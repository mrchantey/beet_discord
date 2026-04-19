use crate::prelude::Message;
use crate::prelude::*;
use beet::prelude::*;

#[derive(SystemParam)]
pub struct DiscordQuery<'w, 's> {
	commands: Commands<'w, 's>,
	pub bots: Query<'w, 's, (&'static BotState, &'static mut ChannelMap)>,
}

impl<'w, 's> DiscordQuery<'w, 's> {
	/// May Respond if:
	/// - not sent by this bot
	/// and any of the following
	/// - mentions this bot
	/// - in a channel with this bots name
	/// - in a channel/thread created by this bot
	pub fn should_respond(
		&self,
		bot: Entity,
		message: &Message,
	) -> Result<bool> {
		let (bot_state, channel_map) = self.bots.get(bot)?;
		let channel = channel_map.get(message.channel_id)?;
		if message.author.id == bot_state.user_id() {
			// i sent the message, do not respond
			false
		} else if
		// mentions bot
		message.mentions_user(bot_state.user_id())
		// its a channel with the same name of
			|| channel.name_matches()
			|| channel.is_owner()
		{
			true
		} else {
			false
		}
		.xok()
	}

	pub fn should_create_thread(
		&self,
		bot: Entity,
		message: &Message,
	) -> Result<bool> {
		let (_, channel_map) = self.bots.get(bot)?;
		let channel = channel_map.get(message.channel_id)?;
		channel.is_text().xok()
	}

	pub fn channel(
		&self,
		entity: Entity,
		message: &Message,
	) -> Result<&ChannelInfo> {
		let (_bot_state, channel_map) = self.bots.get(entity)?;
		let channel = channel_map.get(message.channel_id)?;
		channel.xok()
	}

	pub fn try_add(
		&mut self,
		entity: Entity,
		channel: Channel,
	) -> Result<bool> {
		let (bot_state, mut channels) = self.bots.get_mut(entity)?;
		channels
			.try_add(&mut self.commands, entity, bot_state, channel)
			.xok()
	}
}
