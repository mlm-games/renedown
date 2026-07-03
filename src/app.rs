use crate::markdown::MarkdownDocument;
use repose_core::{prelude::*, set_theme_default, signal};
use repose_material::material3::*;
use repose_ui::scroll::{ScrollArea, ScrollState, remember_scroll_state};
use repose_ui::*;
use std::rc::Rc;

const SAMPLE: &str = r##"# Renedown

A **real** Markdown renderer using `pulldown-cmark`, drawn with Repose Material 3.

## Features

- headings
- paragraphs
- block quotes
- ordered and unordered lists
- task lists
- fenced code blocks
- inline `code`
- clickable [links](https://crates.io)
- tables

---

## Table

| Feature         | Status |
|-----------------|--------|
| Tables          | yes    |
| Clickable link  | yes    |
| Task lists      | yes    |

## Task list

- [x] Real `pulldown-cmark` parser
- [x] Tables
- [x] Clickable links
- [ ] Images rendered natively

> This app runs on desktop, web (WebGPU/WebGL), and Android
> from a single Repose codebase.

## Code

```rust
fn main() {
    println!("hello from renedown");
}
```
"##;

/// Which pane is visible in the compact (phone) layout.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Pane {
    Editor,
    Preview,
}

const COMPACT_BREAKPOINT: f32 = 720.0;

pub fn app(_s: &mut Scheduler) -> View {
    set_theme_default(Theme::default());

    let doc = remember_with_key("renedown:doc", || signal(SAMPLE.to_string()));
    let last_link = remember_with_key("renedown:last_link", || signal(String::new()));
    let pane = remember_with_key("renedown:pane", || signal(Pane::Preview));
    let preview_scroll = remember_scroll_state("renedown:preview_scroll");
    let editor_scroll = remember_scroll_state("renedown:editor_scroll");

    let current_doc = doc.get();
    let current_link = last_link.get();
    let current_pane = pane.get();

    let on_link: Rc<dyn Fn(String)> = {
        let last_link = last_link.clone();
        Rc::new(move |url: String| {
            log::info!("link clicked: {}", url);
            last_link.set(url);
        })
    };

    Surface(
        Modifier::new()
            .fill_max_size()
            .background(theme().background),
        Column(Modifier::new().fill_max_size()).child((
            box_with_constraints_with_key(
                format!(
                    "{}:{}",
                    current_doc.len(),
                    matches!(current_pane, Pane::Editor)
                ),
                Modifier::new().fill_max_width().weight(1.0),
                {
                    let doc = doc.clone();
                    let pane = pane.clone();
                    let preview_scroll = preview_scroll.clone();
                    let editor_scroll = editor_scroll.clone();
                    let on_link = on_link.clone();
                    let current_doc = current_doc.clone();

                    move |scope| {
                        let compact = scope.max_width < COMPACT_BREAKPOINT;

                        let top = top_bar(
                            compact,
                            current_pane,
                            {
                                let pane = pane.clone();
                                move |p| pane.set(p)
                            },
                            {
                                let doc = doc.clone();
                                move || doc.set(SAMPLE.to_string())
                            },
                            {
                                let doc = doc.clone();
                                move || doc.set(String::new())
                            },
                        );

                        let editor = panel(
                            "Editor",
                            editor_view(current_doc.clone(), editor_scroll.clone(), {
                                let doc = doc.clone();
                                move |s: String| doc.set(s)
                            }),
                        );

                        let preview = panel(
                            "Preview",
                            preview_view(
                                current_doc.clone(),
                                preview_scroll.clone(),
                                on_link.clone(),
                            ),
                        );

                        let body = if compact {
                            // Phone: one pane at a time, switched from the app bar.
                            Column(Modifier::new().fill_max_size().padding(12.0)).child(
                                match current_pane {
                                    Pane::Editor => editor,
                                    Pane::Preview => preview,
                                },
                            )
                        } else {
                            // Desktop / tablet / wide web: split view.
                            Row(Modifier::new().fill_max_size().padding(16.0)).child((
                                Surface(Modifier::new().fill_max_height().weight(1.0), editor),
                                hspace(16.0),
                                Surface(Modifier::new().fill_max_height().weight(1.0), preview),
                            ))
                        };

                        Column(Modifier::new().fill_max_size()).child((top, body))
                    }
                },
            ),
            // ---------- status bar ----------
            status_bar(&current_doc, &current_link, {
                let last_link = last_link.clone();
                move || last_link.set(String::new())
            }),
        )),
    )
}

