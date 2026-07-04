use crate::latex::render_math_string;
use pulldown_cmark::{
    Alignment as MdAlignment, CodeBlockKind, Event, HeadingLevel, LinkType, MetadataBlockKind,
    Options, Parser, Tag, TagEnd, TextMergeStream,
};
use repose_core::{PaddingValues, TextDecoration, clipboard::copy_to_clipboard, prelude::*};
use repose_material::material3::{DividerConfig, HorizontalDivider, IconButton, IconButtonConfig};
use repose_material::{Icon, material_symbols};
use repose_ui::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::LazyLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

material_symbols! {
    CONTENT_COPY : '\u{E14D}',
}

static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(|| SyntaxSet::load_defaults_newlines());
static THEME_SET: LazyLock<ThemeSet> = LazyLock::new(|| ThemeSet::load_defaults());

#[derive(Debug, Clone)]
enum Block {
    Heading {
        level: u8,
        id: Option<String>,
        classes: Vec<String>,
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
    DefinitionList(Vec<DefinitionListEntry>),
    Rule,
    Html(String),
    HtmlBlock(String),
    MetadataBlock {
        kind: MetadataBlockKind,
        body: String,
    },
    FootnoteDefinition {
        label: String,
        blocks: Vec<Block>,
    },
    DisplayMath(String),
}

#[derive(Debug, Clone)]
struct ListItem {
    task: Option<bool>,
    blocks: Vec<Block>,
}

#[derive(Debug, Clone)]
struct DefinitionListEntry {
    title: Vec<Inline>,
    definitions: Vec<Vec<Block>>,
}

#[derive(Debug, Clone)]
enum Inline {
    Text(String),
    Code(String),
    Html(String),
    InlineHtml(String),
    Strong(Vec<Inline>),
    Emphasis(Vec<Inline>),
    Strike(Vec<Inline>),
    Superscript(Vec<Inline>),
    Subscript(Vec<Inline>),
    Link {
        link_type: LinkType,
        label: Vec<Inline>,
        url: String,
        #[allow(dead_code)]
        title: String,
    },
    Image {
        #[allow(dead_code)]
        link_type: LinkType,
        label: Vec<Inline>,
        url: String,
        title: String,
    },
    InlineMath(String),
    FootnoteReference(String),
    SoftBreak,
    HardBreak,
    TaskMarker(bool),
}

#[derive(Clone, Copy)]
struct InlineStyle {
    size: f32,
    color: Color,
}

fn parse_markdown_cached<'a>(src: &'a str) -> Rc<Vec<Block>> {
    let cache = remember_with_key("renedown:parse_cache", || {
        Rc::new(RefCell::new((String::new(), Rc::new(Vec::new()))))
    });
    let mut cache = cache.borrow_mut();
    if cache.0 != src {
        cache.0 = src.to_string();
        cache.1 = Rc::new(parse_markdown(src));
    }
    cache.1.clone()
}

