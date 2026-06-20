mod app;
mod markdown;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    repose_platform::run_desktop_app(|s, _rc| app::app(s))
}
