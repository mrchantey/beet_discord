use crate::prelude::*;
use beet::prelude::*;
use tracing::error;
use tracing::info;
use tracing::warn;
use twilight_model::application::interaction::Interaction;
use twilight_model::application::interaction::InteractionData;
use twilight_model::application::interaction::InteractionType;
use twilight_model::application::interaction::application_command::CommandDataOption;
use twilight_model::application::interaction::application_command::CommandOptionValue;
use twilight_model::application::interaction::modal::ModalInteractionComponent;
use twilight_model::channel::message::MessageFlags;
use twilight_model::channel::message::component::SelectMenuOption;
use twilight_model::channel::message::embed::Embed;
use twilight_model::guild::Guild;
use twilight_model::http::interaction::InteractionResponse;
use twilight_model::http::interaction::InteractionResponseData;
use twilight_model::user::User;

/// Observer called when any interaction (slash command, component, modal) is received.
pub fn handle_interaction(
	ev: On<DiscordInteraction>,
	mut commands: Commands,
	query: Query<(&BotState, &DiscordHttpClient)>,
) -> Result {
	let entity = ev.event_target();
	let interaction = ev.interaction.clone();

	let (bot_state, http) = query.get(entity)?;
	let start_time = bot_state.start_time();
	let http = http.clone();

	commands.queue_async(async move |_| {
		if let Err(e) =
			dispatch_interaction(&http, &interaction, start_time).await
		{
			error!(error = %e, "failed to handle interaction");
		}
	});

	Ok(())
}

async fn dispatch_interaction(
	http: &DiscordHttpClient,
	interaction: &Interaction,
	start_time: std::time::Instant,
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
	match interaction.kind {
		InteractionType::ApplicationCommand => {
			handle_slash_command(http, interaction, start_time).await
		}
		InteractionType::MessageComponent => {
			handle_component(http, interaction).await
		}
		InteractionType::ModalSubmit => {
			handle_modal_submit(http, interaction).await
		}
		InteractionType::Ping => {
			let resp = InteractionResponse::pong();
			http.send(CreateInteractionResponse::new(
				interaction.id,
				interaction.token.clone(),
				resp,
			))
			.await?;
			Ok(())
		}
		_ => Ok(()),
	}
}

// ---------------------------------------------------------------------------
// Slash command handler
// ---------------------------------------------------------------------------

fn command_info(
	interaction: &Interaction,
) -> Option<(&str, &[CommandDataOption])> {
	match interaction.data.as_ref()? {
		InteractionData::ApplicationCommand(data) => {
			Some((&data.name, &data.options))
		}
		_ => None,
	}
}

fn get_option_u64(options: &[CommandDataOption], name: &str) -> Option<u64> {
	options
		.iter()
		.find(|o| o.name == name)
		.and_then(|o| match &o.value {
			CommandOptionValue::Integer(v) => Some(*v as u64),
			CommandOptionValue::Number(v) => Some(*v as u64),
			_ => None,
		})
}

