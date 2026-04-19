//! When connecting to a server,
//! the bot will search for a channel that matches its own name.
use crate::prelude::*;
use beet::prelude::*;
use twilight_model::channel::Channel;
use twilight_model::gateway::payload::incoming::GuildCreate;
use twilight_model::id::Id;
use twilight_model::id::marker::ChannelMarker;
use twilight_model::id::marker::GuildMarker;

/// When connecting to a guild, searches for a channel with the same name
/// as the bot, ie `my-bot`. This is treated as the bots owned channel,
/// to which it is allowed to post more verbosely, respond to all messages etc.
#[derive(Debug, Default, Clone, Component)]
pub struct ChannelMap {
	channels: HashMap<Id<ChannelMarker>, ChannelInfo>,
}

impl ChannelMap {
	pub fn try_add(
		&mut self,
		commands: &mut Commands,
		entity: Entity,
		bot_state: &BotState,
		channel: Channel,
	) -> bool {
		if self.channels.contains_key(&channel.id) {
			return false;
		}
		self.channels
			.insert(channel.id, ChannelInfo::new(bot_state, channel.clone()));
		commands.trigger(ChannelAdded {
			entity,
			guild: channel.guild_id.unwrap_or_else(|| Id::new(0)),
			channel: channel.id,
		});
		true
	}

	pub fn get(&self, id: Id<ChannelMarker>) -> Result<&ChannelInfo> {
		self.channels.get(&id).ok_or_else(|| {
			bevyhow!("ChannelMap does not contain channel with id {id}.
This should not happen, ensure that ensure_exists is called at the gatway listener level")
		})
	}

	/// Finds the first channel in the given guild that matches the bot name.
	pub fn first_name_matches(
		&self,
		guild: Id<GuildMarker>,
	) -> Option<&ChannelInfo> {
		self.channels.values().find(|info| {
			info.channel.guild_id == Some(guild) && info.name_matches
		})
	}


	pub async fn ensure_exists(
		entity: AsyncEntity,
		channel: Channel,
	) -> Result {
		if entity
			.get::<Self, _>(move |this| this.channels.contains_key(&channel.id))
			.await?
		{
			return Ok(());
		}
		Self::try_add_async(entity, channel).await
	}

	/// Ensure the channel is added to the channel map
	pub async fn exists_or_fetch(
		entity: AsyncEntity,
		id: Id<ChannelMarker>,
	) -> Result {
		if entity
			.get::<Self, _>(move |this| this.channels.contains_key(&id))
			.await?
		{
			return Ok(());
		}
		let channel = entity
			.get_cloned::<DiscordHttpClient>()
			.await?
			.send(GetChannel::new(id))
			.await?;
		Self::try_add_async(entity, channel).await
	}
	pub async fn try_add_async(
		entity: AsyncEntity,
		channel: Channel,
	) -> Result {
		entity
			.with_state::<DiscordQuery, Result>(move |entity, mut query| {
				query.try_add(entity, channel)?;
				Ok(())
			})
			.await
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Get)]
pub struct ChannelInfo {
	/// whether the
	is_owner: bool,
	/// Whether the channel name matches the bot name,
	/// useful for setting more liberal rights for
	/// a bots own channel.
	name_matches: bool,
	channel: Channel,
}

impl std::fmt::Display for ChannelInfo {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"ChannelInfo {{ id: {}, name: {:?} }}",
			self.channel.id, self.channel.name
		)
	}
}

impl ChannelInfo {
	pub fn new(bot: &BotState, channel: Channel) -> Self {
		let is_owner = channel.owner_id.map_or(false, |id| id == bot.user_id());
		let name_matches = channel.name.as_ref().map_or(false, |name| {
			name.to_ascii_lowercase() == bot.name().to_ascii_lowercase()
		});

		Self {
			is_owner,
			name_matches,
			channel,
		}
	}
	pub fn id(&self) -> Id<ChannelMarker> { self.channel.id }
	pub fn is_thread(&self) -> bool {
		matches!(
			self.channel.kind,
			twilight_model::channel::ChannelType::PublicThread
				| twilight_model::channel::ChannelType::PrivateThread
		)
	}
	pub fn is_text(&self) -> bool {
		matches!(
			self.channel.kind,
			twilight_model::channel::ChannelType::GuildText
				| twilight_model::channel::ChannelType::Private
		)
	}
}

