use crate::prelude::*;
use beet::prelude::*;

pub fn on_join_bot_channel(
	ev: On<ChannelAdded>,
	mut commands: Commands,
	query: Query<&ChannelMap>,
) -> Result {
	let channel_map = query.get(ev.event_target())?;
	let channel_info = channel_map.get(ev.channel)?;
	if !channel_info.name_matches() {
		return Ok(());
	}
	let channel_id = ev.channel;
	// ?	let
	commands
		.entity(ev.event_target())
		.queue_async(async move |entity| {
			let http = entity.get_cloned::<DiscordHttpClient>().await?;

			for guild in http.send(GetCurrentUserGuilds::new()).await? {
				info!("Member of guild: {} - {}", guild.name, guild.id);
			}

			http.send(CreateTypingTrigger::new(channel_id)).await?;
			thread_utils::send_oneshot(
				&http,
				Actor::developer(),
				r#"
You just rejoined your own discord channel after some time,
greet the user!
"#,
				channel_id,
				None,
			)
			.await?;
			Ok(())
		});
	Ok(())
}
