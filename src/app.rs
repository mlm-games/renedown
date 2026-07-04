use crate::markdown::MarkdownDocument;
use repose_core::scroll::ScrollBinding;
use repose_core::{PaddingValues, prelude::*, set_theme_default, signal};
use repose_material::material3::*;
use repose_ui::scroll::remember_scroll_state;
use repose_ui::*;
use std::rc::Rc;

const SAMPLE: &str = r##"# Renedown

A **real** Markdown renderer using `pulldown-cmark 0.13`, drawn with [Repose Material 3](https://example.com).

## Typography

Normal paragraph with **bold**, _italic_, ~~strikethrough~~ and `inline code`.
Soft break — same visual line.

Hard break follows (two trailing spaces)  
new visual line.

Super ^script^ and sub ~script~ demo. Math inline: $E = mc^2$.

Latex won't work yet

## Block quote

> "Simplicity is the ultimate sophistication."
>
> Nested paragraph inside the quote.

## Lists

- Alpha
- Beta
  - Nested
  - Another
- Gamma

1. First
2. Second
3. Third

- [x] Clickable links
- [x] Tables with alignment
- [ ] Native image loading

## Definition list

Term one
: The first definition.
: An alternative definition.

Term two
: The definition for the second term.

## Table

| Feature            | Status | Notes         |
|:-------------------|:------:|--------------:|
| Headings H1-H6     |   ✓    | with dividers |
| Bold / Italic      |   ✓    |               |
| Strikethrough      |   ✓    |               |
| Inline code        |   ✓    |               |
| Fenced code blocks |   ✓    | lang badge    |
| Block quotes       |   ✓    | accent bar    |
| Ordered lists      |   ✓    |               |
| Nested lists       |   ✓    |               |
| Task lists         |   ✓    |               |
| Tables             |   ✓    | col-alignment |
| Links / wikilinks  |   ✓    |               |
| Horizontal rules   |   ✓    |               |
| Footnotes[^1]      |   ✓    |               |
| Math ($x^2$)       |   ✓    |               |
| Super / sub        |   ✓    |               |
| Definition lists   |   ✓    |               |

[^1]: This is a footnote definition.

## Code

```rust
fn greet(name: &str) -> String {
    format!("Hello, {name}!")
}
```

---

> Runs on **desktop**, **web**, and **Android** from one Repose codebase.
"##;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Pane {
    Editor,
    Preview,
}

const COMPACT_BREAKPOINT: f32 = 760.0;

pub fn app(_s: &mut Scheduler) -> View {
    set_theme_default(Theme::default());

    // Single reactive source of truth.
    let doc = remember_with_key("renedown:doc", || signal(SAMPLE.to_string()));
    let last_link = remember_with_key("renedown:last_link", || signal(String::new()));
    let pane = remember_with_key("renedown:pane", || signal(Pane::Preview));
    let page_scroll = remember_scroll_state("renedown:page_scroll");

    // Compact mode: set by on_size_changed on the root Box.
    let compact = remember_with_key("renedown:compact", || signal(false));

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
        // Only build the visible pane in compact mode.
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

    // Page-wide scroll binding (cached so physics persists).
    let page_binding =
        remember_with_key("renedown:page_binding", || page_scroll.to_binding());
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
            {
                let doc = doc.clone();
                move || doc.set(SAMPLE.to_string())
            },
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

    actions.push(TextButton(
        Modifier::new(),
        on_reset,
        ButtonConfig::default(),
        || Text("Sample").size(14.0),
    ));
    actions.push(hspace(4.0));
    actions.push(TextButton(
        Modifier::new(),
        on_clear,
        ButtonConfig::default(),
        || Text("Clear").size(14.0),
    ));

    Column(Modifier::new().fill_max_width().background(theme().surface)).child((
        TopAppBar(
            Text("Renedown"),
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
                .color(theme().primary),
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
    MarkdownDocument(&value, on_link)
        .modifier(Modifier::new().fill_max_size().background(theme().surface_container_lowest).padding(18.0))
}

fn divider() -> View {
    HorizontalDivider(DividerConfig::default())
}

fn hspace(dp: f32) -> View {
    Box(Modifier::new().width(dp).height(1.0))
}
