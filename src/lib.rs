mod app;
mod latex;
mod markdown;

mod file_picker;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

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

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn wasm_start() -> Result<(), JsValue> {
    register_fonts();
    repose_platform::web::run_web_app(
        |s, _rc| app::app(s, None),
        repose_platform::web::WebOptions::new(None),
    )
}

#[cfg(target_os = "android")]
use log::LevelFilter;
#[cfg(target_os = "android")]
use repose_platform::android::run_android_app;
#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "C" fn android_main(android_app: AndroidApp) {
    android_logger::init_once(android_logger::Config::default().with_max_level(LevelFilter::Info));

    register_fonts();
    rlobkit_dialogs::init();
    rlobkit_dialogs::init_shared_pending_state();

    // Forward window insets from RlobKitMainActivity into repose's layout system.
    rlobkit_app_events::insets::set_on_insets(Box::new(|insets| {
        let r = repose_core::locals::WindowInsets {
            top: insets.top,
            bottom: insets.bottom,
            left: insets.left,
            right: insets.right,
            ime_bottom: insets.ime_bottom,
        };
        repose_core::locals::set_window_insets_default(r);
    }));

    // Read initial content from a VIEW intent (content:// URI resolved by
    // RlobKitIntentBridge.captureViewIntent → pending_intent file).
    let initial = android_app
        .internal_data_path()
        .as_deref()
        .and_then(|dir| rlobkit_app_events::take_pending_intent(dir))
        .and_then(|intent| String::from_utf8(intent.data).ok())
        .filter(|s| !s.is_empty());

    if let Err(err) = run_android_app(android_app, move |s, _rc| app::app(s, initial.clone())) {
        log::error!("Renedown failed: {err:?}");
    }
}
