use game_2::game_2::Game2;
use muleengine::application_runner;

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    application_runner::run(true, Game2::new);
}
