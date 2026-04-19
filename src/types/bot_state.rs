use crate::beet_thread::discord_meta::DiscordActorMeta;
use crate::beet_thread::discord_meta::DiscordMeta;
use crate::prelude::*;
use beet::prelude::*;
use std::time::Instant;
use tracing::info;

use twilight_model::id::Id;
use twilight_model::id::marker::ApplicationMarker;
use twilight_model::id::marker::UserMarker;


/// Core bot identity and lifecycle state.
#[derive(Debug, Clone, Component)]
pub struct BotState {
	/// The bot's own user ID
	bot_user_id: Id<UserMarker>,
	/// The application ID
	application_id: Id<ApplicationMarker>,
	/// The bot's username, not unique
	bot_name: String,
	/// Global application name
	global_name: Option<String>,
	/// Timestamp of when the bot started.
	start_time: Instant,
}

impl BotState {
	pub fn user_id(&self) -> Id<UserMarker> { self.bot_user_id }
	pub fn name(&self) -> &str { &self.bot_name }
	pub fn global_name(&self) -> Option<&str> { self.global_name.as_deref() }
	pub fn application_id(&self) -> Id<ApplicationMarker> {
		self.application_id
	}
	pub fn start_time(&self) -> Instant { self.start_time }

	pub fn create_actor(&self) -> Actor {
		let actor_name = self.name().to_string();
		let mut actor = Actor::new(actor_name, ActorKind::Agent);
		let meta = DiscordActorMeta {
			discord_user: Some(self.user_id().get()),
		};
		meta.write_to(actor.metadata_mut());
		actor
	}
}

/// Called when the bot receives the READY event from the gateway.
///
/// Stores identity information in [`BotState`] and registers slash commands
/// globally (once per session).
pub fn init_bot_state(ev: On<DiscordReady>, mut commands: Commands) -> Result {
	let entity = ev.event_target();


	let state = BotState {
		bot_user_id: ev.user.id,
		application_id: ev.application.id,
		bot_name: ev.user.name.clone(),
		global_name: ev.user.global_name.clone(),
		start_time: Instant::now(),
	};
	info!("bot is ready:{state:#?}");
	commands.entity(entity).insert(state);
	Ok(())
}
