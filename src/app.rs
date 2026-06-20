use crate::markdown::MarkdownDocument;
use repose_core::{prelude::*, set_theme_default, signal};
use repose_material::material3::*;
use repose_ui::scroll::{remember_scroll_state, ScrollArea};
use repose_ui::*;
use std::rc::Rc;

const SAMPLE: &str = r##"# Renedown

A **real** Markdown renderer using `pulldown-cmark`.

## Features

- headings
- paragraphs
- block quotes
- ordered and unordered lists
- task lists
- fenced code blocks
- inline `code`
- 
- tables

---

## Table

| Feature         | Status |
|----------------|--------|
| Tables         | yes    |
| Clickable link | yes    |
| Task lists     | yes    |

## Task list

- [x] Real `pulldown-cmark` parser
- [x] Tables
- [x] Clickable links
- [ ] Images rendered natively

> This app is written in the showcase-style crate shape.
> The renderer is a real event-driven Markdown renderer.

## Code

```rust
fn main() {
    println!("hello from renedown");
}
```
"##;

pub fn app(_s: &mut Scheduler) -> View {
    set_theme_default(Theme::default());

    let doc = remember_with_key("renedown:doc", || signal(SAMPLE.to_string()));
    let last_link = remember_with_key("renedown:last_link", || signal(String::new()));
    let preview_scroll = remember_scroll_state("renedown:preview_scroll");

    let current_doc = doc.get();
    let current_link = last_link.get();

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
            top_bar(
                {
                    let doc = doc.clone();
                    move || doc.set(SAMPLE.to_string())
                },
                {
                    let doc = doc.clone();
                    move || doc.set(String::new())
                },
            ),
            if current_link.is_empty() {
                Box(Modifier::new())
            } else {
                Surface(
                    Modifier::new()
                        .fill_max_width()
                        .padding(8.0)
                        .background(theme().surface)
                        .border(1.0, theme().outline, 0.0),
                    Text(format!("Last clicked: {}", current_link))
                        .size(12.0)
                        .color(theme().on_surface_variant),
                )
            },
            box_with_constraints_with_key(
                current_doc.clone(),
                Modifier::new().fill_max_size().padding(16.0),
                {
                    let doc = doc.clone();
                    let preview_scroll = preview_scroll.clone();
                    let on_link = on_link.clone();

                    move |scope| {
                        let editor = panel(
                            "Editor",
                            editor_view(
                                current_doc.clone(),
                                {
                                    let doc = doc.clone();
                                    move |s: String| doc.set(s)
                                },
                            ),
                        );

                        let preview = panel(
                            "Preview",
                            preview_view(
                                current_doc.clone(),
                                preview_scroll.clone(),
                                on_link.clone(),
                            ),
                        );

                        if scope.max_width < 900.0 {
                            Column(Modifier::new().fill_max_size()).child((
                                Surface(
                                    Modifier::new()
                                        .fill_max_width()
                                        .height(320.0),
                                    editor,
                                ),
                                vspace(16.0),
                                Surface(
                                    Modifier::new().fill_max_size(),
                                    preview,
                                ),
                            ))
                        } else {
                            Row(Modifier::new().fill_max_size()).child((
                                Surface(
                                    Modifier::new()
                                        .fill_max_height()
                                        .weight(1.0),
                                    editor,
                                ),
                                hspace(16.0),
                                Surface(
                                    Modifier::new()
                                        .fill_max_height()
                                        .weight(1.0),
                                    preview,
                                ),
                            ))
                        }
                    }
                },
            ),
        )),
    )
}

fn top_bar(
    on_reset: impl Fn() + 'static,
    on_clear: impl Fn() + 'static,
) -> View {
    Surface(
        Modifier::new()
            .fill_max_width()
            .padding(12.0)
            .background(theme().surface)
            .border(1.0, theme().outline, 0.0),
        Row(Modifier::new().fill_max_width().height(64.0).align_items(AlignItems::Center)).child((
            Column(Modifier::new()).child((
                Text("Renedown")
                    .size(20.0)
                    .color(theme().on_surface),
                Text("desktop / web / android")
                    .size(12.0)
                    .color(theme().on_surface_variant),
            )),
            Spacer(),
            TextButton(Modifier::new(), on_reset, || {
                Text("Load sample").size(14.0)
            }),
            hspace(8.0),
            TextButton(Modifier::new(), on_clear, || {
                Text("Clear").size(14.0)
            }),
        )),
    )
}

fn panel(title: &str, body: View) -> View {
    OutlinedCard(
        Modifier::new().fill_max_size(),
        Column(Modifier::new().fill_max_size()).child((
            Surface(
                Modifier::new()
                    .fill_max_width()
                    .padding(12.0)
                    .background(theme().surface),
                Text(title)
                    .size(14.0)
                    .color(theme().on_surface_variant),
            ),
            Surface(
                Modifier::new()
                    .fill_max_size()
                    .padding(12.0),
                body,
            ),
        )),
    )
}

fn editor_view(
    value: String,
    on_change: impl Fn(String) + 'static,
) -> View {
    Surface(
        Modifier::new()
            .fill_max_size()
            .padding(12.0),
        TextArea(
            "Write markdown\u{2026}",
            value,
            Modifier::new().fill_max_size(),
            Some(on_change),
            None::<fn(String)>,
        ),
    )
}

fn preview_view(
    value: String,
    scroll: Rc<repose_ui::scroll::ScrollState>,
    on_link: Rc<dyn Fn(String)>,
) -> View {
    ScrollArea(
        Modifier::new().fill_max_size(),
        scroll,
        Surface(
            Modifier::new()
                .fill_max_width()
                .padding(16.0),
            MarkdownDocument(&value, on_link),
        ),
    )
}

fn vspace(dp: f32) -> View {
    Space(Modifier::new().height(dp))
}

fn hspace(dp: f32) -> View {
    Space(Modifier::new().width(dp))
}
