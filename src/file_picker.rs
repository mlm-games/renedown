use rlobkit_dialogs::picker::{OpenFileOptions, SaveFileOptions};
use rlobkit_dialogs::{RlobKit, RlobKitType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveOutcome {
    Saved,
    Cancelled,
}

pub fn spawn_open_file(
    title: &str,
    extensions: &[&str],
) -> flume::Receiver<Result<Option<String>, String>> {
    let (tx, rx) = flume::unbounded();
    let title = title.to_string();
    let exts: Vec<String> = extensions.iter().map(|s| s.to_string()).collect();

    #[cfg(not(target_arch = "wasm32"))]
    std::thread::spawn(move || {
        let result = (|| -> Result<Option<String>, String> {
            let result = futures_lite::future::block_on(RlobKit::open_file_picker(
                OpenFileOptions {
                    file_type: RlobKitType::Custom {
                        extensions: exts,
                        mime_types: vec!["text/markdown".to_string()],
                    },
                    title: Some(title),
                    ..Default::default()
                },
            ))
            .map_err(|e| e.to_string())?;
            match result {
                Some(mut files) => {
                    if let Some(file) = files.pop() {
                        let bytes = file.read_bytes().map_err(|e| e.to_string())?;
                        String::from_utf8(bytes.to_vec())
                            .map(Some)
                            .map_err(|e| e.to_string())
                    } else {
                        Ok(None)
                    }
                }
                None => Ok(None),
            }
        })();
        let _ = tx.send(result);
    });

    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(async move {
        let result = (|| async {
            let result = RlobKit::open_file_picker(OpenFileOptions {
                file_type: RlobKitType::Custom {
                    extensions: exts,
                    mime_types: vec!["text/markdown".to_string()],
                },
                title: Some(title),
                ..Default::default()
            })
            .await
            .map_err(|e| e.to_string())?;
            match result {
                Some(mut files) => {
                    if let Some(file) = files.pop() {
                        let bytes = file.read_bytes().map_err(|e| e.to_string())?;
                        String::from_utf8(bytes.to_vec())
                            .map(Some)
                            .map_err(|e| e.to_string())
                    } else {
                        Ok(None)
                    }
                }
                None => Ok(None),
            }
        })()
        .await;
        let _ = tx.send(result);
    });

    rx
}

pub fn spawn_save_file(
    title: &str,
    suggested_name: &str,
    extension: &str,
    content: Vec<u8>,
) -> flume::Receiver<Result<SaveOutcome, String>> {
    let (tx, rx) = flume::unbounded();
    let title = title.to_string();
    let suggested = suggested_name.to_string();
    let ext = extension.to_string();

    #[cfg(not(target_arch = "wasm32"))]
    std::thread::spawn(move || {
        let result = (|| -> Result<SaveOutcome, String> {
            match futures_lite::future::block_on(RlobKit::save_bytes(
                SaveFileOptions {
                    suggested_name: Some(suggested),
                    extension: Some(ext),
                    file_type: Some(RlobKitType::Custom {
                        extensions: vec![],
                        mime_types: vec![],
                    }),
                    title: Some(title),
                    ..Default::default()
                },
                &content,
            ))
            .map_err(|e| e.to_string())?
            {
                Some(_) => Ok(SaveOutcome::Saved),
                None => Ok(SaveOutcome::Cancelled),
            }
        })();
        let _ = tx.send(result);
    });

    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(async move {
        let result = (|| async {
            match RlobKit::save_bytes(
                SaveFileOptions {
                    suggested_name: Some(suggested),
                    extension: Some(ext),
                    file_type: Some(RlobKitType::Custom {
                        extensions: vec![],
                        mime_types: vec![],
                    }),
                    title: Some(title),
                    ..Default::default()
                },
                &content,
            )
            .await
            .map_err(|e| e.to_string())?
            {
                Some(_) => Ok(SaveOutcome::Saved),
                None => Ok(SaveOutcome::Cancelled),
            }
        })()
        .await;
        let _ = tx.send(result);
    });

    rx
}
