use repose_core::{prelude::*, PaddingValues};
use repose_ui::*;

fn latex_cmd_to_unicode(cmd: &str) -> Option<&'static str> {
    match cmd {
        // Greek lowercase
        "alpha" => Some("\u{03B1}"),
        "beta" => Some("\u{03B2}"),
        "gamma" => Some("\u{03B3}"),
        "delta" => Some("\u{03B4}"),
        "epsilon" => Some("\u{03B5}"),
        "varepsilon" => Some("\u{03F5}"),
        "zeta" => Some("\u{03B6}"),
        "eta" => Some("\u{03B7}"),
        "theta" => Some("\u{03B8}"),
        "vartheta" => Some("\u{03D1}"),
        "iota" => Some("\u{03B9}"),
        "kappa" => Some("\u{03BA}"),
        "lambda" => Some("\u{03BB}"),
        "mu" => Some("\u{03BC}"),
        "nu" => Some("\u{03BD}"),
        "xi" => Some("\u{03BE}"),
        "omicron" => Some("o"),
        "pi" => Some("\u{03C0}"),
        "varpi" => Some("\u{03D6}"),
        "rho" => Some("\u{03C1}"),
        "varrho" => Some("\u{03F1}"),
        "sigma" => Some("\u{03C3}"),
        "varsigma" => Some("\u{03C2}"),
        "tau" => Some("\u{03C4}"),
        "upsilon" => Some("\u{03C5}"),
        "phi" => Some("\u{03C6}"),
        "varphi" => Some("\u{03D5}"),
        "chi" => Some("\u{03C7}"),
        "psi" => Some("\u{03C8}"),
        "omega" => Some("\u{03C9}"),
        // Greek uppercase
        "Alpha" => Some("A"),
        "Beta" => Some("B"),
        "Gamma" => Some("\u{0393}"),
        "Delta" => Some("\u{0394}"),
        "Epsilon" => Some("E"),
        "Zeta" => Some("Z"),
        "Eta" => Some("H"),
        "Theta" => Some("\u{0398}"),
        "Iota" => Some("I"),
        "Kappa" => Some("K"),
        "Lambda" => Some("\u{039B}"),
        "Mu" => Some("M"),
        "Nu" => Some("N"),
        "Xi" => Some("\u{039E}"),
        "Omicron" => Some("O"),
        "Pi" => Some("\u{03A0}"),
        "Rho" => Some("P"),
        "Sigma" => Some("\u{03A3}"),
        "Tau" => Some("T"),
        "Upsilon" => Some("\u{03A5}"),
        "Phi" => Some("\u{03A6}"),
        "Chi" => Some("X"),
        "Psi" => Some("\u{03A8}"),
        "Omega" => Some("\u{03A9}"),
        // Operators
        "sum" => Some("\u{2211}"),
        "prod" => Some("\u{220F}"),
        "coprod" => Some("\u{2210}"),
        "int" => Some("\u{222B}"),
        "iint" => Some("\u{222C}"),
        "iiint" => Some("\u{222D}"),
        "oint" => Some("\u{222E}"),
        "bigcup" => Some("\u{22C3}"),
        "bigcap" => Some("\u{22C2}"),
        "bigvee" => Some("\u{22C1}"),
        "bigwedge" => Some("\u{22C0}"),
        "bigoplus" => Some("\u{2A01}"),
        "bigotimes" => Some("\u{2A02}"),
        "biguplus" => Some("\u{2A04}"),
        "bigsqcup" => Some("\u{2A06}"),
        "nabla" => Some("\u{2207}"),
        "partial" => Some("\u{2202}"),
        "infty" => Some("\u{221E}"),
        "emptyset" => Some("\u{2205}"),
        "varnothing" => Some("\u{2205}"),
        "hbar" => Some("\u{210F}"),
        "ell" => Some("\u{2113}"),
        "imath" => Some("\u{0131}"),
        "jmath" => Some("\u{0237}"),
        "Re" => Some("\u{211C}"),
        "Im" => Some("\u{2111}"),
        "aleph" => Some("\u{2135}"),
        "wp" => Some("\u{2118}"),
        "forall" => Some("\u{2200}"),
        "exists" => Some("\u{2203}"),
        "nexists" => Some("\u{2204}"),
        "neg" => Some("\u{00AC}"),
        "lnot" => Some("\u{00AC}"),
        "top" => Some("\u{22A4}"),
        "bot" => Some("\u{22A5}"),
        "perp" => Some("\u{22A5}"),
        "angle" => Some("\u{2220}"),
        "measuredangle" => Some("\u{2221}"),
        "triangle" => Some("\u{25B3}"),
        "Box" => Some("\u{25A1}"),
        "Diamond" => Some("\u{25C7}"),
        // Relations
        "le" => Some("\u{2264}"),
        "leq" => Some("\u{2264}"),
        "ge" => Some("\u{2265}"),
        "geq" => Some("\u{2265}"),
        "neq" => Some("\u{2260}"),
        "ne" => Some("\u{2260}"),
        "approx" => Some("\u{2248}"),
        "simeq" => Some("\u{2243}"),
        "sim" => Some("\u{223C}"),
        "cong" => Some("\u{2245}"),
        "equiv" => Some("\u{2261}"),
        "propto" => Some("\u{221D}"),
        "parallel" => Some("\u{2225}"),
        "mid" => Some("\u{2223}"),
        "doteq" => Some("\u{2250}"),
        "models" => Some("\u{22A7}"),
        "subset" => Some("\u{2282}"),
        "supset" => Some("\u{2283}"),
        "subseteq" => Some("\u{2286}"),
        "supseteq" => Some("\u{2287}"),
        "sqsubset" => Some("\u{228F}"),
        "sqsupset" => Some("\u{2290}"),
        "sqsubseteq" => Some("\u{2291}"),
        "sqsupseteq" => Some("\u{2292}"),
        "in" => Some("\u{2208}"),
        "ni" => Some("\u{220B}"),
        "notin" => Some("\u{2209}"),
        "owns" => Some("\u{220B}"),
        "prec" => Some("\u{227A}"),
        "succ" => Some("\u{227B}"),
        "preceq" => Some("\u{2AAF}"),
        "succeq" => Some("\u{2AB0}"),
        "ll" => Some("\u{226A}"),
        "gg" => Some("\u{226B}"),
        "asymp" => Some("\u{224D}"),
        "smile" => Some("\u{2323}"),
        "frown" => Some("\u{2322}"),
        // Binary operators
        "pm" => Some("\u{00B1}"),
        "mp" => Some("\u{2213}"),
        "times" => Some("\u{00D7}"),
        "div" => Some("\u{00F7}"),
        "ast" => Some("*"),
        "star" => Some("\u{22C6}"),
        "circ" => Some("\u{2218}"),
        "bullet" => Some("\u{2219}"),
        "cdot" => Some("\u{22C5}"),
        "cdotp" => Some("\u{22C5}"),
        "centerdot" => Some("\u{22C5}"),
        "wedge" => Some("\u{2227}"),
        "vee" => Some("\u{2228}"),
        "cap" => Some("\u{2229}"),
        "cup" => Some("\u{222A}"),
        "sqcap" => Some("\u{2293}"),
        "sqcup" => Some("\u{2294}"),
        "uplus" => Some("\u{228E}"),
        "oplus" => Some("\u{2295}"),
        "ominus" => Some("\u{2296}"),
        "otimes" => Some("\u{2297}"),
        "oslash" => Some("\u{2298}"),
        "odot" => Some("\u{2299}"),
        "dagger" => Some("\u{2020}"),
        "ddagger" => Some("\u{2021}"),
        "amalg" => Some("\u{2A3F}"),
        "wr" => Some("\u{2240}"),
        "setminus" => Some("\u{2216}"),
        "triangleleft" => Some("\u{25C1}"),
        "triangleright" => Some("\u{25B7}"),
        // Arrows
        "to" => Some("\u{2192}"),
        "rightarrow" => Some("\u{2192}"),
        "Rightarrow" => Some("\u{21D2}"),
        "leftarrow" => Some("\u{2190}"),
        "Leftarrow" => Some("\u{21D0}"),
        "leftrightarrow" => Some("\u{2194}"),
        "Leftrightarrow" => Some("\u{21D4}"),
        "mapsto" => Some("\u{21A6}"),
        "longmapsto" => Some("\u{27FC}"),
        "Longmapsto" => Some("\u{27FC}"),
        "gets" => Some("\u{2190}"),
        "hookrightarrow" => Some("\u{21AA}"),
        "hookleftarrow" => Some("\u{21A9}"),
        "rightharpoonup" => Some("\u{21C0}"),
        "rightharpoondown" => Some("\u{21C1}"),
        "leftharpoonup" => Some("\u{21BC}"),
        "leftharpoondown" => Some("\u{21BD}"),
        "rightleftharpoons" => Some("\u{21CC}"),
        "uparrow" => Some("\u{2191}"),
        "Uparrow" => Some("\u{21D1}"),
        "downarrow" => Some("\u{2193}"),
        "Downarrow" => Some("\u{21D3}"),
        "updownarrow" => Some("\u{2195}"),
        "Updownarrow" => Some("\u{21D5}"),
        "nearrow" => Some("\u{2197}"),
        "searrow" => Some("\u{2198}"),
        "nwarrow" => Some("\u{2196}"),
        "swarrow" => Some("\u{2199}"),
        "Longrightarrow" => Some("\u{27F9}"),
        "longrightarrow" => Some("\u{27F6}"),
        "longleftarrow" => Some("\u{27F5}"),
        "Longleftarrow" => Some("\u{27F8}"),
        "longleftrightarrow" => Some("\u{27F7}"),
        "Longleftrightarrow" => Some("\u{27FA}"),
        "iff" => Some("\u{27FA}"),
        "implies" => Some("\u{27F9}"),
        "leadsto" => Some("\u{21DD}"),
        // Miscellaneous symbols
        "dots" => Some("\u{2026}"),
        "ldots" => Some("\u{2026}"),
        "cdots" => Some("\u{22EF}"),
        "vdots" => Some("\u{22EE}"),
        "ddots" => Some("\u{22F1}"),
        "prime" => Some("\u{2032}"),
        "dprime" => Some("\u{2033}"),
        "tripleprime" => Some("\u{2034}"),
        "backprime" => Some("\u{2035}"),
        "colon" => Some(":"),
        "ellipsis" => Some("\u{2026}"),
        "surd" => Some("\u{221A}"),
        "hslash" => Some("\u{210F}"),
        "degree" => Some("\u{00B0}"),
        "P" => Some("\u{00B6}"),
        "S" => Some("\u{00A7}"),
        "copyright" => Some("\u{00A9}"),
        "pounds" => Some("\u{00A3}"),
        "clubsuit" => Some("\u{2663}"),
        "diamondsuit" => Some("\u{2662}"),
        "heartsuit" => Some("\u{2661}"),
        "spadesuit" => Some("\u{2660}"),
        "checkmark" => Some("\u{2713}"),
        "triangledown" => Some("\u{25BD}"),
        "square" => Some("\u{25A1}"),
        "blacksquare" => Some("\u{25A0}"),
        "lozenge" => Some("\u{25CA}"),
        "flat" => Some("\u{266D}"),
        "natural" => Some("\u{266E}"),
        "sharp" => Some("\u{266F}"),

        _ => None,
    }
}

