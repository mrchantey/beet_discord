//! Implementation for request types, mapping to the twilight client capabilities:
//! ## Summary
//!
//! Below is a summary of the implementation parity with twilight
//!
//! | Status | Count |
//! |--------|-------|
//! | ✅ Implemented | 145 |
//! | ❌ Missing | 35 |
//! | **Total** | **180** |
//!
//! ### Remaining ❌ items (low priority / complex body types)
//!
//! - Guild channel positions, vanity URL, voice regions (#33, #35, #36)
//! - Add guild member via OAuth (#41)
//! - Single role GET, role positions, role member counts (#48, #52, #53)
//! - Voice states (#71–74)
//! - Update thread, archived thread listings (#95–98)
//! - Create guild sticker (multipart upload) (#118)
//! - Create/update auto-mod rules (complex body) (#123, #125)
//! - Create/update/create-from template (#128, #131, #132)
//! - Guild widget, welcome screen update, onboarding update, MFA (#138–140, #142–143, #145)
//! - Gateway URL (#147)
//! - Entitlements & SKUs (#148–151)
//! - Application emojis (#154–157)
//! - Webhook with token variant (#80)
//! - OAuth current authorization / application (#67–69)
//! - Create/update individual commands (#166, #170, #172, #176)
//! - Update command permissions (#180)


/// Concrete request types implementing [`IntoDiscordRequest`].
mod requests;
pub use requests::*;
/// Additional channel, message, and reaction request types.
mod requests_channels;
pub use requests_channels::*;
/// Interaction response, followup, and application command request types.
mod requests_interactions;
pub use requests_interactions::*;
/// Guild, member, role, and ban request types.
mod requests_guilds;
pub use requests_guilds::*;
/// Webhook, thread, invite, emoji, sticker, stage, scheduled event, template,
/// auto-moderation, voice, poll, and guild welcome/onboarding request types.
mod requests_misc;
pub use requests_misc::*;
