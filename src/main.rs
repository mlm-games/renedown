mod app;
mod markdown;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    repose_platform::run_desktop_app(|s, _rc| app::app(s))?;

    Ok(())
}