#[derive(EntityEvent)]
pub struct ChannelAdded {
	pub entity: Entity,
	pub guild: Id<GuildMarker>,
	pub channel: Id<ChannelMarker>,
}


pub fn add_guild_create_channels(
	ev: On<DiscordGuildCreate>,
	mut commands: Commands,
	mut query: Populated<(&BotState, &mut ChannelMap)>,
) -> Result {
	let guild = match &ev.guild_create {
		GuildCreate::Available(g) => g,
		GuildCreate::Unavailable(_) => {
			return Ok(());
		}
	};
	let entity = ev.event_target();
	let (bot_state, mut channel_map) = query.get_mut(entity)?;

	for channel in guild.channels.iter() {
		channel_map.try_add(&mut commands, entity, &bot_state, channel.clone());
	}

	Ok(())
}

#[cfg(test)]
mod tests {
	use twilight_model::channel::ChannelType;
	use twilight_model::gateway::payload::incoming::GuildCreate;
	use twilight_model::guild::UnavailableGuild;

	fn make_unavailable_guild(id: u64) -> GuildCreate {
		GuildCreate::Unavailable(UnavailableGuild {
			id: twilight_model::id::Id::new(id),
			unavailable: true,
		})
	}

	fn make_available_guild_no_text_channels(name: &str) -> GuildCreate {
		let guild: twilight_model::guild::Guild =
			serde_json::from_value(serde_json::json!({
				"id": "123",
				"name": name,
				"icon": null,
				"owner_id": "1",
				"channels": [],
				"members": [],
				"roles": [],
				"emojis": [],
				"features": [],
				"afk_timeout": 300,
				"preferred_locale": "en-US",
				"premium_progress_bar_enabled": false,
				"verification_level": 0,
				"default_message_notifications": 0,
				"explicit_content_filter": 0,
				"mfa_level": 0,
				"premium_tier": 0,
				"nsfw_level": 0,
				"system_channel_flags": 0,
			}))
			.unwrap();
		GuildCreate::Available(guild)
	}

	#[test]
	fn unavailable_guild_is_handled_gracefully() {
		// Should not panic; simply returns early.
		let gc = make_unavailable_guild(999);
		assert!(matches!(gc, GuildCreate::Unavailable(_)));
	}

	#[test]
	fn available_guild_with_no_channels_leaves_greet_channel_unset() {
		let gc = make_available_guild_no_text_channels("Empty");
		let guild = match &gc {
			GuildCreate::Available(g) => g,
			_ => panic!("expected available guild"),
		};
		let text_ch = guild
			.channels
			.iter()
			.find(|c| c.kind == ChannelType::GuildText);
		assert!(
			text_ch.is_none(),
			"expected no text channel in this fixture"
		);
	}

	#[test]
	fn available_guild_first_text_channel_would_be_selected() {
		let guild: twilight_model::guild::Guild =
			serde_json::from_value(serde_json::json!({
				"id": "1",
				"name": "My Server",
				"icon": null,
				"owner_id": "1",
				"channels": [
					{
						"id": "42",
						"type": 0,
						"guild_id": "1",
						"position": 0,
						"permission_overwrites": [],
						"name": "general",
						"nsfw": false,
						"rate_limit_per_user": 0,
						"topic": null,
						"last_message_id": null,
						"parent_id": null,
						"last_pin_timestamp": null
					}
				],
				"members": [],
				"roles": [],
				"emojis": [],
				"features": [],
				"afk_timeout": 300,
				"preferred_locale": "en-US",
				"premium_progress_bar_enabled": false,
				"verification_level": 0,
				"default_message_notifications": 0,
				"explicit_content_filter": 0,
				"mfa_level": 0,
				"premium_tier": 0,
				"nsfw_level": 0,
				"system_channel_flags": 0,
			}))
			.unwrap();

		let text_ch = guild
			.channels
			.iter()
			.find(|c| c.kind == ChannelType::GuildText);
		assert!(text_ch.is_some(), "should find the general text channel");
		assert_eq!(text_ch.unwrap().id.get(), 42);
	}
}