async fn handle_slash_command(
	http: &DiscordHttpClient,
	interaction: &Interaction,
	start_time: std::time::Instant,
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
	let (name, options) =
		command_info(interaction).ok_or("missing interaction data")?;

	let response = match name {
		"ping" => text_response("🏓 Pong!"),

		"uptime" => {
			let elapsed = start_time.elapsed();
			let secs = elapsed.as_secs();
			text_response(format!(
				"⏱️ Bot uptime: {}h {}m {}s",
				secs / 3600,
				(secs % 3600) / 60,
				secs % 60
			))
		}

		"roll" => {
			let sides = get_option_u64(options, "sides").unwrap_or(6) as u32;
			let sides = sides.max(2).min(1000);
			let result = (rand::random::<u32>() % sides) + 1;
			let text = format!("🎲 Rolling a d{}... **{}**!", sides, result);

			InteractionResponse::message(
				InteractionResponseData::default()
					.with_content(text)
					.with_components(vec![action_row(vec![button(
						1,
						"🎲 Reroll",
						format!("reroll:{}", sides),
					)])]),
			)
		}

		"serverinfo" => {
			let text = if let Some(guild_id) = interaction.guild_id {
				match http.send(GetGuild::new(guild_id)).await {
					Ok(guild) => format_guild_info(&guild),
					Err(e) => format!("❌ Error: {}", e),
				}
			} else {
				"❌ This command only works in a server.".to_string()
			};
			text_response(text)
		}

		"whoami" => {
			let text = match interaction.author() {
				Some(user) => format_whoami(user),
				None => "❌ Couldn't determine your user info.".to_string(),
			};
			text_response(text)
		}

		"count" => {
			#[allow(deprecated)]
			let text = if let Some(ch_id) = interaction.channel_id {
				match http.count_messages(ch_id).await {
					Ok(count) => {
						format!(
							"📊 This channel has approximately **{}** messages.",
							count
						)
					}
					Err(e) => format!("❌ Error: {}", e),
				}
			} else {
				"❌ No channel context.".to_string()
			};
			text_response(text)
		}

		"first" => {
			#[allow(deprecated)]
			let text = if let Some(ch_id) = interaction.channel_id {
				match http.get_first_message(ch_id).await {
					Ok(first_msg) => {
						let ts_str = first_msg.timestamp.iso_8601().to_string();
						let ts = if let Ok(dt) =
							chrono::DateTime::parse_from_rfc3339(&ts_str)
						{
							dt.format("%B %d, %Y at %H:%M UTC").to_string()
						} else {
							ts_str
						};
						format!(
							"📜 **First message in this channel:**\n> {}\n— *{}* on {}",
							first_msg.content, first_msg.author.name, ts
						)
					}
					Err(e) => format!("❌ Error: {}", e),
				}
			} else {
				"❌ No channel context.".to_string()
			};
			text_response(text)
		}

		"help" => text_response(help_text()),

		"report" => InteractionResponse::modal(
			InteractionResponseData::default()
				.with_title("📝 Submit a Report")
				.with_custom_id("report_modal")
				.with_components(vec![
					action_row(vec![text_input(
						"report_subject",
						"Subject",
						1,
						true,
					)]),
					action_row(vec![text_input(
						"report_body",
						"Description",
						2,
						true,
					)]),
				]),
		),

		"send-logo" => {
			let ack = InteractionResponse::defer();
			http.send(CreateInteractionResponse::new(
				interaction.id,
				interaction.token.clone(),
				ack,
			))
			.await?;

			#[allow(deprecated)]
			if let Some(ch_id) = interaction.channel_id {
				match std::fs::read("./logo-square.png") {
					Ok(file_content) => {
						if let Err(e) = http
							.send(
								CreateMessageWithFile::new(
									ch_id,
									"logo-square.png",
									file_content,
								)
								.content("Here's our logo! 🎨"),
							)
							.await
						{
							warn!(error = %e, "failed to send logo file");
							let _ = http
								.send(CreateMessage::new(ch_id).content(
									format!("❌ Failed to send logo: {}", e),
								))
								.await;
						}
					}
					Err(e) => {
						warn!(error = %e, "failed to read logo file");
						let _ =
							http.send(CreateMessage::new(ch_id).content(
								format!("❌ Failed to read logo file: {}", e),
							))
							.await;
					}
				}
			}
			return Ok(());
		}

		"demo-select" => InteractionResponse::message(
			InteractionResponseData::default()
				.with_content(
					"Please select your favorite programming language:",
				)
				.with_components(vec![action_row(vec![string_select(
					"language_select",
					"Choose a language...",
					vec![
						SelectMenuOption {
							default: false,
							description: Some(
								"Fast, safe, and concurrent".to_string(),
							),
							emoji: None,
							label: "Rust".to_string(),
							value: "rust".to_string(),
						},
						SelectMenuOption {
							default: false,
							description: Some(
								"Simple and versatile".to_string(),
							),
							emoji: None,
							label: "Python".to_string(),
							value: "python".to_string(),
						},
						SelectMenuOption {
							default: false,
							description: Some("Typed JavaScript".to_string()),
							emoji: None,
							label: "TypeScript".to_string(),
							value: "typescript".to_string(),
						},
						SelectMenuOption {
							default: false,
							description: Some(
								"Simple and efficient".to_string(),
							),
							emoji: None,
							label: "Go".to_string(),
							value: "go".to_string(),
						},
					],
				)])]),
		),

		_ => {
			info!(command = name, "unknown slash command");
			text_response(format!("Unknown command: `/{}`", name))
		}
	};

	http.send(CreateInteractionResponse::new(
		interaction.id,
		interaction.token.clone(),
		response,
	))
	.await?;
	Ok(())
}

