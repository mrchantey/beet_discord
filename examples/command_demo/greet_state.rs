use crate::prelude::*;
use beet::prelude::*;
use twilight_model::gateway::presence::Status;
use twilight_model::gateway::presence::UserOrId;
use twilight_model::id::Id;
use twilight_model::id::marker::UserMarker;

/// State for the "greet users who come online" feature.

#[derive(Component, Default)]
#[component(on_add=on_add)]
#[require(ChannelMap)]
pub struct GreetState {
	/// Users we've already greeted this session (to avoid spamming).
	pub greeted_users: HashSet<Id<UserMarker>>,
}

fn on_add(mut world: DeferredWorld, cx: HookContext) {
	world
		.commands()
		.entity(cx.entity)
		.observe(greet_users_coming_online);
}

/// Observer called when a user's presence changes.
///
/// Sends a one-time greeting when a user comes online for the first time
/// this session.
fn greet_users_coming_online(
	ev: On<DiscordPresenceUpdate>,
	mut commands: Commands,
	mut query: Populated<(
		&BotState,
		&ChannelMap,
		&mut GreetState,
		&DiscordHttpClient,
	)>,
) -> Result {
	if ev.status != Status::Online {
		return Ok(());
	}

	let entity = ev.event_target();

	let user_id = match &ev.user {
		UserOrId::User(u) => u.id,
		UserOrId::UserId { id } => *id,
	};

	let (bot_state, channel_map, mut greet_state, http) =
		query.get_mut(entity)?;
	// Skip if this is the bot itself.
	if bot_state.user_id() == user_id {
		return Ok(());
	}
	// Skip if already greeted this session.
	if greet_state.greeted_users.contains(&user_id) {
		return Ok(());
	}
	// if bot has no channel do nothing
	let Some(channel_info) =
		channel_map.first_name_matches(ev.guild_id).cloned()
	else {
		return Ok(());
	};

	greet_state.greeted_users.insert(user_id);

	let http = http.clone();
	info!(
		user_id = %user_id,
		channel_info = %channel_info,
		"greeting user coming online"
	);

	commands.queue_async(async move |_| {
		let greeting = format!(
			"Welcome online, <@{}>! 🎉 Hope you're having a great day!",
			user_id
		);
		http.send(CreateMessage::new(channel_info.id()).content(&greeting))
			.await?;
		Ok(())
	});

	Ok(())
}
