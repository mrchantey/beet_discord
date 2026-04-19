use crate::prelude::Message;
use crate::prelude::*;
use beet::prelude::*;
use std::sync::Arc;

use super::discord_meta::DiscordActorMeta;
use super::discord_meta::DiscordMeta;
use super::discord_meta::DiscordPostMeta;
use super::discord_meta::DiscordStatus;
use super::discord_meta::DiscordThreadMeta;


pub fn handle_message(ev: On<DiscordMessage>, mut commands: Commands) {
	let message = ev.message.clone();
	trace!("message received: {message:#?}");
	commands.entity(ev.event_target()).queue_async(
		async move |entity| -> Result {
			// Skip messages authored by the bot itself
			let author_id = message.author.id;
			let is_own_message = entity
				.get::<BotState, _>(move |bot| bot.user_id() == author_id)
				.await?;
			if is_own_message {
				trace!("ignoring bot's own message");
				return Ok(());
			}

			info!("getting channel");
			let channel_id = get_channel(entity, message.clone()).await?;
			info!("getting thread");
			let thread = get_thread(entity, channel_id).await?;
			info!("inserting post");
			insert_post(entity.world().entity(thread), message.clone()).await?;
			info!("triggering response");
			trigger_response(entity, thread, channel_id, message).await?;
			Ok(())
		},
	);
}

async fn get_channel(
	entity: AsyncEntity,
	message: Arc<Message>,
) -> Result<Id<ChannelMarker>> {
	let message2 = message.clone();
	let should_create_thread = entity
		.with_state::<DiscordQuery, _>(move |entity, query| {
			query.should_create_thread(entity, &message2)
		})
		.await?;

	if !should_create_thread {
		return Ok(message.channel_id);
	}
	let thread = CreateThreadFromMessage::new(
		message.channel_id,
		message.id,
		// TODO llm assigned name
		"New Thread",
	)
	.auto_archive_duration(60);
	let channel = entity
		.get_cloned::<DiscordHttpClient>()
		.await?
		.send(thread)
		.await?;
	let id = channel.id;
	ChannelMap::try_add_async(entity, channel).await?;
	Ok(id)
}


async fn get_thread(
	entity: AsyncEntity,
	channel_id: Id<ChannelMarker>,
) -> Result<Entity> {
	if let Some(entity) = entity
		.world()
		.with_state::<Query<(Entity, &Thread)>, _>(move |query| {
			query
				.iter()
				.find(|(_, thread)| {
					DiscordThreadMeta::read_from(thread.metadata())
						.discord_channel == Some(channel_id.get())
				})
				.map(|(entity, _)| entity)
		})
		.await
	{
		info!("Found thread in memory");
		return Ok(entity);
	}
	let store_entity = entity
		.with_then(move |mut entity| -> Result<Entity> {
			let path = RelPath::new("channels")
				.join(channel_id.to_string())
				.with_extension("json");
			let bucket = entity.get_or_else::<Bucket>()?.clone();
			let blob = bucket.blob(path);
			let bot_id = entity.id();
			entity
				.world_scope(|world| {
					world
						.spawn((
							ChildOf(bot_id),
							bucket,
							blob.clone(),
							SceneStore::default(),
						))
						.id()
				})
				.xok()
		})
		.await?;
	let channel_name = entity
		.get::<ChannelMap, _>(move |map| {
			map.get(channel_id).map(|info| {
				info.channel()
					.name
					.clone()
					.unwrap_or_else(|| String::from("New Thread"))
			})
		})
		.await??;

	let spawned = SceneStore::load_or_create(
		entity.world().entity(store_entity),
		async move |entity| {
			info!("Creating New Thread");
			let mut thread = Thread::new(channel_name);
			let thread_meta = DiscordThreadMeta {
				discord_channel: Some(channel_id.get()),
			};

			let bucket = entity.get_cloned::<Bucket>().await?;
			let system_prompt = bucket
				.blob(RelPath::new("agent/system_prompt.md"))
				.get_media()
				.await?
				.to_string();


			thread_meta.write_to(thread.metadata_mut());
			(thread, children![(Actor::system(), children![
				Post::spawn(system_prompt),
				// add more system prompts here as required
			])])
				.xok()
		},
	)
	.await?;
	info!("Thread loaded");
	let thread_entity = entity
		.world()
		.with_state::<Query<(), With<Thread>>, Result<Entity>>(move |query| {
			spawned
				.into_iter()
				.find(|entity| query.contains(*entity))
				.ok_or_else(|| bevyhow!("Loaded scene has no thread"))
		})
		.await?;
	Ok(thread_entity)
}