// ---------------------------------------------------------------------------
// Component interaction handler
// ---------------------------------------------------------------------------

fn component_info(interaction: &Interaction) -> Option<(&str, &[String])> {
	match interaction.data.as_ref()? {
		InteractionData::MessageComponent(data) => {
			Some((&data.custom_id, &data.values))
		}
		_ => None,
	}
}

async fn handle_component(
	http: &DiscordHttpClient,
	interaction: &Interaction,
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
	let (custom_id, values) =
		component_info(interaction).ok_or("missing interaction data")?;

	if custom_id.starts_with("reroll:") {
		let sides: u32 = custom_id
			.strip_prefix("reroll:")
			.and_then(|s| s.parse().ok())
			.unwrap_or(6)
			.max(2)
			.min(1000);

		let result = (rand::random::<u32>() % sides) + 1;
		let text = format!("🎲 Rolling a d{}... **{}**!", sides, result);

		let response = InteractionResponse::update(
			InteractionResponseData::default()
				.with_content(text)
				.with_components(vec![action_row(vec![button(
					1,
					"🎲 Reroll",
					format!("reroll:{}", sides),
				)])]),
		);

		http.send(CreateInteractionResponse::new(
			interaction.id,
			interaction.token.clone(),
			response,
		))
		.await?;
	} else if !values.is_empty() {
		let selected = values.join(", ");
		let text = format!("You selected: **{}**", selected);
		let response = InteractionResponse::message(
			InteractionResponseData::default()
				.with_content(text)
				.with_flags(MessageFlags::EPHEMERAL),
		);
		http.send(CreateInteractionResponse::new(
			interaction.id,
			interaction.token.clone(),
			response,
		))
		.await?;
	} else {
		info!(custom_id, "unhandled component interaction");
	}

	Ok(())
}

// ---------------------------------------------------------------------------
// Modal submit handler
// ---------------------------------------------------------------------------

fn modal_text_inputs(
	interaction: &Interaction,
) -> Option<(String, Vec<(String, String)>)> {
	match interaction.data.as_ref()? {
		InteractionData::ModalSubmit(data) => {
			let custom_id = data.custom_id.clone();
			let mut inputs = Vec::new();
			for row in &data.components {
				if let ModalInteractionComponent::ActionRow(action_row) = row {
					for component in &action_row.components {
						if let ModalInteractionComponent::TextInput(ti) =
							component
						{
							inputs
								.push((ti.custom_id.clone(), ti.value.clone()));
						}
					}
				}
			}
			Some((custom_id, inputs))
		}
		_ => None,
	}
}

async fn handle_modal_submit(
	http: &DiscordHttpClient,
	interaction: &Interaction,
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
	let (custom_id, text_inputs) =
		modal_text_inputs(interaction).ok_or("missing interaction data")?;

	if custom_id == "report_modal" {
		let mut subject = String::new();
		let mut body = String::new();

		for (id, value) in &text_inputs {
			match id.as_str() {
				"report_subject" => subject = value.clone(),
				"report_body" => body = value.clone(),
				_ => {}
			}
		}

		let author_name = interaction
			.author()
			.map(|u| u.tag())
			.unwrap_or_else(|| "Unknown".to_string());

		let embed = Embed::new()
			.with_title(format!("📝 Report: {}", subject))
			.with_description(&body)
			.with_color(0xFF6600)
			.with_footer(format!("Submitted by {}", author_name))
			.with_timestamp(chrono::Utc::now().to_rfc3339());

		let response = InteractionResponse::message(
			InteractionResponseData::default()
				.with_content("✅ Report submitted! Thank you.")
				.with_embeds(vec![embed]),
		);

		http.send(CreateInteractionResponse::new(
			interaction.id,
			interaction.token.clone(),
			response,
		))
		.await?;
	}

	Ok(())
}

