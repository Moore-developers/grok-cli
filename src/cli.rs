use crate::app::AppContext;
use crate::args::{Cli, TopLevelCommand};
use crate::error::CommandError;
use crate::{auth, model, state, task, usage};

pub type CommandResult = Result<(), CommandError>;

pub fn dispatch(ctx: &AppContext, cli: Cli) -> CommandResult {
    match cli.command {
        TopLevelCommand::Login(opts) => auth::runtime::login(ctx, opts),
        TopLevelCommand::Status(opts) => auth::auth_status(ctx, opts),
        TopLevelCommand::Refresh(opts) => auth::refresh::refresh(ctx, opts),
        TopLevelCommand::Logout(opts) => auth::logout(ctx, opts),
        TopLevelCommand::ExchangeCode(opts) => auth::login::exchange_code(ctx, opts),
        TopLevelCommand::State(opts) => state::show(ctx, opts),
        TopLevelCommand::Model(cmd) => model::execute(ctx, cmd),
        TopLevelCommand::Usage(opts) => usage::command::execute(ctx, opts),
        TopLevelCommand::Chat(opts) => task::chat::execute(ctx, opts),
        TopLevelCommand::Search(opts) => task::search::execute(ctx, opts),
        TopLevelCommand::Image(opts) => task::image::execute(ctx, opts),
        TopLevelCommand::Video(opts) => task::video::execute(ctx, opts),
        TopLevelCommand::Tts(opts) => task::audio::execute_tts(ctx, opts),
        TopLevelCommand::Stt(opts) => task::audio::execute_stt(ctx, opts),
        TopLevelCommand::SttStream(opts) => task::audio::execute_stt_stream(ctx, opts),
    }
}
