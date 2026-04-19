mod handle_interaction;
mod parse_bang_command;
mod register_commands;
use crate::prelude::*;
use beet::prelude::*;
use handle_interaction::*;
use parse_bang_command::*;
use register_commands::*;
mod greet_state;
use greet_state::*;


/// Startup system that spawns the discord bot.
pub fn spawn_command_demo(mut commands: Commands) {
	commands
		.spawn((DiscordBot::default(), GreetState::default()))
		.observe(init_bot_state)
		.observe(register_commands)
		.observe(parse_bang_command)
		.observe(handle_interaction);
}