// ---------------------------------------------------------------------------
// Response helpers
// ---------------------------------------------------------------------------

fn text_response(text: impl Into<String>) -> InteractionResponse {
	InteractionResponse::text(text)
}

// ---------------------------------------------------------------------------
// Formatting helpers
// ---------------------------------------------------------------------------

fn format_guild_info(guild: &Guild) -> String {
	let member_count = guild
		.approximate_member_count
		.map(|n| n.to_string())
		.unwrap_or_else(|| "unknown".to_string());
	let online_count = guild
		.approximate_presence_count
		.map(|n| n.to_string())
		.unwrap_or_else(|| "unknown".to_string());
	let owner_str = guild.owner_id.to_string();
	let created_at = guild
		.created_at_ms()
		.and_then(|ms| chrono::DateTime::from_timestamp_millis(ms as i64))
		.map(|dt| dt.format("%B %d, %Y").to_string())
		.unwrap_or_else(|| "unknown".to_string());

	format!(
		"🏰 **Server Info: {}**\n\
         • **Members:** {} ({} online)\n\
         • **Owner:** <@{}>\n\
         • **Created:** {}",
		guild.name, member_count, online_count, owner_str, created_at
	)
}

fn format_whoami(user: &User) -> String {
	let avatar_url = user
		.avatar_url()
		.unwrap_or_else(|| "No avatar set".to_string());
	format!(
		"👤 **About You:**\n\
         • **Username:** {}\n\
         • **User ID:** {}\n\
         • **Avatar:** {}",
		user.tag(),
		user.id,
		avatar_url
	)
}