fn top_bar(
    compact: bool,
    current_pane: Pane,
    on_pane: impl Fn(Pane) + Clone + 'static,
    on_reset: impl Fn() + 'static,
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

    actions.push(TextButton(Modifier::new(), on_reset, || {
        Text("Sample").size(14.0)
    }));
    actions.push(hspace(4.0));
    actions.push(TextButton(Modifier::new(), on_clear, || {
        Text("Clear").size(14.0)
    }));

    Surface(
        Modifier::new()
            .fill_max_width()
            .background(theme().surface)
            .border(1.0, theme().outline, 0.0),
        Row(Modifier::new()
            .fill_max_width()
            .height(64.0)
            .padding(12.0)
            .align_items(AlignItems::Center))
        .child((
            Column(Modifier::new()).child((
                Text("Renedown").size(20.0).color(theme().on_surface),
                Text("Markdown \u{2022} desktop / web / android")
                    .size(11.0)
                    .color(theme().on_surface_variant),
            )),
            Spacer(),
            Row(Modifier::new().align_items(AlignItems::Center)).with_children(actions),
        )),
    )
}

/// A small M3-style segmented control built from primitives.
fn segmented(
    options: &[(&'static str, Pane)],
    current: Pane,
    on_select: impl Fn(Pane) + Clone + 'static,
) -> View {
    let buttons: Vec<View> = options
        .iter()
        .map(|(label, value)| {
            let active = *value == current;
            let value = *value;
            let on_select = on_select.clone();

            Surface(
                Modifier::new()
                    .padding(8.0)
                    .background(if active {
                        theme().primary
                    } else {
                        theme().surface
                    })
                    .border(
                        1.0,
                        if active {
                            theme().primary
                        } else {
                            theme().outline
                        },
                        16.0,
                    )
                    .on_pointer_up(move |_| on_select(value)),
                Text(label.to_string()).size(13.0).color(if active {
                    theme().on_primary
                } else {
                    theme().on_surface
                }),
            )
        })
        .collect();

    let mut children = Vec::new();
    for (i, b) in buttons.into_iter().enumerate() {
        if i > 0 {
            children.push(hspace(4.0));
        }
        children.push(b);
    }

    Row(Modifier::new().align_items(AlignItems::Center)).with_children(children)
}

fn status_bar(doc: &str, last_link: &str, on_dismiss: impl Fn() + 'static) -> View {
    let words = doc.split_whitespace().count();
    let chars = doc.chars().count();

    let mut children: Vec<View> = vec![
        Text(format!("{} words \u{2022} {} chars", words, chars))
            .size(12.0)
            .color(theme().on_surface_variant),
        Spacer(),
    ];

    if !last_link.is_empty() {
        children.push(
            Text(format!("Link: {}", last_link))
                .size(12.0)
                .color(theme().primary),
        );
        children.push(hspace(8.0));
        children.push(TextButton(Modifier::new(), on_dismiss, || {
            Text("\u{2715}").size(12.0)
        }));
    }

    Surface(
        Modifier::new()
            .fill_max_width()
            .padding(8.0)
            .background(theme().surface)
            .border(1.0, theme().outline, 0.0),
        Row(Modifier::new()
            .fill_max_width()
            .align_items(AlignItems::Center))
        .with_children(children),
    )
}

fn panel(title: &str, body: View) -> View {
    OutlinedCard(
        Modifier::new().fill_max_size(),
        Column(Modifier::new().fill_max_size()).child((
            Surface(
                Modifier::new()
                    .fill_max_width()
                    .padding(10.0)
                    .background(theme().surface),
                Text(title.to_uppercase())
                    .size(12.0)
                    .color(theme().on_surface_variant),
            ),
            Divider(),
            Surface(Modifier::new().fill_max_size().weight(1.0), body),
        )),
    )
}

fn editor_view(
    value: String,
    scroll: Rc<ScrollState>,
    on_change: impl Fn(String) + 'static,
) -> View {
    ScrollArea(
        Modifier::new().fill_max_size(),
        scroll,
        Surface(
            Modifier::new().fill_max_size().padding(12.0),
            TextArea(
                "Write markdown\u{2026}",
                value,
                Modifier::new().fill_max_size(),
                Some(on_change),
                None::<fn(String)>,
            ),
        ),
    )
}

fn preview_view(value: String, scroll: Rc<ScrollState>, on_link: Rc<dyn Fn(String)>) -> View {
    ScrollArea(
        Modifier::new().fill_max_size(),
        scroll,
        Surface(
            Modifier::new().fill_max_width().padding(16.0),
            MarkdownDocument(&value, on_link),
        ),
    )
}

fn hspace(dp: f32) -> View {
    Space(Modifier::new().width(dp))
}
