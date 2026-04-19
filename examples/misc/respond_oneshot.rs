use crate::prelude::*;
use beet::prelude::*;


pub fn respond_oneshot(
	ev: On<DiscordMessage>,
	mut commands: Commands,
	query: DiscordQuery,
) -> Result {
	if !query.should_respond(ev.event_target(), &ev)? {
		return Ok(());
	}
	let channel = query.channel(ev.event_target(), &ev)?;

	let msg_id = ev.message.id;
	let actor_kind = if ev.message.author.bot {
		ActorKind::Agent
	} else {
		ActorKind::User
	};
	let should_create_thread = channel.is_text();
	let is_thread = channel.is_thread();
	let actor = Actor::new(&ev.message.author.name, actor_kind);
	let channel_id = ev.message.channel_id;
	let content = ev.message.content.clone();
	commands
		.entity(ev.event_target())
		.queue_async(async move |entity| {
			let http = entity.get_cloned::<DiscordHttpClient>().await?;
			http.send(CreateTypingTrigger::new(channel_id)).await?;
			let mut target_channel = channel_id;
			let mut target_message = Some(msg_id);
			if should_create_thread {
				let thread = CreateThreadFromMessage::new(
					channel_id,
					msg_id,
					"New Thread",
				)
				.auto_archive_duration(60);
				let channel = http.send(thread).await?;
				target_channel = channel.id;
				target_message = None;
			}
			if is_thread {
				// never reply to messages in a thread
				target_message = None;
			}
			// http.send(CreateReaction::new(channel_id, msg_id, "👍"))
			// 	.await?;
			// let text = format!("You sent me a DM with content: {}", content);
			thread_utils::send_oneshot(
				&http,
				actor,
				&content,
				target_channel,
				target_message,
			)
			.await?;


			Ok(())
		});


	Ok(())
}