async fn insert_post(thread: AsyncEntity, message: Arc<Message>) -> Result {
	thread
		.with_state::<(Commands, ThreadQuery), _>(
			move |entity, (mut commands, query)| {
				let author_id = message.author.id.get();
				let author_name = &message.author.name;
				let content = &message.content;
				// TODO reset thread keepalive
				let thread = query.thread(entity)?;


				let (actor_id, actor_entity) = if let Some(actor) =
					thread.actors().iter().find(|actor| {
						let meta =
							DiscordActorMeta::read_from(actor.metadata());
						meta.discord_user == Some(author_id)
					}) {
					info!("Actor Found: {}", author_name);
					(actor.id(), actor.entity)
				} else {
					let actor_kind = if message.author.bot {
						ActorKind::Agent
					} else {
						ActorKind::User
					};
					info!("Creating Actor: {}", author_name);
					let mut actor = Actor::new(author_name, actor_kind);
					let actor_meta = DiscordActorMeta {
						discord_user: Some(author_id),
					};
					actor_meta.write_to(actor.metadata_mut());
					let actor_id = actor.id();
					let actor_entity =
						commands.spawn((ChildOf(thread.entity), actor)).id();
					(actor_id, actor_entity)
				};
				let mut post = AgentPost::new_text(
					actor_id,
					thread.id(),
					content,
					PostStatus::Completed,
				);

				// Mark incoming Discord posts as
				// already delivered so `send_posts` never tries to re-send them.
				let post_meta = DiscordPostMeta {
					discord_status: Some(DiscordStatus::Complete),
					discord_messages: None,
				};
				post_meta.write_to(post.metadata_mut());

				commands.spawn((post, ChildOf(actor_entity)));

				Ok(())
			},
		)
		.await
}


async fn trigger_response(
	bot: AsyncEntity,
	thread: Entity,
	channel_id: Id<ChannelMarker>,
	message: Arc<Message>,
) -> Result {
	bot.with_state::<(
		Commands,
		DiscordQuery,
		ThreadQuery,
		Query<&BotState>,
	), Result<_>>(
		move |entity,
		      (mut commands, discord_query, thread_query, bot_query)| {
			if !discord_query.should_respond(entity, &message)? {
				return Ok(());
			}
			let thread = thread_query.thread(thread)?;
			let bot = bot_query.get(entity)?;
			let bot_user_id = bot.user_id().get();
			let actor = if let Some(actor) =
				thread.actors().iter().find(|actor| {
					let meta = DiscordActorMeta::read_from(actor.metadata());
					meta.discord_user == Some(bot_user_id)
				}) {
				info!("Bot actor found");
				actor.entity
			} else {
				info!("Creating bot actor");
				let actor = bot.create_actor();
				commands
					.spawn((
						ChildOf(thread.entity),
						actor,
						OpenAiProvider::gpt_5_mini()?,
						// OllamaProvider::qwen_3_8b()
						// disable streaming since we're aggregating
						// .without_streaming(),
					))
					.id()
			};
			// just trigger the actor, posts are propagated via system
			info!("Triggering bot call");

			commands.entity(entity).queue_async(
				async move |entity| -> Result {
					entity
						.get_cloned::<DiscordHttpClient>()
						.await?
						.send(CreateTypingTrigger::new(channel_id))
						.await?;
					Ok(())
				},
			);
			commands.entity(actor).call::<(), Outcome>((), default());

			Ok(())
		},
	)
	.await
}


pub fn send_posts(
	mut commands: Commands,
	bots: AncestorQuery<(&BotState, &DiscordHttpClient)>,
	scene_stores: AncestorQuery<&SceneOf>,
	posts: Query<(Entity, &Post), Changed<Post>>,
	threads: ThreadQuery,
) -> Result {
	for (entity, post) in posts.iter() {
		if post.in_progress() {
			continue;
		}

		let post_meta = DiscordPostMeta::read_from(post.metadata());
		match post_meta.discord_status {
			Some(DiscordStatus::Complete) | Some(DiscordStatus::Sending) => {
				continue;
			}
			_ => {}
		};

		// Skip posts with empty body – avoids sending a blank message
		// during the brief window before the LLM fills in the content.
		if post.body_str().map_or(true, |s| s.is_empty()) {
			continue;
		}

		let thread = threads.thread(entity)?;
		let actor = thread.actor_from_id(post.author())?;

		let scene = scene_stores.get(entity)?.0;
		let (bot, http) = bots.get(scene)?;
		let bot_user_id = bot.user_id().get();

		let actor_meta = DiscordActorMeta::read_from(actor.metadata());
		if actor_meta.discord_user != Some(bot_user_id) {
			info!("ignoring post not sent by bot");
			// only send messages authored by the bot itself
			continue;
		}

		let thread_meta = DiscordThreadMeta::read_from(thread.metadata());
		let Some(channel) = thread_meta.discord_channel else {
			warn!("disregarding post with no discord channel: {post:?}");
			continue;
		};
		let channel_id = Id::<ChannelMarker>::new(channel);
		let http = http.clone();

		let chunks = thread_utils::chunk_message(post.body_str()?, 2000);


		commands.entity(entity).queue(|mut entity: EntityWorldMut| {
			let Some(mut post) = entity.get_mut::<Post>() else {
				return;
			};
			let meta = DiscordPostMeta {
				discord_status: Some(DiscordStatus::Sending),
				discord_messages: None,
			};
			meta.write_to(post.metadata_mut());
		});

		commands
			.entity(entity)
			.queue_async(async move |entity| -> Result {
				let mut messages = Vec::new();
				info!("sending {} message chunks", chunks.len());
				for chunk in chunks {
					let message = CreateMessage::new(channel_id).content(chunk);
					let message = http.send(message).await?;
					messages.push(message);
				}
				entity
					.get_mut::<Post, _>(move |mut post| {
						let meta = DiscordPostMeta {
							discord_status: Some(DiscordStatus::Complete),
							discord_messages: Some(
								messages.iter().map(|m| m.id.get()).collect(),
							),
						};
						meta.write_to(post.metadata_mut());
					})
					.await?;
				info!("all message chunks sent");
				Ok(())
			});
	}
	Ok(())
}
