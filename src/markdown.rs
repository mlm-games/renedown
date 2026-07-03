use pulldown_cmark::{
    Alignment as MdAlignment, CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd,
};
use repose_core::{PaddingValues, prelude::*};
use repose_material::material3::{DividerConfig, HorizontalDivider};
use repose_ui::*;
use std::rc::Rc;

#[derive(Debug, Clone)]
enum Block {
    Heading {
        level: u8,
        inlines: Vec<Inline>,
    },
    Paragraph(Vec<Inline>),
    BlockQuote(Vec<Block>),
    CodeBlock {
        lang: Option<String>,
        code: String,
    },
    List {
        ordered: bool,
        start: usize,
        items: Vec<ListItem>,
    },
    Table {
        alignments: Vec<MdAlignment>,
        head: Vec<Vec<Inline>>,
        rows: Vec<Vec<Vec<Inline>>>,
    },
    Rule,
    Html(String),
}

#[derive(Debug, Clone)]
struct ListItem {
    task: Option<bool>,
    blocks: Vec<Block>,
}

#[derive(Debug, Clone)]
enum Inline {
    Text(String),
    Code(String),
    Strong(Vec<Inline>),
    Emphasis(Vec<Inline>),
    Strike(Vec<Inline>),
    Link { label: Vec<Inline>, url: String },
    Image { alt: Vec<Inline>, url: String },
    SoftBreak,
    HardBreak,
    TaskMarker(bool),
}

#[derive(Clone, Copy)]
struct InlineStyle {
    size: f32,
    color: Color,
}

#[allow(non_snake_case)]
pub fn MarkdownDocument(src: &str, on_link: Rc<dyn Fn(String)>) -> View {
    let blocks = parse_markdown(src);
    let rendered = intersperse_vertical(
        blocks
            .iter()
            .map(|b| render_block(b, on_link.clone()))
            .collect(),
        12.0,
    );

    Column(Modifier::new().fill_max_width()).child(rendered)
}

fn parse_markdown(src: &str) -> Vec<Block> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let events: Vec<_> = Parser::new_ext(src, options).collect();
    let mut pos = 0usize;
    parse_blocks(&events, &mut pos)
}

