//! Discord bot entry point.
//!
//! All transport details live in `discord_io/gateway` (WebSocket) and
//! `discord_io/http` (REST). This file is purely bot logic: reacting to
//! typed events.
use beet::prelude::*;
use beet_discord::prelude::*;



fn main() {
	env_ext::load_dotenv();
	App::new()
		.add_plugins((
			MinimalPlugins,
			LogPlugin {
				// level: Level::TRACE,
				..default()
			},
			DiscordPlugin,
		))
		.add_systems(Startup, spawn_bot)
		.run();
}

/// Startup system that spawns the discord bot.
fn spawn_bot(mut commands: Commands) {
	commands
		.spawn((
			DiscordBot::default(),
			FsBucket::new(WsPathBuf::new(".beet")),
		))
		.observe(init_bot_state)
		.observe(add_guild_create_channels)
		.observe(thread_sync::handle_message)
		// .observe(on_join_bot_channel)
		// .observe(respond_to_message)
	;
}
