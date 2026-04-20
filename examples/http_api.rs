//!
//!

use beet::prelude::*;
use beet_discord::prelude::*;

fn main() -> AppExit {
	env_ext::load_dotenv();

	App::new()
		.add_plugins((
			MinimalPlugins,
			// LogPlugin {
			// 	level: Level::TRACE,
			// 	..default()
			// },
			RouterPlugin,
		))
		.add_systems(Startup, setup)
		.run()
}

fn setup(mut commands: Commands) -> Result {
	commands.spawn((
		DiscordBot::default(),
		CliServer::default(),
		router(),
		children![
			GetGuildAction,
			// route("about", BlobScene::new("content/about.md")),
		],
	));
	Ok(())
}
