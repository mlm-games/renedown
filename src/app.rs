use crate::file_picker;
use crate::markdown::MarkdownDocument;
use repose_core::scroll::ScrollBinding;
use repose_core::{PaddingValues, prelude::*, set_theme_default, signal};
use repose_material::material3::*;
use repose_material::{Icon, material_symbols};
use repose_ui::scroll::remember_scroll_state;
use repose_ui::*;

material_symbols! {
    FOLDER_OPEN : '\u{E2C8}',
    SAVE        : '\u{E161}',
    CLOSE       : '\u{E5CD}',
}
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Pane {
    Editor,
    Preview,
}

const COMPACT_BREAKPOINT: f32 = 760.0;

pub fn app(_s: &mut Scheduler) -> View {
    set_theme_default(Theme::default());

    let doc = remember_with_key("renedown:doc", || signal(String::new()));
    let last_link = remember_with_key("renedown:last_link", || signal(String::new()));
    let pane = remember_with_key("renedown:pane", || signal(Pane::Preview));
    let page_scroll = remember_scroll_state("renedown:page_scroll");
    let compact = remember_with_key("renedown:compact", || signal(false));

    // File picker state: receivers to poll
    let open_rx = remember_with_key("renedown:open_rx", || {
        signal(None::<flume::Receiver<Result<Option<String>, String>>>)
    });
    let save_rx = remember_with_key("renedown:save_rx", || {
        signal(None::<flume::Receiver<Result<(), String>>>)
    });

    // Poll open result
    {
        let rx = open_rx.get();
        if let Some(rx) = rx {
            match rx.try_recv() {
                Ok(result) => {
                    match result {
                        Ok(Some(content)) => doc.set(content),
                        Ok(None) => {}
                        Err(e) => log::error!("Failed to open file: {e}"),
                    }
                    open_rx.set(None);
                }
                Err(flume::TryRecvError::Empty) => open_rx.set(Some(rx)),
                Err(flume::TryRecvError::Disconnected) => open_rx.set(None),
            }
        }
    }

    // Poll save result
    {
        let rx = save_rx.get();
        if let Some(rx) = rx {
            match rx.try_recv() {
                Ok(result) => {
                    match result {
                        Ok(()) => log::info!("File saved"),
                        Err(e) => log::error!("Failed to save file: {e}"),
                    }
                    save_rx.set(None);
                }
                Err(flume::TryRecvError::Empty) => save_rx.set(Some(rx)),
                Err(flume::TryRecvError::Disconnected) => save_rx.set(None),
            }
        }
    }

    let on_open = {
        let open_rx = open_rx.clone();
        move || {
            let rx = file_picker::spawn_open_file("Open Markdown", &["md", "markdown"]);
            open_rx.set(Some(rx));
        }
    };

    let on_save = {
        let doc = doc.clone();
        let save_rx = save_rx.clone();
        move || {
            let content = doc.get().into_bytes();
            let rx = file_picker::spawn_save_file("Save Markdown", "document", "md", content);
            save_rx.set(Some(rx));
        }
    };

    let current_doc = doc.get();
    let current_link = last_link.get();
    let current_pane = pane.get();
    let is_compact = compact.get();

    let on_link: Rc<dyn Fn(String)> = {
        let last_link = last_link.clone();
        Rc::new(move |url: String| {
            log::info!("link clicked: {url}");
            last_link.set(url);
        })
    };

    let body: View = if is_compact {
        let pane_content = match current_pane {
            Pane::Editor => panel(
                "Editor",
                "Write Markdown",
                editor_view(current_doc.clone(), {
                    let doc = doc.clone();
                    move |s| doc.set(s)
                }),
            ),
            Pane::Preview => panel(
                "Preview",
                "Rendered document",
                preview_view(current_doc.clone(), on_link.clone()),
            ),
        };
        Column(Modifier::new().fill_max_size().padding(12.0)).child(pane_content)
    } else {
        Row(Modifier::new()
            .fill_max_size()
            .padding(18.0)
            .column_gap(18.0))
        .child((
            Box(Modifier::new().fill_max_height().flex_grow(1.0)).child(panel(
                "Editor",
                "Write Markdown",
                editor_view(current_doc.clone(), {
                    let doc = doc.clone();
                    move |s| doc.set(s)
                }),
            )),
            Box(Modifier::new().fill_max_height().flex_grow(1.0)).child(panel(
                "Preview",
                "Rendered document",
                preview_view(current_doc.clone(), on_link.clone()),
            )),
        ))
    };

    let page_binding = remember_with_key("renedown:page_binding", || page_scroll.to_binding());
    let page_axis = match &*page_binding {
        ScrollBinding::Vertical(b) => b.clone(),
        _ => unreachable!(),
    };

    let inner = Column(Modifier::new().fill_max_size()).child((
        top_bar(
            is_compact,
            current_pane,
            {
                let pane = pane.clone();
                move |p| pane.set(p)
            },
            on_open,
            on_save,
            {
                let doc = doc.clone();
                move || doc.set(String::new())
            },
        ),
        Box(Modifier::new().fill_max_width().flex_grow(1.0)).child(body),
        status_bar(&current_doc, &current_link, {
            let last_link = last_link.clone();
            move || last_link.set(String::new())
        }),
    ));

    Box(Modifier::new()
        .fill_max_size()
        .background(theme().background)
        .on_size_changed({
            let compact = compact.clone();
            move |size| compact.set(size.x < COMPACT_BREAKPOINT)
        }))
    .child(
        View::new(0, ViewKind::Box)
            .modifier(Modifier::new().fill_max_size().vertical_scroll(page_axis))
            .with_children(vec![inner]),
    )
}