#[allow(non_snake_case)]
pub fn MarkdownDocument(src: &str, on_link: Rc<dyn Fn(String)>) -> View {
    let blocks = parse_markdown_cached(src);
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
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    options.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
    options.insert(Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS);
    options.insert(Options::ENABLE_MATH);
    options.insert(Options::ENABLE_DEFINITION_LIST);
    options.insert(Options::ENABLE_SUPERSCRIPT);
    options.insert(Options::ENABLE_SUBSCRIPT);
    options.insert(Options::ENABLE_WIKILINKS);
    options.insert(Options::ENABLE_GFM);

    let events: Vec<Event<'_>> = TextMergeStream::new(Parser::new_ext(src, options)).collect();
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
                if !inlines.is_empty() {
                    blocks.push(Block::Paragraph(inlines));
                }
            }

            Event::Start(Tag::Heading {
                level, id, classes, ..
            }) => {
                let level = heading_level_to_u8(*level);
                let id = id.as_ref().map(|s| s.to_string());
                let classes = classes.iter().map(|c| c.to_string()).collect();
                *pos += 1;
                let inlines = parse_inlines(events, pos);
                consume_end(events, pos, |end| matches!(end, TagEnd::Heading(_)));
                blocks.push(Block::Heading {
                    level,
                    id,
                    classes,
                    inlines,
                });
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

            Event::Start(Tag::DefinitionList) => {
                *pos += 1;
                let entries = parse_definition_list(events, pos);
                consume_end(events, pos, |end| matches!(end, TagEnd::DefinitionList));
                blocks.push(Block::DefinitionList(entries));
            }

            Event::Rule => {
                *pos += 1;
                blocks.push(Block::Rule);
            }

            Event::Start(Tag::HtmlBlock) => {
                *pos += 1;
                let mut buf = String::new();
                while let Some(ev) = events.get(*pos) {
                    match ev {
                        Event::End(TagEnd::HtmlBlock) => {
                            *pos += 1;
                            break;
                        }
                        Event::Html(h) | Event::InlineHtml(h) | Event::Text(h) => {
                            buf.push_str(h);
                            *pos += 1;
                        }
                        Event::SoftBreak | Event::HardBreak => {
                            buf.push('\n');
                            *pos += 1;
                        }
                        _ => {
                            *pos += 1;
                        }
                    }
                }
                blocks.push(Block::HtmlBlock(buf));
            }

            Event::Html(html) => {
                *pos += 1;
                blocks.push(Block::Html(html.to_string()));
            }

            Event::Start(Tag::MetadataBlock(kind)) => {
                let kind = *kind;
                *pos += 1;
                let mut body = String::new();
                while let Some(ev) = events.get(*pos) {
                    match ev {
                        Event::End(TagEnd::MetadataBlock(_)) => {
                            *pos += 1;
                            break;
                        }
                        Event::Text(t) => {
                            body.push_str(t);
                            *pos += 1;
                        }
                        Event::SoftBreak | Event::HardBreak => {
                            body.push('\n');
                            *pos += 1;
                        }
                        _ => {
                            *pos += 1;
                        }
                    }
                }
                blocks.push(Block::MetadataBlock { kind, body });
            }

            Event::Start(Tag::FootnoteDefinition(label)) => {
                let label = label.to_string();
                *pos += 1;
                let inner = parse_blocks(events, pos);
                consume_end(events, pos, |end| matches!(end, TagEnd::FootnoteDefinition));
                blocks.push(Block::FootnoteDefinition {
                    label,
                    blocks: inner,
                });
            }

            Event::DisplayMath(m) => {
                *pos += 1;
                blocks.push(Block::DisplayMath(m.to_string()));
            }

            Event::Text(_)
            | Event::Code(_)
            | Event::InlineMath(_)
            | Event::TaskListMarker(_)
            | Event::SoftBreak
            | Event::HardBreak
            | Event::InlineHtml(_)
            | Event::FootnoteReference(_)
            | Event::Start(Tag::Strong)
            | Event::Start(Tag::Emphasis)
            | Event::Start(Tag::Strikethrough)
            | Event::Start(Tag::Superscript)
            | Event::Start(Tag::Subscript)
            | Event::Start(Tag::Link { .. })
            | Event::Start(Tag::Image { .. }) => {
                let inlines = parse_inlines(events, pos);
                if !inlines.is_empty() {
                    blocks.push(Block::Paragraph(inlines));
                }
            }

            Event::End(TagEnd::Paragraph) => {
                *pos += 1;
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
                head = parse_table_cells(events, pos, |end| matches!(end, TagEnd::TableHead));
            }
            Event::Start(Tag::TableRow) => {
                *pos += 1;
                let row = parse_table_cells(events, pos, |end| matches!(end, TagEnd::TableRow));
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

fn parse_table_cells(
    events: &[Event<'_>],
    pos: &mut usize,
    end_pred: impl Fn(&TagEnd) -> bool,
) -> Vec<Vec<Inline>> {
    let mut cells = Vec::new();

    while let Some(event) = events.get(*pos) {
        match event {
            Event::Start(Tag::TableCell) => {
                *pos += 1;
                let cell = parse_inlines(events, pos);
                consume_end(events, pos, |end| matches!(end, TagEnd::TableCell));
                cells.push(cell);
            }
            Event::End(end) if end_pred(end) => break,
            _ => *pos += 1,
        }
    }

    cells
}

fn parse_definition_list(events: &[Event<'_>], pos: &mut usize) -> Vec<DefinitionListEntry> {
    let mut entries = Vec::new();

    while let Some(event) = events.get(*pos) {
        match event {
            Event::Start(Tag::DefinitionListTitle) => {
                *pos += 1;
                let title = parse_inlines(events, pos);
                consume_end(events, pos, |end| {
                    matches!(end, TagEnd::DefinitionListTitle)
                });
                entries.push(DefinitionListEntry {
                    title,
                    definitions: Vec::new(),
                });
            }
            Event::Start(Tag::DefinitionListDefinition) => {
                *pos += 1;
                let def = parse_blocks(events, pos);
                consume_end(events, pos, |end| {
                    matches!(end, TagEnd::DefinitionListDefinition)
                });
                if let Some(last) = entries.last_mut() {
                    last.definitions.push(def);
                } else {
                    entries.push(DefinitionListEntry {
                        title: Vec::new(),
                        definitions: vec![def],
                    });
                }
            }
            Event::End(TagEnd::DefinitionList) => break,
            _ => {
                *pos += 1;
            }
        }
    }

    entries
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

            Event::Html(t) => {
                inlines.push(Inline::Html(t.to_string()));
                *pos += 1;
            }

            Event::InlineHtml(t) => {
                inlines.push(Inline::InlineHtml(t.to_string()));
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

            Event::InlineMath(m) => {
                inlines.push(Inline::InlineMath(m.to_string()));
                *pos += 1;
            }

            Event::FootnoteReference(l) => {
                inlines.push(Inline::FootnoteReference(l.to_string()));
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

            Event::Start(Tag::Superscript) => {
                *pos += 1;
                let inner = parse_inlines(events, pos);
                consume_end(events, pos, |end| matches!(end, TagEnd::Superscript));
                inlines.push(Inline::Superscript(inner));
            }

            Event::Start(Tag::Subscript) => {
                *pos += 1;
                let inner = parse_inlines(events, pos);
                consume_end(events, pos, |end| matches!(end, TagEnd::Subscript));
                inlines.push(Inline::Subscript(inner));
            }

            Event::Start(Tag::Link {
                link_type,
                dest_url,
                title,
                ..
            }) => {
                let link_type = *link_type;
                let url = dest_url.to_string();
                let title = title.to_string();
                *pos += 1;
                let label = parse_inlines(events, pos);
                consume_end(events, pos, |end| matches!(end, TagEnd::Link));
                inlines.push(Inline::Link {
                    link_type,
                    label,
                    url,
                    title,
                });
            }

            Event::Start(Tag::Image {
                link_type,
                dest_url,
                title,
                ..
            }) => {
                let link_type = *link_type;
                let url = dest_url.to_string();
                let title = title.to_string();
                *pos += 1;
                let label = parse_inlines(events, pos);
                consume_end(events, pos, |end| matches!(end, TagEnd::Image));
                inlines.push(Inline::Image {
                    link_type,
                    label,
                    url,
                    title,
                });
            }

            Event::End(_) => break,

            _ => break,
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
        Block::Heading {
            level,
            id,
            classes,
            inlines,
        } => render_heading(*level, id.as_deref(), classes, inlines, on_link),

        Block::Paragraph(inlines) => render_rich_text(
            inlines,
            InlineStyle {
                size: 15.0,
                color: theme().on_surface,
            },
            on_link,
        ),

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
                    Box(Modifier::new().width(5.0).background(theme().primary)),
                    Box(Modifier::new().flex_grow(1.0).padding(14.0))
                        .child(Column(Modifier::new().fill_max_width()).child(children)),
                )),
            )
        }

        Block::CodeBlock { lang, code } => {
            let code_text = code.trim_end().to_string();

            Box(Modifier::new()
                .fill_max_width()
                .background(theme().surface_container)
                .clip_rounded(16.0)
                .border(1.0, theme().outline_variant, 16.0))
            .child(
                Column(Modifier::new().fill_max_width()).child((
                    Row(Modifier::new()
                        .fill_max_width()
                        .padding_values(PaddingValues {
                            left: 14.0,
                            right: 8.0,
                            top: 8.0,
                            bottom: 4.0,
                        }))
                        .child((
                            lang.as_ref().map_or(
                                Box(Modifier::new()),
                                |l| {
                                    Box(Modifier::new()
                                        .background(theme().secondary_container)
                                        .clip_rounded(999.0)
                                        .padding_values(PaddingValues {
                                            left: 8.0,
                                            right: 8.0,
                                            top: 4.0,
                                            bottom: 4.0,
                                        })
                                        .align_items(AlignItems::CENTER)
                                        .justify_content(JustifyContent::CENTER))
                                    .child(
                                        Text(l.clone())
                                            .font_family("monospace")
                                            .size(11.0)
                                            .color(theme().on_secondary_container),
                                    )
                                },
                            ),
                            Spacer(),
                            IconButton(
                                Icon(Symbols::CONTENT_COPY).size(18.0).color(theme().on_surface.with_alpha_f32(0.6)),
                                {
                                    let code_copy = code_text.clone();
                                    move || copy_to_clipboard(&code_copy)
                                },
                                IconButtonConfig {
                                    container_size: Some(32.0),
                                    colors: repose_material::material3::IconButtonColors {
                                        container_color: Color::TRANSPARENT,
                                        content_color: theme().on_surface.with_alpha_f32(0.6),
                                        disabled_container_color: Color::TRANSPARENT,
                                        disabled_content_color: theme().on_surface.with_alpha_f32(0.38),
                                    },
                                    ..Default::default()
                                },
                            ),
                        )),
                    Box(Modifier::new().padding(14.0)).child(highlight_code(&code_text, lang.as_deref())),
                )),
            )
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

            // Left padding creates visual nesting for nested lists.
            Box(Modifier::new()
                .fill_max_width()
                .padding_values(PaddingValues {
                    left: 20.0,
                    right: 0.0,
                    top: 0.0,
                    bottom: 0.0,
                }))
            .child(Column(Modifier::new().fill_max_width()).child(rendered))
        }

        Block::Table {
            alignments,
            head,
            rows,
        } => render_table(alignments, head, rows, on_link),

        Block::DefinitionList(entries) => render_definition_list(entries, on_link),

        Block::Rule => divider(),

        Block::Html(html) => html_pre(html, theme().surface_container),
        Block::HtmlBlock(html) => html_pre(html, theme().surface_container_high),

        Block::MetadataBlock { kind, body } => {
            let label = match kind {
                MetadataBlockKind::YamlStyle => "yaml",
                MetadataBlockKind::PlusesStyle => "toml",
            };
            Box(Modifier::new()
                .fill_max_width()
                .background(theme().surface_container)
                .clip_rounded(12.0)
                .border(1.0, theme().outline_variant, 12.0)
                .padding(12.0))
            .child(
                Column(Modifier::new().fill_max_width()).child((
                    Text(format!("metadata ({label})"))
                        .size(11.0)
                        .color(theme().primary),
                    vspace(4.0),
                    Text(body.trim().to_string())
                        .font_family("monospace")
                        .size(12.0)
                        .color(theme().on_surface_variant),
                )),
            )
        }

        Block::FootnoteDefinition { label, blocks } => {
            let children = intersperse_vertical(
                blocks
                    .iter()
                    .map(|b| render_block(b, on_link.clone()))
                    .collect(),
                6.0,
            );
            Box(Modifier::new()
                .fill_max_width()
                .background(theme().surface_container)
                .clip_rounded(12.0)
                .border(1.0, theme().outline_variant, 12.0)
                .padding(12.0))
            .child(
                Column(Modifier::new().fill_max_width()).child((
                    Text(format!("[^{label}]"))
                        .size(11.0)
                        .color(theme().primary),
                    vspace(4.0),
                    Column(Modifier::new().fill_max_width()).child(children),
                )),
            )
        }

        Block::DisplayMath(m) => Box(Modifier::new()
            .fill_max_width()
            .background(theme().surface_container_high)
            .clip_rounded(12.0)
            .border(1.0, theme().outline_variant, 12.0)
            .padding_values(PaddingValues {
                left: 14.0,
                right: 14.0,
                top: 12.0,
                bottom: 12.0,
            }))
        .child(
            Row(Modifier::new().fill_max_width().align_items(AlignItems::FLEX_START))
                .child((
                    Text("𝑓  ").font_family("monospace").size(14.0).color(theme().on_surface),
                    render_math_string(m.trim(), 14.0),
                )),
        ),
    }
}

fn render_heading(
    level: u8,
    id: Option<&str>,
    classes: &[String],
    inlines: &[Inline],
    on_link: Rc<dyn Fn(String)>,
) -> View {
    let (size, color) = match level {
        1 => (31.0, theme().primary),
        2 => (25.0, theme().on_surface),
        3 => (21.0, theme().on_surface),
        4 => (18.0, theme().on_surface),
        _ => (15.0, theme().on_surface_variant),
    };

    let content = render_rich_text(inlines, InlineStyle { size, color }, on_link);

    let attrs = if id.is_some() || !classes.is_empty() {
        let mut txt = String::new();
        if let Some(i) = id {
            txt.push('#');
            txt.push_str(i);
        }
        for c in classes {
            if !txt.is_empty() {
                txt.push(' ');
            }
            txt.push('.');
            txt.push_str(c);
        }
        Some(
            Text(txt)
                .font_family("monospace")
                .size(10.0)
                .color(theme().on_surface_variant),
        )
    } else {
        None
    };

    let mut children: Vec<View> = vec![content];
    if let Some(a) = attrs {
        children.push(vspace(2.0));
        children.push(a);
    }
    if level <= 2 {
        children.push(vspace(8.0));
        children.push(divider());
    }
    Column(Modifier::new().fill_max_width()).child(children)
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
        Box(Modifier::new().flex_grow(1.0))
            .child(Column(Modifier::new().fill_max_width()).child(rendered)),
    ))
}