fn help_text() -> String {
	"🤖 **Available Commands:**\n\
     *Prefix commands (! or @mention):*\n\
     • `!hello` — Say hello!\n\
     • `!ping` — Check bot latency\n\
     • `!uptime` — See how long the bot has been running\n\
     • `!roll [sides]` — Roll a dice (default: 6 sides)\n\
     • `!count` — Count messages in this channel\n\
     • `!first` — Show the first message ever sent in this channel\n\
     • `!serverinfo` — Show server information\n\
     • `!whoami` — Show info about yourself\n\
     • `!help` — Show this help message\n\
     \n\
     *Slash commands:*\n\
     • `/ping` `/uptime` `/roll` `/serverinfo` `/whoami` `/count` `/first` `/help`\n\
     • `/report` — Submit a report via a pop-up form\n\
     • `/send-logo` — Send the bot logo\n\
     • `/demo-select` — Demo the select menu component"
		.to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
	use super::*;
	use twilight_model::http::interaction::InteractionResponseType;

	// -- text_response() ---------------------------------------------------

	#[test]
	fn text_response_sets_content() {
		let resp = text_response("hello");
		assert_eq!(
			resp.data.as_ref().unwrap().content.as_deref(),
			Some("hello")
		);
	}

	#[test]
	fn text_response_uses_channel_message_type() {
		let resp = text_response("x");
		assert!(matches!(
			resp.kind,
			InteractionResponseType::ChannelMessageWithSource
		));
	}

	#[test]
	fn text_response_has_no_extras() {
		let resp = text_response("x");
		let data = resp.data.unwrap();
		assert!(data.embeds.is_none());
		assert!(data.components.is_none());
		assert!(data.flags.is_none());
	}

	// -- format_guild_info() -----------------------------------------------

	#[test]
	fn format_guild_info_includes_guild_name() {
		let guild: Guild = serde_json::from_value(serde_json::json!({
			"id": "123",
			"name": "Test Server",
			"icon": null,
			"owner_id": "456",
			"approximate_member_count": 42,
			"approximate_presence_count": 10,
			"channels": [],
			"members": [],
			"roles": [],
			"emojis": [],
			"features": [],
			"afk_timeout": 300,
			"preferred_locale": "en-US",
			"premium_progress_bar_enabled": false,
			"verification_level": 0,
			"default_message_notifications": 0,
			"explicit_content_filter": 0,
			"mfa_level": 0,
			"premium_tier": 0,
			"nsfw_level": 0,
			"system_channel_flags": 0,
		}))
		.expect("valid guild JSON");
		let text = format_guild_info(&guild);
		assert!(text.contains("Test Server"), "missing guild name");
		assert!(text.contains("42"), "missing member count");
		assert!(text.contains("10"), "missing online count");
		assert!(text.contains("<@456>"), "missing owner mention");
	}

	#[test]
	fn format_guild_info_handles_missing_counts() {
		let guild: Guild = serde_json::from_value(serde_json::json!({
			"id": "1",
			"name": "Empty",
			"icon": null,
			"owner_id": "1",
			"channels": [],
			"members": [],
			"roles": [],
			"emojis": [],
			"features": [],
			"afk_timeout": 300,
			"preferred_locale": "en-US",
			"premium_progress_bar_enabled": false,
			"verification_level": 0,
			"default_message_notifications": 0,
			"explicit_content_filter": 0,
			"mfa_level": 0,
			"premium_tier": 0,
			"nsfw_level": 0,
			"system_channel_flags": 0,
		}))
		.expect("valid guild JSON");
		let text = format_guild_info(&guild);
		assert!(
			text.contains("unknown"),
			"missing 'unknown' for absent counts"
		);
	}

	// -- format_whoami() ---------------------------------------------------

	#[test]
	fn format_whoami_includes_username_and_id() {
		let user: User = serde_json::from_value(serde_json::json!({
			"id": "789",
			"username": "alice",
			"discriminator": "0001",
			"avatar": null,
			"bot": false,
			"global_name": null,
		}))
		.expect("valid user JSON");
		let text = format_whoami(&user);
		assert!(text.contains("alice"), "missing username");
		assert!(text.contains("789"), "missing user id");
	}

	// -- help_text() -------------------------------------------------------

	#[test]
	fn help_text_mentions_all_prefix_commands() {
		let text = help_text();
		for cmd in &[
			"!hello",
			"!ping",
			"!uptime",
			"!roll",
			"!count",
			"!first",
			"!serverinfo",
			"!whoami",
			"!help",
		] {
			assert!(text.contains(cmd), "help text missing {}", cmd);
		}
	}

	// -- get_option_u64() --------------------------------------------------

	#[test]
	fn get_option_u64_finds_integer_option() {
		use twilight_model::application::interaction::application_command::CommandDataOption;
		use twilight_model::application::interaction::application_command::CommandOptionValue;
		let options = vec![CommandDataOption {
			name: "sides".to_string(),
			value: CommandOptionValue::Integer(20),
		}];
		assert_eq!(get_option_u64(&options, "sides"), Some(20));
	}

	#[test]
	fn get_option_u64_returns_none_for_missing() {
		use twilight_model::application::interaction::application_command::CommandDataOption;
		use twilight_model::application::interaction::application_command::CommandOptionValue;
		let options = vec![CommandDataOption {
			name: "other".to_string(),
			value: CommandOptionValue::Integer(5),
		}];
		assert_eq!(get_option_u64(&options, "sides"), None);
	}

	// -- reroll sides clamping ---------------------------------------------

	#[test]
	fn reroll_sides_clamped_correctly() {
		let raw: u32 = 1u32;
		let clamped = raw.max(2).min(1000);
		assert_eq!(clamped, 2);

		let raw: u32 = 5000u32;
		let clamped = raw.max(2).min(1000);
		assert_eq!(clamped, 1000);
	}
}
