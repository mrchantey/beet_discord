use crate::prelude::*;
use beet::prelude::*;
use twilight_model::id::Id;
use twilight_model::id::marker::ChannelMarker;
use twilight_model::id::marker::MessageMarker;

pub async fn send_oneshot(
	http: &DiscordHttpClient,
	actor: Actor,
	message: &str,
	channel_id: Id<ChannelMarker>,
	message_id: Option<Id<MessageMarker>>,
) -> Result {
	let content = oneshot_model(actor, message).await?;
	for chunk in chunk_message(&content, 2000) {
		let mut message = CreateMessage::new(channel_id).content(chunk);
		if let Some(message_id) = message_id {
			message = message.reply_to(message_id);
		}
		http.send(message).await?;
	}

	Ok(())
}

pub fn chunk_message(input: &str, max_len: usize) -> Vec<String> {
	let mut chunks = Vec::new();
	let mut current = String::new();

	for line in input.lines() {
		if current.len() + line.len() + 1 > max_len {
			chunks.push(current);
			current = String::new();
		}
		current.push_str(line);
		current.push('\n');
	}

	if !current.is_empty() {
		chunks.push(current);
	}

	chunks
}


pub async fn oneshot_model(actor: Actor, message: &str) -> Result<String> {
	let mut thread_view = ThreadMut::spawn();
	let out = thread_view
		.insert_actor(Actor::system())
		.insert_post(
			r#"
I do not have memory, my developer is too lazy to have made that yet.
So i never bother to ask follow up questions etc, no point.
			"#,
		)
		.thread_view()
		.insert_actor(actor)
		.insert_post(message)
		.thread_view()
		.insert_actor(Actor::agent())
		.with_bundle(
			OpenAiProvider::gpt_5_mini()?
				// OllamaProvider::qwen_3_8b()
				// disable streaming since we're aggregating
				.without_streaming(),
		)
		.send_and_collect()
		.await
		.unwrap()
		.into_iter()
		.filter(|post| post.intent().is_display())
		.xtry_map(|post| post.body_string())?
		.join("\n")
		.xok();

	thread_view.despawn();
	out
}
