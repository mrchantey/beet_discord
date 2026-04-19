use crate::prelude::*;
use beet::prelude::*;

pub fn default_bot() -> impl Bundle {}

#[derive(Component)]
#[component(on_add=on_add)]
#[require(ChannelMap)]
pub struct DiscordBot {
	/// The bot's token, usually loaded from the environment at startup.
	token: String,
}

impl DiscordBot {
	pub fn new(token: String) -> Self { Self { token } }
	pub fn token(&self) -> &str { &self.token }
}

#[allow(unused)]
fn on_add(mut world: DeferredWorld, cx: HookContext) {
	let entity = cx.entity;

	let token = world
		.entity(entity)
		.get::<DiscordBot>()
		.unwrap()
		.token
		.clone();
	let mut commands = world.commands();
	// TODO http client should just be this bot
	commands
		.entity(cx.entity)
		.insert(DiscordHttpClient::new(&token));

	commands.queue_async(async move |world| {
		start_gateway_listener(world.entity(entity)).await
	});
}

impl Default for DiscordBot {
	fn default() -> Self {
		Self {
			token: env_ext::var("DISCORD_TOKEN").unwrap(),
		}
	}
}