fn parse_blocks(events: &[Event<'_>], pos: &mut usize) -> Vec<Block> {
    let mut blocks = Vec::new();

    while let Some(event) = events.get(*pos) {
        match event {
            Event::Start(Tag::Paragraph) => {
                *pos += 1;
                let inlines = parse_inlines(events, pos);
                consume_end(events, pos, |end| matches!(end, TagEnd::Paragraph));
                blocks.push(Block::Paragraph(inlines));
            }

            Event::Start(Tag::Heading { level, .. }) => {
                let level = heading_level_to_u8(*level);
                *pos += 1;
                let inlines = parse_inlines(events, pos);
                consume_end(events, pos, |end| matches!(end, TagEnd::Heading(_)));
                blocks.push(Block::Heading { level, inlines });
            }

            Event::Start(Tag::BlockQuote(_)) => {
                *pos += 1;
                let inner = parse_blocks(events, pos);
                consume_end(events, pos, |end| matches!(end, TagEnd::BlockQuote(_)));
                blocks.push(Block::BlockQuote(inner));
            }

            Event::Start(Tag::CodeBlock(kind)) => {
                let lang = match kind {
                    CodeBlockKind::Fenced(info) => {
                        let first = info
                            .split_whitespace()
                            .next()
                            .unwrap_or("")
                            .trim()
                            .to_string();
                        if first.is_empty() { None } else { Some(first) }
                    }
                    CodeBlockKind::Indented => None,
                };

                *pos += 1;
                let mut code = String::new();

                while let Some(ev) = events.get(*pos) {
                    match ev {
                        Event::End(TagEnd::CodeBlock) => {
                            *pos += 1;
                            break;
                        }
                        Event::Text(t) | Event::Code(t) | Event::Html(t) | Event::InlineHtml(t) => {
                            code.push_str(t);
                            *pos += 1;
                        }
                        Event::SoftBreak | Event::HardBreak => {
                            code.push('\n');
                            *pos += 1;
                        }
                        _ => {
                            *pos += 1;
                        }
                    }
                }

                blocks.push(Block::CodeBlock { lang, code });
            }

            Event::Start(Tag::List(start)) => {
                let ordered = start.is_some();
                let start = start.unwrap_or(1) as usize;
                *pos += 1;
                let items = parse_list_items(events, pos);
                consume_end(events, pos, |end| matches!(end, TagEnd::List(_)));
                blocks.push(Block::List {
                    ordered,
                    start,
                    items,
                });
            }

            Event::Start(Tag::Table(alignments)) => {
                let alignments = alignments.clone();
                *pos += 1;
                let (head, rows) = parse_table(events, pos);
                blocks.push(Block::Table {
                    alignments,
                    head,
                    rows,
                });
            }

            Event::Rule => {
                *pos += 1;
                blocks.push(Block::Rule);
            }

            Event::Html(html) => {
                *pos += 1;
                blocks.push(Block::Html(html.to_string()));
            }

            Event::Text(_)
            | Event::Code(_)
            | Event::TaskListMarker(_)
            | Event::SoftBreak
            | Event::HardBreak
            | Event::Start(Tag::Strong)
            | Event::Start(Tag::Emphasis)
            | Event::Start(Tag::Strikethrough)
            | Event::Start(Tag::Link { .. })
            | Event::Start(Tag::Image { .. }) => {
                let inlines = parse_inlines(events, pos);
                if !inlines.is_empty() {
                    blocks.push(Block::Paragraph(inlines));
                }
            }

            Event::End(_) => break,

            _ => {
                *pos += 1;
            }
        }
    }

    blocks
}

fn parse_list_items(events: &[Event<'_>], pos: &mut usize) -> Vec<ListItem> {
    let mut items = Vec::new();

    while let Some(event) = events.get(*pos) {
        match event {
            Event::Start(Tag::Item) => {
                *pos += 1;
                let mut blocks = parse_blocks(events, pos);
                consume_end(events, pos, |end| matches!(end, TagEnd::Item));
                let task = peel_task_marker(&mut blocks);
                items.push(ListItem { task, blocks });
            }
            Event::End(TagEnd::List(_)) => break,
            _ => *pos += 1,
        }
    }

    items
}

fn parse_table(events: &[Event<'_>], pos: &mut usize) -> (Vec<Vec<Inline>>, Vec<Vec<Vec<Inline>>>) {
    let mut head = Vec::new();
    let mut rows = Vec::new();

    while let Some(event) = events.get(*pos) {
        match event {
            Event::Start(Tag::TableHead) => {
                *pos += 1;
                head = parse_table_head(events, pos);
                consume_end(events, pos, |end| matches!(end, TagEnd::TableHead));
            }
            Event::Start(Tag::TableRow) => {
                *pos += 1;
                let row = parse_table_row(events, pos);
                consume_end(events, pos, |end| matches!(end, TagEnd::TableRow));
                rows.push(row);
            }
            Event::End(TagEnd::Table) => {
                *pos += 1;
                break;
            }
            _ => *pos += 1,
        }
    }

    (head, rows)
}

fn parse_table_head(events: &[Event<'_>], pos: &mut usize) -> Vec<Vec<Inline>> {
    let mut cells = Vec::new();

    while let Some(event) = events.get(*pos) {
        match event {
            Event::Start(Tag::TableCell) => {
                *pos += 1;
                let cell = parse_inlines(events, pos);
                consume_end(events, pos, |end| matches!(end, TagEnd::TableCell));
                cells.push(cell);
            }
            Event::Start(Tag::TableRow) => {
                *pos += 1;
                let row = parse_table_row(events, pos);
                consume_end(events, pos, |end| matches!(end, TagEnd::TableRow));
                return row;
            }
            Event::End(TagEnd::TableHead) => break,
            _ => *pos += 1,
        }
    }

    cells
}

fn parse_table_row(events: &[Event<'_>], pos: &mut usize) -> Vec<Vec<Inline>> {
    let mut cells = Vec::new();

    while let Some(event) = events.get(*pos) {
        match event {
            Event::Start(Tag::TableCell) => {
                *pos += 1;
                let cell = parse_inlines(events, pos);
                consume_end(events, pos, |end| matches!(end, TagEnd::TableCell));
                cells.push(cell);
            }
            Event::End(TagEnd::TableRow) => break,
            _ => *pos += 1,
        }
    }

    cells
}

fn parse_inlines(events: &[Event<'_>], pos: &mut usize) -> Vec<Inline> {
    let mut inlines = Vec::new();

    while let Some(event) = events.get(*pos) {
        match event {
            Event::Text(t) => {
                inlines.push(Inline::Text(t.to_string()));
                *pos += 1;
            }

            Event::Code(t) => {
                inlines.push(Inline::Code(t.to_string()));
                *pos += 1;
            }

            Event::SoftBreak => {
                inlines.push(Inline::SoftBreak);
                *pos += 1;
            }

            Event::HardBreak => {
                inlines.push(Inline::HardBreak);
                *pos += 1;
            }

            Event::TaskListMarker(checked) => {
                inlines.push(Inline::TaskMarker(*checked));
                *pos += 1;
            }

            Event::Start(Tag::Strong) => {
                *pos += 1;
                let inner = parse_inlines(events, pos);
                consume_end(events, pos, |end| matches!(end, TagEnd::Strong));
                inlines.push(Inline::Strong(inner));
            }

            Event::Start(Tag::Emphasis) => {
                *pos += 1;
                let inner = parse_inlines(events, pos);
                consume_end(events, pos, |end| matches!(end, TagEnd::Emphasis));
                inlines.push(Inline::Emphasis(inner));
            }

            Event::Start(Tag::Strikethrough) => {
                *pos += 1;
                let inner = parse_inlines(events, pos);
                consume_end(events, pos, |end| matches!(end, TagEnd::Strikethrough));
                inlines.push(Inline::Strike(inner));
            }

            Event::Start(Tag::Link { dest_url, .. }) => {
                let url = dest_url.to_string();
                *pos += 1;
                let label = parse_inlines(events, pos);
                consume_end(events, pos, |end| matches!(end, TagEnd::Link));
                inlines.push(Inline::Link { label, url });
            }

            Event::Start(Tag::Image { dest_url, .. }) => {
                let url = dest_url.to_string();
                *pos += 1;
                let alt = parse_inlines(events, pos);
                consume_end(events, pos, |end| matches!(end, TagEnd::Image));
                inlines.push(Inline::Image { alt, url });
            }

            Event::End(_) => break,

            _ => {
                *pos += 1;
            }
        }
    }

    inlines
}

fn peel_task_marker(blocks: &mut Vec<Block>) -> Option<bool> {
    let first = blocks.first_mut()?;
    let Block::Paragraph(inlines) = first else {
        return None;
    };

    if inlines.is_empty() {
        return None;
    }

    match inlines.remove(0) {
        Inline::TaskMarker(checked) => {
            if let Some(Inline::Text(t)) = inlines.first_mut() {
                *t = t.trim_start().to_string();
            }
            Some(checked)
        }
        other => {
            inlines.insert(0, other);
            None
        }
    }
}

fn consume_end<F>(events: &[Event<'_>], pos: &mut usize, pred: F)
where
    F: Fn(&TagEnd) -> bool,
{
    if let Some(Event::End(end)) = events.get(*pos) {
        if pred(end) {
            *pos += 1;
        }
    }
}

fn heading_level_to_u8(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn render_block(block: &Block, on_link: Rc<dyn Fn(String)>) -> View {
    match block {
        Block::Heading { level, inlines } => render_heading(*level, inlines, on_link),

        Block::Paragraph(inlines) => {
            FlowRow(Modifier::new().fill_max_width()).child(render_inlines(
                inlines,
                InlineStyle {
                    size: 15.0,
                    color: theme().on_surface,
                },
                on_link,
            ))
        }

        Block::BlockQuote(blocks) => {
            let children = intersperse_vertical(
                blocks
                    .iter()
                    .map(|b| render_block(b, on_link.clone()))
                    .collect(),
                8.0,
            );

            Box(Modifier::new()
                .fill_max_width()
                .background(theme().surface_container)
                .clip_rounded(14.0)
                .border(1.0, theme().outline_variant, 14.0))
            .child(
                Row(Modifier::new().fill_max_width()).child((
                    Box(Modifier::new()
                        .width(5.0)
                        .fill_max_height()
                        .background(theme().primary)),
                    Box(Modifier::new()
                        .fill_max_width()
                        .flex_grow(1.0)
                        .padding(14.0))
                    .child(Column(Modifier::new().fill_max_width()).child(children)),
                )),
            )
        }

        Block::CodeBlock { lang, code } => {
            let mut children: Vec<View> = Vec::new();

            if let Some(lang) = lang {
                children.push(
                    Row(Modifier::new().fill_max_width()).child((
                        Box(Modifier::new()
                            .background(theme().secondary_container)
                            .clip_rounded(999.0)
                            .padding_values(PaddingValues {
                                left: 8.0,
                                right: 8.0,
                                top: 4.0,
                                bottom: 4.0,
                            }))
                        .child(
                            Text(lang.clone())
                                .font_family("monospace")
                                .size(11.0)
                                .color(theme().on_secondary_container),
                        ),
                        Spacer(),
                    )),
                );
                children.push(vspace(10.0));
            }

            children.push(
                Text(code.trim_end().to_string())
                    .font_family("monospace")
                    .size(13.0)
                    .color(theme().on_surface),
            );

            Box(Modifier::new()
                .fill_max_width()
                .background(theme().surface_container)
                .clip_rounded(16.0)
                .border(1.0, theme().outline_variant, 16.0)
                .padding(14.0))
            .child(Column(Modifier::new().fill_max_width()).child(children))
        }

        Block::List {
            ordered,
            start,
            items,
        } => {
            let rendered = intersperse_vertical(
                items
                    .iter()
                    .enumerate()
                    .map(|(idx, item)| {
                        let marker = match item.task {
                            Some(true) => "☑".to_string(),
                            Some(false) => "☐".to_string(),
                            None if *ordered => format!("{}.", start + idx),
                            None => "•".to_string(),
                        };
                        render_list_item(&marker, &item.blocks, on_link.clone())
                    })
                    .collect(),
                7.0,
            );

            Column(Modifier::new().fill_max_width()).child(rendered)
        }

        Block::Table {
            alignments,
            head,
            rows,
        } => render_table(alignments, head, rows, on_link),

        Block::Rule => divider(),

        Block::Html(html) => Box(Modifier::new()
            .fill_max_width()
            .background(theme().surface_container)
            .clip_rounded(12.0)
            .border(1.0, theme().outline_variant, 12.0)
            .padding(12.0))
        .child(
            Text(html.clone())
                .font_family("monospace")
                .size(12.0)
                .color(theme().on_surface_variant),
        ),
    }
}

fn render_heading(level: u8, inlines: &[Inline], on_link: Rc<dyn Fn(String)>) -> View {
    let (size, color) = match level {
        1 => (31.0, theme().primary),
        2 => (25.0, theme().on_surface),
        3 => (21.0, theme().on_surface),
        4 => (18.0, theme().on_surface),
        _ => (15.0, theme().on_surface_variant),
    };

    let content = FlowRow(Modifier::new().fill_max_width()).child(render_inlines(
        inlines,
        InlineStyle { size, color },
        on_link,
    ));

    if level <= 2 {
        Column(Modifier::new().fill_max_width()).child((content, vspace(8.0), divider()))
    } else {
        content
    }
}

fn render_list_item(marker: &str, blocks: &[Block], on_link: Rc<dyn Fn(String)>) -> View {
    let rendered = intersperse_vertical(
        blocks
            .iter()
            .map(|b| render_block(b, on_link.clone()))
            .collect(),
        4.0,
    );

    Row(Modifier::new().fill_max_width()).child((
        Box(Modifier::new().width(30.0))
            .child(Text(marker.to_string()).size(15.0).color(theme().primary)),
        Box(Modifier::new().fill_max_width().flex_grow(1.0))
            .child(Column(Modifier::new().fill_max_width()).child(rendered)),
    ))
}

fn render_table(
    _alignments: &[MdAlignment],
    head: &[Vec<Inline>],
    rows: &[Vec<Vec<Inline>>],
    on_link: Rc<dyn Fn(String)>,
) -> View {
    let mut row_views = Vec::new();

    if !head.is_empty() {
        row_views.push(render_table_row(head, true, false, on_link.clone()));
    }

    for (i, row) in rows.iter().enumerate() {
        row_views.push(render_table_row(row, false, i % 2 == 1, on_link.clone()));
    }

    Box(Modifier::new()
        .fill_max_width()
        .background(theme().surface_container_low)
        .clip_rounded(16.0)
        .border(1.0, theme().outline_variant, 16.0))
    .child(Column(Modifier::new().fill_max_width()).child(row_views))
}

fn render_table_row(
    row: &[Vec<Inline>],
    header: bool,
    striped: bool,
    on_link: Rc<dyn Fn(String)>,
) -> View {
    let cells: Vec<View> = row
        .iter()
        .map(|cell| {
            let style = InlineStyle {
                size: if header { 14.0 } else { 13.5 },
                color: if header {
                    theme().primary
                } else {
                    theme().on_surface
                },
            };

            Box(Modifier::new().flex_grow(1.0).padding(10.0)).child(
                FlowRow(Modifier::new().fill_max_width()).child(render_inlines(
                    cell,
                    style,
                    on_link.clone(),
                )),
            )
        })
        .collect();

    let bg = if header {
        theme().primary_container
    } else if striped {
        theme().surface_container
    } else {
        theme().surface_container_low
    };

    Column(Modifier::new().fill_max_width()).child((
        Box(Modifier::new().fill_max_width().background(bg))
            .child(Row(Modifier::new().fill_max_width()).child(cells)),
        divider(),
    ))
}

fn render_inlines(
    inlines: &[Inline],
    style: InlineStyle,
    on_link: Rc<dyn Fn(String)>,
) -> Vec<View> {
    let mut views = Vec::new();

    for inline in inlines {
        match inline {
            Inline::Text(text) => {
                for word in text.split_inclusive(' ') {
                    views.push(Text(word.to_string()).size(style.size).color(style.color));
                }
            }

            Inline::Code(text) => {
                views.push(
                    Box(Modifier::new()
                        .background(theme().surface_container_high)
                        .clip_rounded(7.0)
                        .border(1.0, theme().outline_variant, 7.0)
                        .padding_values(PaddingValues {
                            left: 5.0,
                            right: 5.0,
                            top: 2.0,
                            bottom: 2.0,
                        }))
                    .child(
                        Text(text.clone())
                            .font_family("monospace")
                            .size((style.size - 1.5).max(11.0))
                            .color(theme().primary),
                    ),
                );
            }

            Inline::Strong(children) => {
                views.extend(render_inlines(
                    children,
                    InlineStyle {
                        size: style.size + 0.5,
                        color: theme().on_surface,
                    },
                    on_link.clone(),
                ));
            }

            Inline::Emphasis(children) | Inline::Strike(children) => {
                views.extend(render_inlines(
                    children,
                    InlineStyle {
                        size: style.size,
                        color: theme().on_surface_variant,
                    },
                    on_link.clone(),
                ));
            }

            Inline::Link { label, url } => {
                let url_clone = url.clone();
                let handler = on_link.clone();
                let children = render_inlines(
                    label,
                    InlineStyle {
                        size: style.size,
                        color: theme().primary,
                    },
                    on_link.clone(),
                );

                views.push(
                    Box(Modifier::new()
                        .clickable()
                        .on_click(move || handler(url_clone.clone())))
                    .child(Row(Modifier::new()).child(children)),
                );
            }

            Inline::Image { alt, url } => {
                let alt_text = if alt.is_empty() {
                    "image".to_string()
                } else {
                    plain_text(alt)
                };

                let url_clone = url.clone();
                let handler = on_link.clone();

                views.push(
                    Box(Modifier::new()
                        .background(theme().surface_container)
                        .clip_rounded(10.0)
                        .border(1.0, theme().outline_variant, 10.0)
                        .padding_values(PaddingValues {
                            left: 8.0,
                            right: 8.0,
                            top: 5.0,
                            bottom: 5.0,
                        })
                        .clickable()
                        .on_click(move || handler(url_clone.clone())))
                    .child(
                        Text(format!("🖼 {alt_text}"))
                            .size(style.size - 1.0)
                            .color(theme().primary),
                    ),
                );
            }

            Inline::SoftBreak | Inline::HardBreak => {
                views.push(Text(" ".to_string()).size(style.size).color(style.color));
            }

            Inline::TaskMarker(checked) => {
                views.push(
                    Text(if *checked { "☑ " } else { "☐ " }.to_string())
                        .size(style.size)
                        .color(theme().primary),
                );
            }
        }
    }

    views
}

fn divider() -> View {
    HorizontalDivider(DividerConfig::default())
}

fn plain_text(inlines: &[Inline]) -> String {
    let mut out = String::new();
    for inline in inlines {
        match inline {
            Inline::Text(s) | Inline::Code(s) => out.push_str(s),
            Inline::Strong(xs) | Inline::Emphasis(xs) | Inline::Strike(xs) => {
                out.push_str(&plain_text(xs))
            }
            Inline::Link { label, .. } => out.push_str(&plain_text(label)),
            Inline::Image { alt, .. } => out.push_str(&plain_text(alt)),
            Inline::SoftBreak | Inline::HardBreak => out.push(' '),
            Inline::TaskMarker(c) => out.push_str(if *c { "[x] " } else { "[ ] " }),
        }
    }
    out
}

fn intersperse_vertical(children: Vec<View>, gap: f32) -> Vec<View> {
    let mut out = Vec::new();
    for (idx, child) in children.into_iter().enumerate() {
        if idx > 0 {
            out.push(vspace(gap));
        }
        out.push(child);
    }
    out
}

fn vspace(dp: f32) -> View {
    Box(Modifier::new().height(dp).width(1.0))
}