fn top_bar(
    compact: bool,
    current_pane: Pane,
    on_pane: impl Fn(Pane) + Clone + 'static,
    on_open: impl Fn() + 'static,
    on_save: impl Fn() + 'static,
    on_clear: impl Fn() + 'static,
) -> View {
    let mut actions: Vec<View> = Vec::new();

    if compact {
        actions.push(segmented(
            &[("Edit", Pane::Editor), ("Read", Pane::Preview)],
            current_pane,
            on_pane,
        ));
        actions.push(hspace(8.0));
    }

    actions.push(IconButton(
        Icon(Symbols::FOLDER_OPEN).size(20.0),
        on_open,
        IconButtonConfig::default(),
    ));
    actions.push(IconButton(
        Icon(Symbols::SAVE).size(20.0),
        on_save,
        IconButtonConfig::default(),
    ));
    actions.push(OutlinedButton(
        Modifier::new(),
        on_clear,
        ButtonConfig::default(),
        || {
            Row(Modifier::new()
                .align_items(AlignItems::CENTER)
                .column_gap(4.0))
            .child((Icon(Symbols::CLOSE).size(18.0), Text("Clear").size(14.0)))
        },
    ));

    Column(Modifier::new().fill_max_width().background(theme().surface)).child((
        TopAppBar(
            #[cfg(all(not(target_os = "android")))]
            Text("Renedown"),
            #[cfg(all(target_os = "android"))]
            Text(""),
            None,
            None,
            actions,
            TopAppBarConfig::default(),
        ),
        Box(Modifier::new()
            .fill_max_width()
            .padding_values(PaddingValues {
                left: 16.0,
                right: 16.0,
                top: 0.0,
                bottom: 10.0,
            }))
        .child(
            Text("Markdown editor · desktop / web / Android")
                .size(12.0)
                .color(theme().on_surface_variant),
        ),
        divider(),
    ))
}

fn segmented(
    options: &[(&'static str, Pane)],
    current: Pane,
    on_select: impl Fn(Pane) + Clone + 'static,
) -> View {
    let children: Vec<View> = options
        .iter()
        .map(|(label, value)| {
            let value = *value;
            let selected = value == current;
            let on_select = on_select.clone();

            FilterChip(
                selected,
                move || on_select(value),
                Text(*label).size(13.0),
                None,
                None,
                ChipConfig::default(),
            )
        })
        .collect();

    Row(Modifier::new()
        .align_items(AlignItems::CENTER)
        .column_gap(6.0))
    .child(children)
}

fn status_bar(doc: &str, last_link: &str, on_dismiss: impl Fn() + 'static) -> View {
    let words = doc.split_whitespace().count();
    let chars = doc.chars().count();
    let lines = doc.lines().count().max(1);

    let mut children: Vec<View> = vec![
        Text(format!("{lines} lines · {words} words · {chars} chars"))
            .size(12.0)
            .color(theme().on_surface_variant),
        Spacer(),
    ];

    if !last_link.is_empty() {
        children.push(
            Text(format!("Link: {last_link}"))
                .size(12.0)
                .color(theme().primary)
                .url(last_link.to_string()),
        );
        children.push(hspace(8.0));
        children.push(TextButton(
            Modifier::new(),
            on_dismiss,
            ButtonConfig::default(),
            || Text("Dismiss").size(12.0),
        ));
    }

    Box(Modifier::new()
        .fill_max_width()
        .background(theme().surface)
        .padding_values(PaddingValues {
            left: 12.0,
            right: 12.0,
            top: 8.0,
            bottom: 8.0,
        }))
    .child(
        Row(Modifier::new()
            .fill_max_width()
            .align_items(AlignItems::CENTER))
        .child(children),
    )
}

fn panel(title: &str, subtitle: &str, body: View) -> View {
    Box(Modifier::new()
        .fill_max_size()
        .background(theme().surface_container_low)
        .clip_rounded(22.0)
        .border(1.0, theme().outline_variant, 22.0))
    .child(
        Column(Modifier::new().fill_max_size()).child((
            Box(Modifier::new()
                .fill_max_width()
                .padding_values(PaddingValues {
                    left: 18.0,
                    right: 18.0,
                    top: 14.0,
                    bottom: 12.0,
                }))
            .child(Column(Modifier::new()).child((
                Text(title).size(16.0).color(theme().on_surface),
                Text(subtitle).size(11.0).color(theme().on_surface_variant),
            ))),
            divider(),
            Box(Modifier::new().fill_max_size().flex_grow(1.0)).child(body),
        )),
    )
}

fn editor_view(value: String, on_change: impl Fn(String) + 'static) -> View {
    TextField(
        Modifier::new().fill_max_width().fill_max_height(),
        value,
        on_change,
        repose_material::material3::TextFieldConfig {
            placeholder: Some("Write Markdown".into()),
            single_line: false,
            ..Default::default()
        },
    )
}

fn preview_view(value: String, on_link: Rc<dyn Fn(String)>) -> View {
    MarkdownDocument(&value, on_link).modifier(
        Modifier::new()
            .fill_max_size()
            .background(theme().surface_container_lowest)
            .padding(18.0),
    )
}

fn divider() -> View {
    HorizontalDivider(DividerConfig::default())
}

fn hspace(dp: f32) -> View {
    Box(Modifier::new().width(dp).height(1.0))
}