fn render_table(
    alignments: &[MdAlignment],
    head: &[Vec<Inline>],
    rows: &[Vec<Vec<Inline>>],
    on_link: Rc<dyn Fn(String)>,
) -> View {
    let mut row_views = Vec::new();

    if !head.is_empty() {
        row_views.push(render_table_row(
            head,
            alignments,
            true,
            false,
            on_link.clone(),
        ));
    }

    for (i, row) in rows.iter().enumerate() {
        row_views.push(render_table_row(
            row,
            alignments,
            false,
            i % 2 == 1,
            on_link.clone(),
        ));
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
    alignments: &[MdAlignment],
    header: bool,
    striped: bool,
    on_link: Rc<dyn Fn(String)>,
) -> View {
    let cells: Vec<View> = row
        .iter()
        .enumerate()
        .map(|(idx, cell)| {
            let style = InlineStyle {
                size: if header { 14.0 } else { 13.5 },
                color: if header {
                    theme().primary
                } else {
                    theme().on_surface
                },
            };

            let justify = cell_justify(alignments.get(idx));

            Box(Modifier::new().flex_grow(1.0).flex_basis(0.0).padding(10.0)).child(
                Row(Modifier::new().fill_max_width().justify_content(justify))
                    .child(render_rich_text(cell, style, on_link.clone())),
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

fn render_definition_list(entries: &[DefinitionListEntry], on_link: Rc<dyn Fn(String)>) -> View {
    let items: Vec<View> = entries
        .iter()
        .map(|entry| {
            let title = render_rich_text(
                &entry.title,
                InlineStyle {
                    size: 15.0,
                    color: theme().primary,
                },
                on_link.clone(),
            );

            let defs: Vec<View> = entry
                .definitions
                .iter()
                .map(|def_blocks| {
                    let def_children = intersperse_vertical(
                        def_blocks
                            .iter()
                            .map(|b| render_block(b, on_link.clone()))
                            .collect(),
                        6.0,
                    );
                    Row(Modifier::new().fill_max_width()).child((
                        Box(Modifier::new().width(18.0))
                            .child(Text("—".to_string()).size(14.0).color(theme().outline)),
                        Box(Modifier::new().flex_grow(1.0))
                            .child(Column(Modifier::new().fill_max_width()).child(def_children)),
                    ))
                })
                .collect();

            let mut column_children: Vec<View> = vec![title, vspace(4.0)];
            for (i, def) in defs.into_iter().enumerate() {
                if i > 0 {
                    column_children.push(vspace(4.0));
                }
                column_children.push(def);
            }

            Column(Modifier::new().fill_max_width()).child(column_children)
        })
        .collect();

    let rendered = intersperse_vertical(items, 10.0);

    Box(Modifier::new()
        .fill_max_width()
        .background(theme().surface_container_low)
        .clip_rounded(14.0)
        .border(1.0, theme().outline_variant, 14.0)
        .padding(14.0))
    .child(Column(Modifier::new().fill_max_width()).child(rendered))
}

fn html_pre(html: &str, bg: Color) -> View {
    Box(Modifier::new()
        .fill_max_width()
        .background(bg)
        .clip_rounded(12.0)
        .border(1.0, theme().outline_variant, 12.0)
        .padding(12.0))
    .child(
        Text(html.trim().to_string())
            .font_family("monospace")
            .size(12.0)
            .color(theme().on_surface_variant),
    )
}

fn render_rich_text(inlines: &[Inline], style: InlineStyle, on_link: Rc<dyn Fn(String)>) -> View {
    let mut rows: Vec<View> = split_hard_breaks(inlines)
        .into_iter()
        .map(|line| {
            FlowRow(Modifier::new().fill_max_width()).child(render_inlines(
                &line,
                style,
                on_link.clone(),
            ))
        })
        .collect();

    if rows.is_empty() {
        Box(Modifier::new())
    } else if rows.len() == 1 {
        rows.remove(0)
    } else {
        Column(Modifier::new().fill_max_width()).child(intersperse_vertical(rows, 4.0))
    }
}

fn render_inlines(inlines: &[Inline], base: InlineStyle, on_link: Rc<dyn Fn(String)>) -> Vec<View> {
    let mut views = Vec::new();
    let mut text_buf = String::new();
    let mut spans: Vec<TextSpan> = Vec::new();

    let flush = |views: &mut Vec<View>, text_buf: &mut String, spans: &mut Vec<TextSpan>| {
        if !text_buf.is_empty() {
            let text = std::mem::take(text_buf);
            let s = std::mem::take(spans);
            views.push(
                AnnotatedText(AnnotatedString {
                    text,
                    spans: s.into(),
                })
                .size(base.size)
                .color(base.color),
            );
        }
    };

    for inline in inlines {
        match inline {
            Inline::Text(t) => {
                text_buf.push_str(t);
            }

            Inline::Code(t) => {
                flush(&mut views, &mut text_buf, &mut spans);
                views.push(inline_code_chip(t, base.size, theme().on_surface));
            }

            Inline::Html(t) | Inline::InlineHtml(t) => {
                flush(&mut views, &mut text_buf, &mut spans);
                views.push(inline_html_chip(t, (base.size - 2.0).max(10.0)));
            }

            Inline::Strong(children) => {
                let start = text_buf.len();
                let child_style = InlineStyle {
                    size: base.size + 0.5,
                    color: theme().on_surface,
                };
                accumulate_text_inlines(children, child_style, &mut text_buf, &mut spans, &on_link);
                if text_buf.len() > start {
                    spans.push(TextSpan {
                        start,
                        end: text_buf.len(),
                        style: SpanStyle {
                            font_size: Some(child_style.size),
                            color: Some(child_style.color),
                            ..SpanStyle::default()
                        },
                        url: None,
                    });
                }
            }

            Inline::Emphasis(children) => {
                let start = text_buf.len();
                accumulate_text_inlines(children, base, &mut text_buf, &mut spans, &on_link);
                if text_buf.len() > start {
                    spans.push(TextSpan {
                        start,
                        end: text_buf.len(),
                        style: SpanStyle::default(),
                        url: None,
                    });
                }
            }

            Inline::Strike(children) => {
                let start = text_buf.len();
                accumulate_text_inlines(children, base, &mut text_buf, &mut spans, &on_link);
                if text_buf.len() > start {
                    spans.push(TextSpan {
                        start,
                        end: text_buf.len(),
                        style: SpanStyle {
                            text_decoration: Some(TextDecoration {
                                strikethrough: true,
                                ..TextDecoration::default()
                            }),
                            ..SpanStyle::default()
                        },
                        url: None,
                    });
                }
            }

            Inline::Superscript(children) => {
                flush(&mut views, &mut text_buf, &mut spans);
                let child_style = InlineStyle {
                    size: (base.size * 0.65).max(9.0),
                    color: theme().on_surface,
                };
                let text = plain_text(children);
                views.push(
                    Box(Modifier::new().translate(0.0, -base.size * 0.15))
                        .child(Text(text).size(child_style.size).color(child_style.color)),
                );
            }

            Inline::Subscript(children) => {
                flush(&mut views, &mut text_buf, &mut spans);
                let child_style = InlineStyle {
                    size: (base.size * 0.65).max(9.0),
                    color: theme().on_surface,
                };
                let text = plain_text(children);
                views.push(
                    Box(Modifier::new().translate(0.0, base.size * 0.7))
                        .child(Text(text).size(child_style.size).color(child_style.color)),
                );
            }

            Inline::Link {
                link_type,
                label,
                url,
                title: _,
            } => {
                flush(&mut views, &mut text_buf, &mut spans);
                let url_clone = url.clone();
                let handler = on_link.clone();
                let mut children = render_inlines(
                    label,
                    InlineStyle {
                        size: base.size,
                        color: theme().primary,
                    },
                    on_link.clone(),
                );

                if matches!(link_type, LinkType::WikiLink { .. }) {
                    children.insert(
                        0,
                        Text("[[".to_string())
                            .size(base.size)
                            .color(theme().outline),
                    );
                    children.push(
                        Text("]]".to_string())
                            .size(base.size)
                            .color(theme().outline),
                    );
                }

                views.push(
                    Box(Modifier::new()
                        .clickable()
                        .on_click(move || handler(url_clone.clone())))
                    .child(FlowRow(Modifier::new()).child(children)),
                );
            }

            Inline::Image {
                link_type: _,
                label,
                url,
                title,
            } => {
                flush(&mut views, &mut text_buf, &mut spans);
                let alt = if label.is_empty() {
                    "image".to_string()
                } else {
                    plain_text(label)
                };
                let hint = if title.is_empty() {
                    String::new()
                } else {
                    format!(" ({title})")
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
                        Text(format!("🖼 {alt}{hint}"))
                            .size((base.size - 1.0).max(11.0))
                            .color(theme().primary),
                    ),
                );
            }

            Inline::InlineMath(m) => {
                flush(&mut views, &mut text_buf, &mut spans);
                let math_size = (base.size - 1.0).max(11.0);
                views.push(
                    Box(Modifier::new()
                        .background(theme().surface_container_high)
                        .clip_rounded(6.0)
                        .padding_values(PaddingValues {
                            left: 5.0,
                            right: 5.0,
                            top: 1.0,
                            bottom: 1.0,
                        }))
                    .child(render_math_string(m, math_size)),
                );
            }

            Inline::FootnoteReference(label) => {
                flush(&mut views, &mut text_buf, &mut spans);
                views.push(
                    Box(Modifier::new()
                        .background(theme().secondary_container)
                        .clip_rounded(999.0)
                        .padding_values(PaddingValues {
                            left: 4.0,
                            right: 4.0,
                            top: 1.0,
                            bottom: 1.0,
                        }))
                    .child(
                        Text(format!("[{label}]"))
                            .size(10.0)
                            .color(theme().on_secondary_container),
                    ),
                );
            }

            Inline::SoftBreak => {
                text_buf.push(' ');
            }

            Inline::HardBreak => {
                flush(&mut views, &mut text_buf, &mut spans);
                views.push(Box(Modifier::new().fill_max_width().height(0.0)));
            }

            Inline::TaskMarker(checked) => {
                flush(&mut views, &mut text_buf, &mut spans);
                views.push(
                    Text(if *checked { "☑ " } else { "☐ " }.to_string())
                        .size(base.size)
                        .color(theme().primary),
                );
            }
        }
    }

    flush(&mut views, &mut text_buf, &mut spans);
    views
}

/// Recursively accumulate text-style inlines into the AnnotatedString builder.
/// Non-text inlines encountered at any depth are converted to plain text
/// (they're extremely rare inside formatting like **bold `code`**).
fn accumulate_text_inlines(
    inlines: &[Inline],
    style: InlineStyle,
    text_buf: &mut String,
    spans: &mut Vec<TextSpan>,
    on_link: &Rc<dyn Fn(String)>,
) {
    for inline in inlines {
        match inline {
            Inline::Text(t) => text_buf.push_str(t),
            Inline::Strong(children) => {
                let start = text_buf.len();
                accumulate_text_inlines(children, style, text_buf, spans, on_link);
                if text_buf.len() > start {
                    spans.push(TextSpan {
                        start,
                        end: text_buf.len(),
                        style: SpanStyle {
                            ..SpanStyle::default()
                        },
                        url: None,
                    });
                }
            }
            Inline::Emphasis(children) => {
                let start = text_buf.len();
                accumulate_text_inlines(children, style, text_buf, spans, on_link);
                if text_buf.len() > start {
                    spans.push(TextSpan {
                        start,
                        end: text_buf.len(),
                        style: SpanStyle::default(),
                        url: None,
                    });
                }
            }
            Inline::Strike(children) => {
                let start = text_buf.len();
                accumulate_text_inlines(children, style, text_buf, spans, on_link);
                if text_buf.len() > start {
                    spans.push(TextSpan {
                        start,
                        end: text_buf.len(),
                        style: SpanStyle {
                            text_decoration: Some(TextDecoration {
                                strikethrough: true,
                                ..TextDecoration::default()
                            }),
                            ..SpanStyle::default()
                        },
                        url: None,
                    });
                }
            }
            Inline::Superscript(children) | Inline::Subscript(children) => {
                text_buf.push_str(&plain_text(children));
            }
            // Non-text inlines inside formatting: fall back to plain text
            Inline::Code(t) | Inline::Html(t) | Inline::InlineHtml(t) | Inline::InlineMath(t) => {
                text_buf.push_str(t)
            }
            Inline::Link { label, .. } | Inline::Image { label, .. } => {
                text_buf.push_str(&plain_text(label));
            }
            Inline::FootnoteReference(l) => {
                text_buf.push('[');
                text_buf.push_str(l);
                text_buf.push(']');
            }
            Inline::SoftBreak => text_buf.push(' '),
            Inline::HardBreak => text_buf.push('\n'),
            Inline::TaskMarker(c) => {
                text_buf.push_str(if *c { "[x] " } else { "[ ] " });
            }
        }
    }
}

fn inline_code_chip(text: &str, base_size: f32, color: Color) -> View {
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
        Text(text.to_string())
            .font_family("monospace")
            .size((base_size - 1.5).max(11.0))
            .color(color),
    )
}

fn inline_html_chip(text: &str, size: f32) -> View {
    Box(Modifier::new()
        .background(theme().surface_container)
        .clip_rounded(4.0)
        .padding_values(PaddingValues {
            left: 4.0,
            right: 4.0,
            top: 1.0,
            bottom: 1.0,
        }))
    .child(
        Text(text.to_string())
            .font_family("monospace")
            .size(size)
            .color(theme().on_surface_variant),
    )
}

fn divider() -> View {
    HorizontalDivider(DividerConfig::default())
}

fn plain_text(inlines: &[Inline]) -> String {
    let mut out = String::new();
    for inline in inlines {
        match inline {
            Inline::Text(s)
            | Inline::Code(s)
            | Inline::Html(s)
            | Inline::InlineHtml(s)
            | Inline::InlineMath(s) => out.push_str(s),
            Inline::Strong(xs)
            | Inline::Emphasis(xs)
            | Inline::Strike(xs)
            | Inline::Superscript(xs)
            | Inline::Subscript(xs) => out.push_str(&plain_text(xs)),
            Inline::Link { label, .. } => out.push_str(&plain_text(label)),
            Inline::Image { label, .. } => out.push_str(&plain_text(label)),
            Inline::FootnoteReference(l) => {
                out.push('[');
                out.push_str(l);
                out.push(']');
            }
            Inline::SoftBreak | Inline::HardBreak => out.push(' '),
            Inline::TaskMarker(c) => out.push_str(if *c { "[x] " } else { "[ ] " }),
        }
    }
    out
}

fn split_hard_breaks(inlines: &[Inline]) -> Vec<Vec<Inline>> {
    let mut lines: Vec<Vec<Inline>> = vec![Vec::new()];

    for inline in inlines {
        match inline {
            Inline::HardBreak => lines.push(Vec::new()),
            other => lines.last_mut().unwrap().push(other.clone()),
        }
    }

    lines
}

fn cell_justify(alignment: Option<&MdAlignment>) -> JustifyContent {
    match alignment.copied().unwrap_or(MdAlignment::None) {
        MdAlignment::Left | MdAlignment::None => JustifyContent::FLEX_START,
        MdAlignment::Center => JustifyContent::CENTER,
        MdAlignment::Right => JustifyContent::FLEX_END,
    }
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

fn highlight_code(code: &str, lang: Option<&str>) -> View {
    let syntax = lang
        .and_then(|l| {
            SYNTAX_SET
                .find_syntax_by_token(l)
                .or_else(|| SYNTAX_SET.find_syntax_by_name(l))
                .or_else(|| SYNTAX_SET.find_syntax_by_extension(l))
        })
        .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());

    let theme_name = if theme().on_surface.0 < 128 {
        "base16-ocean.light"
    } else {
        "base16-ocean.dark"
    };
    let theme = &THEME_SET.themes[theme_name];
    let mut highlighter = HighlightLines::new(syntax, theme);

    let mut builder = repose_core::text::AnnotatedStringBuilder::new();
    for line in syntect::util::LinesWithEndings::from(code) {
        let regions = highlighter
            .highlight_line(line, &SYNTAX_SET)
            .unwrap_or_default();
        for (style, text) in &regions {
            let c = repose_core::Color(
                style.foreground.r,
                style.foreground.g,
                style.foreground.b,
                style.foreground.a,
            );
            builder.push_with_style(text, SpanStyle::default().color(c));
        }
    }

    AnnotatedText(builder.build())
        .font_family("monospace")
        .size(15.0)
}



#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> Vec<Block> {
        let mut opts = Options::empty();
        opts.insert(Options::ENABLE_TABLES);
        opts.insert(Options::ENABLE_FOOTNOTES);
        opts.insert(Options::ENABLE_STRIKETHROUGH);
        opts.insert(Options::ENABLE_TASKLISTS);
        opts.insert(Options::ENABLE_SMART_PUNCTUATION);
        opts.insert(Options::ENABLE_HEADING_ATTRIBUTES);
        opts.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
        opts.insert(Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS);
        opts.insert(Options::ENABLE_MATH);
        opts.insert(Options::ENABLE_DEFINITION_LIST);
        opts.insert(Options::ENABLE_SUPERSCRIPT);
        opts.insert(Options::ENABLE_SUBSCRIPT);
        opts.insert(Options::ENABLE_WIKILINKS);
        opts.insert(Options::ENABLE_GFM);

        let events: Vec<Event<'_>> = TextMergeStream::new(Parser::new_ext(src, opts)).collect();
        let mut pos = 0;
        parse_blocks(&events, &mut pos)
    }

    #[test]
    fn nested_lists_are_nested() {
        let src = "- Outer\n  - Inner\n- Sibling\n";
        let blocks = parse(src);
        assert_eq!(blocks.len(), 1);
        let list = &blocks[0];
        if let Block::List { items, .. } = list {
            assert_eq!(items.len(), 2);
            let outer = &items[0];
            assert_eq!(outer.blocks.len(), 2);
            assert!(matches!(outer.blocks[0], Block::Paragraph(_)));
            assert!(matches!(outer.blocks[1], Block::List { .. }));
            let sibling = &items[1];
            assert_eq!(sibling.blocks.len(), 1);
        } else {
            panic!("expected Block::List, got {list:?}");
        }
    }

    #[test]
    fn superscript_parsed() {
        let src = "a ^sup^ b";
        let blocks = parse(src);
        assert_eq!(blocks.len(), 1);
        if let Block::Paragraph(inlines) = &blocks[0] {
            assert!(inlines.iter().any(|i| matches!(i, Inline::Superscript(_))));
        } else {
            panic!("expected Block::Paragraph");
        }
    }

    #[test]
    fn subscript_parsed() {
        let src = "a ~sub~ b";
        let blocks = parse(src);
        assert_eq!(blocks.len(), 1);
        if let Block::Paragraph(inlines) = &blocks[0] {
            assert!(inlines.iter().any(|i| matches!(i, Inline::Subscript(_))));
        } else {
            panic!("expected Block::Paragraph");
        }
    }

    #[test]
    fn display_math_then_text() {
        let src = "before\n\n$$\nE = mc^2\n$$\n\nafter\n";
        let blocks = parse(src);
        assert_eq!(blocks.len(), 3);
        assert!(matches!(blocks[0], Block::Paragraph(_)));
        assert!(matches!(blocks[1], Block::DisplayMath(_)));
        assert!(matches!(blocks[2], Block::Paragraph(_)));
        if let Block::Paragraph(inlines) = &blocks[2] {
            let text = plain_text(inlines);
            assert_eq!(text, "after");
        }
    }
}