fn latex_is_function_name(cmd: &str) -> bool {
    matches!(
        cmd,
        "sin"
            | "cos"
            | "tan"
            | "cot"
            | "sec"
            | "csc"
            | "sinh"
            | "cosh"
            | "tanh"
            | "coth"
            | "arcsin"
            | "arccos"
            | "arctan"
            | "arccot"
            | "arcsec"
            | "arccsc"
            | "log"
            | "ln"
            | "lg"
            | "exp"
            | "det"
            | "dim"
            | "hom"
            | "ker"
            | "deg"
            | "Pr"
            | "gcd"
            | "arg"
            | "min"
            | "max"
            | "sup"
            | "inf"
            | "lim"
            | "limsup"
            | "liminf"
            | "injlim"
            | "projlim"
            | "mod"
            | "bmod"
            | "pmod"
    )
}

fn latex_is_ignorable(cmd: &str) -> bool {
    matches!(
        cmd,
        "left"
            | "right"
            | "bigl"
            | "bigr"
            | "Bigl"
            | "Bigr"
            | "biggl"
            | "biggr"
            | "Biggl"
            | "Biggr"
            | "displaystyle"
            | "textstyle"
            | "scriptstyle"
            | "scriptscriptstyle"
            | "limits"
            | "nolimits"
            | "quad"
            | "qquad"
            | "negthinspace"
            | "negmedspace"
            | "negthickspace"
            | "enspace"
            | "enskip"
            | "relax"
            | "strut"
            | "mathstrut"
    )
}

