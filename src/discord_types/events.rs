use std::sync::Arc;

use beet::prelude::*;
use twilight_model::application::interaction::Interaction;
use twilight_model::channel::message::Message;
use twilight_model::gateway::payload::incoming::GuildCreate;
use twilight_model::gateway::payload::incoming::PresenceUpdate;
use twilight_model::gateway::payload::incoming::Ready;


/// The first dispatch message sent, often used to get the
/// bot name, app id etc.
#[derive(Debug, Clone, EntityEvent)]
pub struct DiscordReady {
	entity: Entity,
	ready: Ready,
}

impl DiscordReady {
	pub fn create(ready: Ready) -> impl FnOnce(Entity) -> Self {
		move |entity| Self { entity, ready }
	}
}

impl std::ops::Deref for DiscordReady {
	type Target = Ready;
	fn deref(&self) -> &Self::Target { &self.ready }
}

/// Sent when connecting to a server, aka [`Guild`].
/// A common task done here is selecting a channel for the bot
/// to use.
#[derive(Debug, Clone, EntityEvent)]
pub struct DiscordGuildCreate {
	entity: Entity,
	pub guild_create: GuildCreate,
}

impl DiscordGuildCreate {
	pub fn create(guild_create: GuildCreate) -> impl FnOnce(Entity) -> Self {
		move |entity| Self {
			entity,
			guild_create,
		}
	}
}

/// Sent when a user comes online or offline.
/// A common task here is greeting users as they come online.
#[derive(Debug, Clone, EntityEvent)]
pub struct DiscordPresenceUpdate {
	entity: Entity,
	pub presence: PresenceUpdate,
}

impl DiscordPresenceUpdate {
	pub fn create(presence: PresenceUpdate) -> impl FnOnce(Entity) -> Self {
		move |entity| Self { entity, presence }
	}
}

impl std::ops::Deref for DiscordPresenceUpdate {
	type Target = PresenceUpdate;

	fn deref(&self) -> &Self::Target { &self.presence }
}

/// Sent when a message is sent in a channel the bot can see,
/// including messages sent by the bot itself.
#[derive(Debug, Clone, EntityEvent)]
pub struct DiscordMessage {
	entity: Entity,
	pub message: Arc<Message>,
}

impl DiscordMessage {
	pub fn create(message: Message) -> impl FnOnce(Entity) -> Self {
		move |entity| Self {
			entity,
			message: Arc::new(message),
		}
	}
}

impl std::ops::Deref for DiscordMessage {
	type Target = Message;
	fn deref(&self) -> &Self::Target { &self.message }
}


/// Sent when a user invokes a slash command or other interaction like
/// clicking a button.
#[derive(Debug, Clone, EntityEvent)]
pub struct DiscordInteraction {
	entity: Entity,
	pub interaction: Interaction,
}

impl DiscordInteraction {
	pub fn create(interaction: Interaction) -> impl FnOnce(Entity) -> Self {
		move |entity| Self {
			entity,
			interaction,
		}
	}
}

impl std::ops::Deref for DiscordInteraction {
	type Target = Interaction;
	fn deref(&self) -> &Self::Target { &self.interaction }
}
