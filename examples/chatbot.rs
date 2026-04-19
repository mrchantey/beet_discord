//! Discord bot entry point.
//!
//! All transport details live in `discord_io/gateway` (WebSocket) and
//! `discord_io/http` (REST). This file is purely bot logic: reacting to
//! typed events.
use beet::prelude::*;
use beet_discord::prelude::*;



fn main() {
	env_ext::load_dotenv();
	App::new()
		.add_plugins((
			MinimalPlugins,
			LogPlugin {
				// level: Level::TRACE,
				..default()
			},
			DiscordPlugin,
			InfraPlugin,
		))
		.add_systems(Startup, setup)
		.run();
}
#[cfg(not(feature = "chatbot_deploy"))]
fn setup(mut commands: Commands) {
	commands
		.spawn((DiscordBot::default(), assets_bucket()))
		.observe(init_bot_state)
		.observe(add_guild_create_channels)
		.observe(thread_sync::handle_message);
}

#[cfg(feature = "chatbot_deploy")]
fn setup(mut commands: Commands) {
	commands.spawn((stack(), stack_cli(), children![route(
		"deploy",
		(exchange_sequence(), children![
			(
				LightsailBlock::default().with_env_vars(vec![
					Variable::process_env("DISCORD_TOKEN"),
					Variable::process_env("OPENAI_API_KEY"),
				]),
				CargoBuild::default()
					.with_release(true)
					.with_target(BuildTarget::Zigbuild)
					.with_example("chatbot")
					.with_additional_args(vec![
						"--features".into(),
						"chatbot_aws".into(),
					])
					.into_build_artifact()
			),
			TofuApplyAction,
			(SyncS3Bucket::new(".beet"), assets_bucket_block()),
		]),
	)]));
}


#[allow(unused)]
fn stack() -> Stack {
	Stack::new("chatbot_example").with_aws_region("us-west-2")
}

#[allow(unused)]
fn assets_bucket_block() -> S3BucketBlock {
	S3BucketBlock::new("assets").with_deploy_versioned(true)
}

/// Resolve the assets bucket. Identical to the Lambda pattern:
/// on deployed instances, assets are accessed via S3 at runtime.
/// During local development, assets are read from the workspace.
#[allow(unused)]
fn assets_bucket() -> impl Component + BucketProvider {
	cfg_if! {
		if #[cfg(feature = "chatbot_aws")]{
			assets_bucket_block().provider(&stack())
		}else{
			FsBucket::new(WsPathBuf::new(".beet"))
		}
	}
}