fn read_alpha_name(chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>) -> String {
    let mut name = String::new();
    while let Some(&(_, c)) = chars.peek() {
        if c.is_ascii_alphabetic() {
            chars.next();
            name.push(c);
        } else {
            break;
        }
    }
    name
}

fn skip_ws(chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>) {
    while let Some(&(_, c)) = chars.peek() {
        if c.is_ascii_whitespace() {
            chars.next();
        } else {
            break;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MatrixEnv {
    Pmatrix,
    Bmatrix,
    Vmatrix,
    Matrix,
    Cases,
    Aligned,
}

#[derive(Debug, Clone)]
enum Seg {
    Text(String),
    Sup(Vec<Seg>),
    Sub(Vec<Seg>),
    Frac(Vec<Seg>, Vec<Seg>),
    Sqrt(Vec<Seg>, Option<Vec<Seg>>),
    Binom(Vec<Seg>, Vec<Seg>),
    Font(Vec<Seg>),
    Accent(&'static str, Vec<Seg>),
    Matrix { env: MatrixEnv, rows: Vec<Vec<Vec<Seg>>> },
}

/// Dispatch a known command (already read) and produce its segment(s).
fn parse_cmd_segments(
    cmd: &str,
    chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
) -> Vec<Seg> {
    if cmd.is_empty() {
        return vec![];
    }
    if latex_is_ignorable(&cmd) {
        return vec![];
    }
    if latex_is_function_name(&cmd) {
        return vec![Seg::Text(cmd.to_string())];
    }
    if let Some(unicode) = latex_cmd_to_unicode(cmd) {
        return vec![Seg::Text(unicode.to_string())];
    }
    if cmd == "over" {
        // \over is special — it operates on the preceding segs.
        // Return a sentinel — caller must handle it.
        return vec![];
    }
    let result: Vec<Seg> = match cmd {
        "frac" => {
            vec![Seg::Frac(parse_math_group(chars), parse_math_group(chars))]
        }
        "sqrt" => {
            skip_ws(chars);
            let degree = if chars.peek().is_some_and(|(_, p)| *p == '[') {
                chars.next();
                let inner = parse_math_until(chars, false, true);
                let _close = chars.next();
                Some(inner)
            } else {
                None
            };
            vec![Seg::Sqrt(parse_math_group(chars), degree)]
        }
        "binom" => {
            vec![Seg::Binom(parse_math_group(chars), parse_math_group(chars))]
        }
        "mathbb" | "mathcal" | "mathrm" | "mathbf" | "mathit" | "mathsf" | "mathtt" | "mathscr" | "mathfrak" => {
            vec![Seg::Font(parse_math_group(chars))]
        }
        "hat" | "bar" | "dot" | "vec" | "tilde" => {
            let combining = match cmd {
                "hat" => "\u{0302}",
                "bar" => "\u{0304}",
                "dot" => "\u{0307}",
                "vec" => "\u{20D7}",
                "tilde" => "\u{0303}",
                _ => unreachable!(),
            };
            vec![Seg::Accent(combining, parse_math_group(chars))]
        }
        "begin" => {
            let env_name = parse_braced_text(chars);
            if let Some(env) = match env_name.as_str() {
                "pmatrix" => Some(MatrixEnv::Pmatrix),
                "bmatrix" => Some(MatrixEnv::Bmatrix),
                "vmatrix" => Some(MatrixEnv::Vmatrix),
                "matrix" => Some(MatrixEnv::Matrix),
                "cases" => Some(MatrixEnv::Cases),
                "aligned" => Some(MatrixEnv::Aligned),
                _ => None,
            } {
                vec![Seg::Matrix { env, rows: parse_matrix_until(chars, &env_name) }]
            } else {
                vec![]
            }
        }
        "text" => {
            vec![Seg::Text(parse_braced_text(chars))]
        }
        _ => {
            vec![Seg::Text(format!("\\{}", cmd))]
        }
    };
    result
}

fn parse_math_until(
    chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
    stop_at_brace: bool,
    stop_at_bracket: bool,
) -> Vec<Seg> {
    let mut segs: Vec<Seg> = Vec::new();
    let mut buf = String::new();

    let flush = |buf: &mut String, segs: &mut Vec<Seg>| {
        if !buf.is_empty() {
            segs.push(Seg::Text(std::mem::take(buf)));
        }
    };

    while let Some(&(_, c)) = chars.peek() {
        match c {
            '\\' => {
                chars.next();
                let cmd = read_alpha_name(chars);
                if cmd.is_empty() {
                    continue;
                }
                if cmd == "over" {
                    flush(&mut buf, &mut segs);
                    let num = std::mem::take(&mut segs);
                    let den = parse_math_until(chars, stop_at_brace, false);
                    segs = vec![Seg::Frac(num, den)];
                    break;
                }
                let cmd_segs = parse_cmd_segments(&cmd, chars);
                for s in cmd_segs {
                    match s {
                        Seg::Text(t) => buf.push_str(&t),
                        other => {
                            flush(&mut buf, &mut segs);
                            segs.push(other);
                        }
                    }
                }
            }
            '{' => {
                chars.next();
                let inner = parse_math_until(chars, true, false);
                chars.next();
                for s in inner {
                    match s {
                        Seg::Text(t) => buf.push_str(&t),
                        other => {
                            flush(&mut buf, &mut segs);
                            segs.push(other);
                        }
                    }
                }
            }
            '}' if stop_at_brace => break,
            ']' if stop_at_bracket => break,
            '^' => {
                chars.next();
                flush(&mut buf, &mut segs);
                let content = parse_super_sub_group(chars);
                segs.push(Seg::Sup(content));
            }
            '_' => {
                chars.next();
                flush(&mut buf, &mut segs);
                let content = parse_super_sub_group(chars);
                segs.push(Seg::Sub(content));
            }
            _ => {
                buf.push(c);
                chars.next();
            }
        }
    }
    flush(&mut buf, &mut segs);
    segs
}

fn parse_math_group(chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>) -> Vec<Seg> {
    skip_ws(chars);
    if chars.peek().is_some_and(|(_, p)| *p == '{') {
        chars.next();
        let inner = parse_math_until(chars, true, false);
        chars.next();
        inner
    } else {
        let mut segs = Vec::new();
        if let Some((_, c)) = chars.next() {
            segs.push(Seg::Text(c.to_string()));
        }
        segs
    }
}

fn parse_super_sub_group(chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>) -> Vec<Seg> {
    skip_ws(chars);
    if chars.peek().is_some_and(|(_, p)| *p == '{') {
        chars.next();
        let inner = parse_math_until(chars, true, false);
        chars.next();
        inner
    } else if chars.peek().is_some_and(|(_, p)| *p == '\\') {
        chars.next();
        let cmd = read_alpha_name(chars);
        if cmd.is_empty() {
            vec![Seg::Text("\\".to_string())]
        } else {
            parse_cmd_segments(&cmd, chars)
        }
    } else if let Some((_, c)) = chars.next() {
        vec![Seg::Text(c.to_string())]
    } else {
        vec![]
    }
}

fn parse_braced_text(chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>) -> String {
    skip_ws(chars);
    if chars.peek().is_some_and(|(_, p)| *p == '{') {
        chars.next();
        let mut inner = String::new();
        let mut depth = 1u32;
        while let Some((_, c)) = chars.next() {
            match c {
                '{' => {
                    depth += 1;
                    inner.push(c);
                }
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    inner.push(c);
                }
                _ => inner.push(c),
            }
        }
        inner
    } else {
        String::new()
    }
}

fn parse_matrix_until(
    chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
    env_name: &str,
) -> Vec<Vec<Vec<Seg>>> {
    let mut rows: Vec<Vec<String>> = vec![vec![String::new()]];
    let mut depth = 0u32;

    while let Some(&(_, c)) = chars.peek() {
        match c {
            '{' => {
                depth += 1;
                let cur = rows.last_mut().unwrap().last_mut().unwrap();
                cur.push('{');
                chars.next();
            }
            '}' => {
                if depth == 0 {
                    break;
                }
                depth -= 1;
                let cur = rows.last_mut().unwrap().last_mut().unwrap();
                cur.push('}');
                chars.next();
            }
            '&' if depth == 0 => {
                chars.next();
                rows.last_mut().unwrap().push(String::new());
            }
            '\\' if depth == 0 => {
                chars.next();
                if chars.peek().is_some_and(|(_, p)| *p == '\\') {
                    chars.next();
                    if chars.peek().is_some_and(|(_, p)| *p == '[') {
                        while let Some(&(_, p)) = chars.peek() {
                            if p == ']' { chars.next(); break; }
                            chars.next();
                        }
                    }
                    rows.push(vec![String::new()]);
                } else {
                    let cmd = read_alpha_name(chars);
                    if cmd == "end" {
                        let end_name = parse_braced_text(chars);
                        if end_name == env_name {
                            break;
                        } else if !end_name.is_empty() {
                            let cur = rows.last_mut().unwrap().last_mut().unwrap();
                            cur.push_str("\\end{");
                            cur.push_str(&end_name);
                            cur.push('}');
                        }
                    } else if !cmd.is_empty() {
                        let cur = rows.last_mut().unwrap().last_mut().unwrap();
                        cur.push('\\');
                        cur.push_str(&cmd);
                    }
                }
            }
            ' ' | '\t' | '\n' | '\r' if depth == 0 => {
                chars.next();
            }
            _ => {
                let cur = rows.last_mut().unwrap().last_mut().unwrap();
                cur.push(c);
                chars.next();
            }
        }
    }

    if rows.last().map_or(false, |r| r.len() == 1 && r[0].is_empty()) {
        rows.pop();
    }

    rows.into_iter().map(|row| {
        row.into_iter().map(|cell| {
            let mut cell_chars = cell.char_indices().peekable();
            parse_math_until(&mut cell_chars, false, false)
        }).collect()
    }).collect()
}

fn build_math_view(segs: Vec<Seg>, font_size: f32) -> Vec<View> {
    let sup_size = (font_size * 0.65).max(9.0);
    let sub_size = (font_size * 0.65).max(9.0);
    let sup_offset = -font_size * 0.15;
    let sub_offset = font_size * 0.55;
    let color = theme().on_surface;

    segs.into_iter()
        .map(|seg| match seg {
            Seg::Text(t) => Text(t)
                .font_family("monospace")
                .size(font_size)
                .color(color)
                .into(),
            Seg::Sup(children) => {
                let kids = build_math_view(children, sup_size);
                Column(Modifier::new()).child((
                    FlowRow(Modifier::new()).child(kids),
                    Box(Modifier::new().height(-sup_offset).width(1.0)),
                ))
            }
            Seg::Sub(children) => {
                let kids = build_math_view(children, sub_size);
                Column(Modifier::new()).child((
                    Box(Modifier::new().height(sub_offset).width(1.0)),
                    FlowRow(Modifier::new()).child(kids),
                ))
            }
            Seg::Frac(num, den) => {
                let num_kids = build_math_view(num, font_size);
                let den_kids = build_math_view(den, font_size);
                let line_color = color;
                Column(Modifier::new().align_items(AlignItems::CENTER)).child((
                    FlowRow(Modifier::new()).child(num_kids),
                    Box(Modifier::new()
                        .fill_max_width()
                        .height(1.5)
                        .background(line_color)),
                    FlowRow(Modifier::new()).child(den_kids),
                ))
            }
            Seg::Binom(top, bot) => {
                let top_kids = build_math_view(top, font_size);
                let bot_kids = build_math_view(bot, font_size);
                let paren_color = color;
                Row(Modifier::new().align_items(AlignItems::CENTER)).child((
                    Text("(").font_family("monospace").size(font_size).color(paren_color),
                    Column(Modifier::new().align_items(AlignItems::CENTER)).child((
                        FlowRow(Modifier::new()).child(top_kids),
                        FlowRow(Modifier::new()).child(bot_kids),
                    )),
                    Text(")").font_family("monospace").size(font_size).color(paren_color),
                ))
            }
            Seg::Font(children) => {
                FlowRow(Modifier::new()).child(build_math_view(children, font_size))
            }
            Seg::Accent(combining, children) => {
                let mut kids = build_math_view(children, font_size);
                kids.push(
                    Text(combining)
                        .font_family("monospace")
                        .size(font_size)
                        .color(color)
                        .into(),
                );
                FlowRow(Modifier::new()).child(kids)
            }
            Seg::Matrix { ref env, ref rows } => {
                let (left_delim, right_delim) = match env {
                    MatrixEnv::Pmatrix => ("(", ")"),
                    MatrixEnv::Bmatrix => ("[", "]"),
                    MatrixEnv::Vmatrix => ("|", "|"),
                    MatrixEnv::Matrix => ("", ""),
                    MatrixEnv::Cases => ("", ""),
                    MatrixEnv::Aligned => ("", ""),
                };
                let row_views: Vec<View> = rows.iter().map(|row| {
                    let cell_views: Vec<View> = row.iter().map(|cell| {
                        let kids = build_math_view(cell.clone(), font_size);
                        Box(Modifier::new().padding_values(PaddingValues {
                            left: 3.0, right: 3.0, top: 1.0, bottom: 1.0,
                        })).child(Row(Modifier::new().align_items(AlignItems::FLEX_START)).child(kids))
                    }).collect();
                    Row(Modifier::new().align_items(AlignItems::FLEX_START)).child(cell_views)
                }).collect();

                let grid = Column(Modifier::new().align_items(AlignItems::CENTER)).child(
                    intersperse_vertical(row_views, 2.0)
                );

                if *env == MatrixEnv::Cases {
                    Row(Modifier::new().align_items(AlignItems::CENTER)).child((
                        Text("\u{007B}").font_family("monospace").size(font_size * 1.5).color(color),
                        grid,
                    ))
                } else if !left_delim.is_empty() {
                    Row(Modifier::new().align_items(AlignItems::CENTER)).child((
                        Text(left_delim).font_family("monospace").size(font_size * 1.2).color(color),
                        grid,
                        Text(right_delim).font_family("monospace").size(font_size * 1.2).color(color),
                    ))
                } else {
                    grid
                }
            }
            Seg::Sqrt(radicand, degree) => {
                let rad_kids = build_math_view(radicand, font_size);
                let overline_color = color;
                if let Some(deg) = degree {
                    let deg_kids = build_math_view(deg, sup_size);
                    Row(Modifier::new().align_items(AlignItems::CENTER)).child((
                        Box(Modifier::new().translate(0.0, sup_offset))
                            .child(FlowRow(Modifier::new()).child(deg_kids)),
                        Text("\u{221A}")
                            .font_family("monospace")
                            .size(font_size)
                            .color(color),
                        Column(Modifier::new().align_items(AlignItems::CENTER)).child((
                            Box(Modifier::new()
                                .fill_max_width()
                                .height(1.5)
                                .background(overline_color)),
                            FlowRow(Modifier::new()).child(rad_kids),
                        )),
                    ))
                } else {
                    Row(Modifier::new().align_items(AlignItems::CENTER)).child((
                        Text("\u{221A}")
                            .font_family("monospace")
                            .size(font_size)
                            .color(color),
                        Column(Modifier::new().align_items(AlignItems::CENTER)).child((
                            Box(Modifier::new()
                                .fill_max_width()
                                .height(1.5)
                                .background(overline_color)),
                            FlowRow(Modifier::new()).child(rad_kids),
                        )),
                    ))
                }
            }
        })
        .collect()
}

fn intersperse_vertical(children: Vec<View>, gap: f32) -> Vec<View> {
    let mut result = Vec::with_capacity(children.len().saturating_mul(2).saturating_sub(1));
    for (i, child) in children.into_iter().enumerate() {
        if i > 0 {
            result.push(Box(Modifier::new().height(gap).width(1.0)));
        }
        result.push(child);
    }
    result
}

pub(crate) fn render_math_string(text: &str, font_size: f32) -> View {
    let mut chars = text.char_indices().peekable();
    let segs = parse_math_until(&mut chars, false, false);
    let children = build_math_view(segs, font_size);
    FlowRow(Modifier::new().align_items(AlignItems::FLEX_START)).child(children)
}

pub(crate) fn render_display_math(text: &str, font_size: f32) -> View {
    let mut chars = text.char_indices().peekable();
    let segs = parse_math_until(&mut chars, false, false);
    let children = build_math_view(segs, font_size);
    Row(Modifier::new().align_items(AlignItems::CENTER)).child(children)
}
