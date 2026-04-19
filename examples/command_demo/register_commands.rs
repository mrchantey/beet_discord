use crate::prelude::CommandExt;
use crate::prelude::*;
use beet::prelude::*;
use tracing::warn;
use twilight_model::application::command::Command;
/// Called when the bot receives the READY event from the gateway.
///
/// Stores identity information in [`BotState`] and registers slash commands
/// globally (once per session).
pub fn register_commands(
	ev: On<DiscordReady>,
	mut commands: Commands,
	query: Populated<&DiscordHttpClient, Without<BotState>>,
) -> Result {
	let entity = ev.event_target();

	let client = query.get(entity)?.clone();
	let app_id = ev.application.id;
	commands.queue_async(async move |_| {
		let cmds = slash_commands();
		match client.send(SetGlobalCommands::new(app_id, cmds)).await {
			Ok(registered) => {
				info!(
					count = registered.len(),
					"registered global slash commands"
				);
			}
			Err(e) => {
				warn!(error = %e, "failed to register global commands");
			}
		}
	});

	Ok(())
}

// ---------------------------------------------------------------------------
// Slash command definitions
// ---------------------------------------------------------------------------

/// Returns the list of slash commands to register with Discord.
fn slash_commands() -> Vec<Command> {
	use twilight_model::application::command::CommandOptionType;

	vec![
		Command::chat_input("ping", "Check bot latency"),
		Command::chat_input("uptime", "See how long the bot has been running"),
		Command::chat_input("roll", "Roll a dice").with_simple_option(
			CommandOptionType::Integer,
			"sides",
			"Number of sides (default: 6)",
			false,
		),
		Command::chat_input("serverinfo", "Show server information"),
		Command::chat_input("whoami", "Show info about yourself"),
		Command::chat_input("count", "Count messages in this channel"),
		Command::chat_input(
			"first",
			"Show the first message ever sent in this channel",
		),
		Command::chat_input("help", "Show available commands"),
		Command::chat_input("report", "Submit a report via a pop-up form"),
		Command::chat_input("send-logo", "Send the bot logo"),
		Command::chat_input("demo-select", "Demo the select menu component"),
	]
}


#[cfg(test)]
mod tests {
	use super::*;
	use twilight_model::application::command::CommandOptionType;

	#[test]
	fn slash_commands_returns_expected_count() {
		let cmds = slash_commands();
		assert_eq!(cmds.len(), 11);
	}

	#[test]
	fn slash_commands_names_are_unique() {
		let cmds = slash_commands();
		let mut names: Vec<&str> =
			cmds.iter().map(|c| c.name.as_str()).collect();
		names.sort();
		names.dedup();
		assert_eq!(names.len(), cmds.len(), "duplicate command names found");
	}

	#[test]
	fn slash_commands_all_have_descriptions() {
		for cmd in slash_commands() {
			assert!(
				!cmd.description.is_empty(),
				"command '{}' has empty description",
				cmd.name
			);
		}
	}

	#[test]
	fn roll_command_has_sides_option() {
		let cmds = slash_commands();
		let roll = cmds.iter().find(|c| c.name == "roll").expect("no /roll");
		assert_eq!(roll.options.len(), 1);
		assert_eq!(roll.options[0].name, "sides");
		assert!(matches!(roll.options[0].kind, CommandOptionType::Integer));
		assert_eq!(roll.options[0].required, Some(false));
	}
}
