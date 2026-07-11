mod app;
mod latex;
mod markdown;

mod file_picker;

fn register_fonts() {
    repose_text::register_font_data(include_bytes!("../assets/fonts/OpenSans-Regular.ttf"));
    repose_text::register_font_data(include_bytes!("../assets/fonts/OpenSans-Italic.ttf"));
    repose_text::register_font_data(include_bytes!("../assets/fonts/OpenSans-Bold.ttf"));
    repose_text::register_font_data(include_bytes!("../assets/fonts/OpenSans-BoldItalic.ttf"));
    repose_text::register_font_data(include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf"));
    repose_text::register_font_data(include_bytes!("../assets/fonts/JetBrainsMono-Italic.ttf"));
    repose_text::register_font_data(include_bytes!("../assets/fonts/JetBrainsMono-Bold.ttf"));
    repose_text::register_font_data(include_bytes!("../assets/fonts/JetBrainsMono-BoldItalic.ttf"));
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    register_fonts();
    rlobkit_dialogs::init();

    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    repose_platform::run_desktop_app(|s, _rc| app::app(s, None))?;

    Ok(())
}
