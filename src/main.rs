mod app;
mod args;
mod auth;
mod cli;
mod error;
mod model;
mod output;
mod state;
mod task;
mod update;
mod upstream;
mod usage;

fn main() {
    std::process::exit(app::run());
}
