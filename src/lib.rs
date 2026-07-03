mod app;
mod markdown;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn wasm_start() -> Result<(), JsValue> {
    repose_platform::web::run_web_app(
        |s, _rc| app::app(s),
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

    if let Err(err) = run_android_app(android_app, |s, _rc| app::app(s)) {
        log::error!("Renedown failed: {err:?}");
    }
}
