use crate::prelude::*;
use beet::prelude::*;


pub struct DiscordPlugin;

impl Plugin for DiscordPlugin {
	fn build(&self, app: &mut App) {
		app.init_plugin::<ThreadPlugin>()
			.add_systems(Update, thread_sync::send_posts);
	}
}
