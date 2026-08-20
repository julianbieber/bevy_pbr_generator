use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParamKind {
    F32,
    I32,
    U32,
    Vec2,
    Vec3,
    Vec4,
}

impl ParamKind {
    fn from_wgsl(ty: &str) -> Option<Self> {
        match ty.replace(char::is_whitespace, "").as_str() {
            "f32" => Some(Self::F32),
            "i32" => Some(Self::I32),
            "u32" => Some(Self::U32),
            "vec2<f32>" | "vec2f" => Some(Self::Vec2),
            "vec3<f32>" | "vec3f" => Some(Self::Vec3),
            "vec4<f32>" | "vec4f" => Some(Self::Vec4),
            _ => None,
        }
    }

    pub fn components(self) -> usize {
        match self {
            Self::F32 | Self::I32 | Self::U32 => 1,
            Self::Vec2 => 2,
            Self::Vec3 => 3,
            Self::Vec4 => 4,
        }
    }

    pub fn is_integer(self) -> bool {
        matches!(self, Self::I32 | Self::U32)
    }

    fn align(self) -> usize {
        match self {
            Self::F32 | Self::I32 | Self::U32 => 4,
            Self::Vec2 => 8,
            Self::Vec3 | Self::Vec4 => 16,
        }
    }

    fn size(self) -> usize {
        match self {
            Self::F32 | Self::I32 | Self::U32 => 4,
            Self::Vec2 => 8,
            Self::Vec3 => 12,
            Self::Vec4 => 16,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Widget {
    Slider {
        min: f32,
        max: f32,
        step: Option<f32>,
    },
    Color,
    Toggle,
    Hidden,
}

#[derive(Clone, Debug)]
pub struct ParamSpec {
    pub name: String,
    pub label: String,
    pub group: String,
    pub kind: ParamKind,
    pub widget: Widget,
    pub default: [f32; 4],
    pub offset: usize,
}

#[derive(Clone, Debug, Default)]
pub struct ParamLayout {
    pub params: Vec<ParamSpec>,
    pub size: usize,
}

impl ParamLayout {
    pub fn defaults(&self) -> Vec<[f32; 4]> {
        self.params.iter().map(|p| p.default).collect()
    }

    pub fn pack(&self, values: &[[f32; 4]]) -> Vec<u8> {
        let mut bytes = vec![0u8; self.size];
        for (spec, value) in self.params.iter().zip(values) {
            for (component, scalar) in value.iter().take(spec.kind.components()).enumerate() {
                let at = spec.offset + component * 4;
                let raw = match spec.kind {
                    ParamKind::I32 => (*scalar as i32).to_ne_bytes(),
                    ParamKind::U32 => (scalar.max(0.0) as u32).to_ne_bytes(),
                    _ => scalar.to_ne_bytes(),
                };
                bytes[at..at + 4].copy_from_slice(&raw);
            }
        }
        bytes
    }

    pub fn groups(&self) -> Vec<(String, Vec<usize>)> {
        let mut groups: Vec<(String, Vec<usize>)> = Vec::new();
        for (index, spec) in self.params.iter().enumerate() {
            if spec.widget == Widget::Hidden {
                continue;
            }
            match groups.last_mut() {
                Some((name, members)) if *name == spec.group => members.push(index),
                _ => groups.push((spec.group.clone(), vec![index])),
            }
        }
        groups
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParamParseError {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for ParamParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

pub fn parse_params(source: &str) -> Result<ParamLayout, ParamParseError> {
    let Some(body) = struct_body(source, "Params") else {
        return Ok(ParamLayout::default());
    };

    let mut params: Vec<ParamSpec> = Vec::new();
    let mut group = String::new();
    let mut cursor = 0usize;

    for (offset, raw_line) in body.lines.iter().enumerate() {
        let line_number = body.first_line + offset;
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(rest) = line.strip_prefix("//") {
            if let Some(name) = rest.trim().strip_prefix("@group") {
                group = name.trim().to_string();
            }
            continue;
        }

        let (declaration, annotation) = match line.find("//") {
            Some(at) => (&line[..at], Some(line[at + 2..].trim())),
            None => (line, None),
        };

        let declaration = declaration.trim().trim_end_matches(',').trim();
        if declaration.is_empty() {
            continue;
        }

        let Some((name, ty)) = declaration.split_once(':') else {
            return Err(ParamParseError {
                line: line_number,
                message: format!("expected `name: type`, found `{declaration}`"),
            });
        };

        let name = name.trim().to_string();
        let Some(kind) = ParamKind::from_wgsl(ty) else {
            return Err(ParamParseError {
                line: line_number,
                message: format!("unsupported parameter type `{}`", ty.trim()),
            });
        };

        let annotation = annotation
            .and_then(|a| a.strip_prefix("@ui"))
            .map(str::trim);
        let (label, widget, default) = match annotation {
            Some(text) => parse_annotation(text, kind).map_err(|message| ParamParseError {
                line: line_number,
                message,
            })?,
            None => (None, default_widget(kind), [0.0; 4]),
        };

        let offset = round_up(cursor, kind.align());
        cursor = offset + kind.size();

        params.push(ParamSpec {
            label: label.unwrap_or_else(|| humanise(&name)),
            name,
            group: group.clone(),
            kind,
            widget,
            default,
            offset,
        });
    }

    Ok(ParamLayout {
        params,
        size: round_up(cursor, 16).max(16),
    })
}

fn default_widget(kind: ParamKind) -> Widget {
    let _ = kind;
    Widget::Slider {
        min: 0.0,
        max: 1.0,
        step: None,
    }
}

struct StructBody<'a> {
    lines: Vec<&'a str>,
    first_line: usize,
}

fn struct_body<'a>(source: &'a str, name: &str) -> Option<StructBody<'a>> {
    let lines: Vec<&str> = source.lines().collect();
    let start = lines.iter().position(|line| {
        let trimmed = line.trim_start();
        trimmed.starts_with("struct")
            && trimmed[6..].trim_start().starts_with(name)
            && trimmed[6..].trim_start()[name.len()..]
                .trim_start()
                .starts_with('{')
    })?;

    let mut depth = 0i32;
    let mut body = Vec::new();
    for (index, line) in lines.iter().enumerate().skip(start) {
        let opens = line.matches('{').count() as i32;
        let closes = line.matches('}').count() as i32;

        if index > start {
            if depth + opens - closes <= 0 {
                let end = line.rfind('}').unwrap_or(0);
                body.push(&line[..end]);
                break;
            }
            body.push(line);
        }

        depth += opens - closes;
        if index == start && depth <= 0 {
            break;
        }
    }

    Some(StructBody {
        lines: body,
        first_line: start + 2,
    })
}

fn round_up(value: usize, alignment: usize) -> usize {
    value.div_ceil(alignment) * alignment
}

fn humanise(name: &str) -> String {
    let mut out = String::new();
    for (index, word) in name.split('_').enumerate() {
        if word.is_empty() {
            continue;
        }
        if index > 0 {
            out.push(' ');
        }
        let mut chars = word.chars();
        match chars.next() {
            Some(first) if index == 0 => {
                out.extend(first.to_uppercase());
                out.push_str(chars.as_str());
            }
            Some(first) => {
                out.push(first);
                out.push_str(chars.as_str());
            }
            None => {}
        }
    }
    out
}

#[derive(Debug, PartialEq)]
enum Token {
    Word(String),
    Number(f32),
    Tuple(Vec<f32>),
    Range(f32, f32),
}

fn tokenize(text: &str) -> Result<Vec<Token>, String> {
    let bytes: Vec<char> = text.chars().collect();
    let mut tokens = Vec::new();
    let mut at = 0usize;

    while at < bytes.len() {
        let c = bytes[at];
        if c.is_whitespace() || c == ',' {
            at += 1;
            continue;
        }

        if c == '(' || c == '[' {
            let close = if c == '(' { ')' } else { ']' };
            let Some(end) = bytes[at..].iter().position(|&x| x == close) else {
                return Err(format!("unclosed `{c}`"));
            };
            let inner: String = bytes[at + 1..at + end].iter().collect();
            let numbers = parse_numbers(&inner)?;
            if c == '[' {
                if numbers.len() != 2 {
                    return Err("a range needs exactly two values".to_string());
                }
                tokens.push(Token::Range(numbers[0], numbers[1]));
            } else {
                tokens.push(Token::Tuple(numbers));
            }
            at += end + 1;
            continue;
        }

        let start = at;
        while at < bytes.len()
            && !bytes[at].is_whitespace()
            && bytes[at] != ','
            && bytes[at] != '('
            && bytes[at] != '['
        {
            at += 1;
        }
        let word: String = bytes[start..at].iter().collect();
        match word.parse::<f32>() {
            Ok(number) => tokens.push(Token::Number(number)),
            Err(_) => tokens.push(Token::Word(word)),
        }
    }

    Ok(tokens)
}

fn parse_numbers(text: &str) -> Result<Vec<f32>, String> {
    text.split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| {
            part.parse::<f32>()
                .map_err(|_| format!("`{part}` is not a number"))
        })
        .collect()
}

fn parse_annotation(
    text: &str,
    kind: ParamKind,
) -> Result<(Option<String>, Widget, [f32; 4]), String> {
    let (label, rest) = match text.strip_prefix('"') {
        Some(after) => match after.find('"') {
            Some(end) => (Some(after[..end].to_string()), after[end + 1..].trim()),
            None => return Err("unterminated label".to_string()),
        },
        None => (None, text),
    };

    let tokens = tokenize(rest)?;
    let mut default = [0.0f32; 4];

    let keyword = match tokens.first() {
        Some(Token::Word(word)) => Some(word.as_str()),
        _ => None,
    };

    match keyword {
        Some("hidden") => {
            let values = match tokens.get(1) {
                Some(Token::Tuple(values)) => Some(values.clone()),
                Some(Token::Number(value)) => Some(vec![*value]),
                _ => None,
            };
            if let Some(values) = values {
                write_components(&mut default, &values, kind)?;
            }
            Ok((label, Widget::Hidden, default))
        }
        Some("toggle") => {
            let on = matches!(tokens.get(1), Some(Token::Word(word)) if word == "true");
            default[0] = if on { 1.0 } else { 0.0 };
            Ok((label, Widget::Toggle, default))
        }
        Some("color") => {
            let components = match tokens.get(1) {
                Some(Token::Word(space)) => {
                    let values = match tokens.get(2) {
                        Some(Token::Tuple(values)) => values.clone(),
                        _ => return Err("expected `srgb(...)` or `linear(...)`".to_string()),
                    };
                    match space.as_str() {
                        "srgb" => values.iter().map(|c| srgb_to_linear(*c)).collect(),
                        "linear" => values,
                        other => return Err(format!("unknown colour space `{other}`")),
                    }
                }
                Some(Token::Tuple(values)) => values.clone(),
                _ => return Err("expected a colour value".to_string()),
            };
            default[3] = 1.0;
            write_components(&mut default, &components, kind)?;
            Ok((label, Widget::Color, default))
        }
        _ => {
            let mut index = 0usize;
            if keyword == Some("vec") {
                index = 1;
            }

            let values = match tokens.get(index) {
                Some(Token::Number(value)) => vec![*value],
                Some(Token::Tuple(values)) => values.clone(),
                _ => return Err("expected a default value".to_string()),
            };
            write_components(&mut default, &values, kind)?;
            index += 1;

            let (mut min, mut max) = (0.0f32, 1.0f32);
            if let Some(Token::Range(low, high)) = tokens.get(index) {
                min = *low;
                max = *high;
                index += 1;
            }

            let mut step = None;
            if let Some(Token::Word(word)) = tokens.get(index) {
                if word == "step" {
                    match tokens.get(index + 1) {
                        Some(Token::Number(value)) => step = Some(*value),
                        _ => return Err("`step` needs a value".to_string()),
                    }
                }
            }

            if kind.is_integer() && step.is_none() {
                step = Some(1.0);
            }

            Ok((label, Widget::Slider { min, max, step }, default))
        }
    }
}

fn write_components(target: &mut [f32; 4], values: &[f32], kind: ParamKind) -> Result<(), String> {
    if values.len() == 1 {
        for slot in target.iter_mut().take(kind.components()) {
            *slot = values[0];
        }
        return Ok(());
    }

    if values.len() > kind.components() {
        return Err(format!(
            "{} values given for a {}-component parameter",
            values.len(),
            kind.components()
        ));
    }

    for (slot, value) in target.iter_mut().zip(values) {
        *slot = *value;
    }
    Ok(())
}

pub fn srgb_to_linear(value: f32) -> f32 {
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

pub fn linear_to_srgb(value: f32) -> f32 {
    if value <= 0.0031308 {
        value * 12.92
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout(source: &str) -> ParamLayout {
        parse_params(source).expect("parse should succeed")
    }

    #[test]
    fn no_params_struct_yields_empty_layout() {
        let parsed = layout("fn surface() {}");
        assert!(parsed.params.is_empty());
        assert_eq!(parsed.size, 0);
    }

    #[test]
    fn scalar_slider_defaults_and_offsets() {
        let parsed = layout(
            "struct Params {\n    scale: f32, // @ui 5.0 [0.1, 32.0]\n    warp: f32, // @ui 0.25 [0.0, 1.0]\n}",
        );
        assert_eq!(parsed.params.len(), 2);
        assert_eq!(parsed.params[0].name, "scale");
        assert_eq!(parsed.params[0].label, "Scale");
        assert_eq!(parsed.params[0].offset, 0);
        assert_eq!(parsed.params[1].offset, 4);
        assert_eq!(parsed.size, 16);
        assert_eq!(parsed.params[0].default[0], 5.0);
        assert_eq!(
            parsed.params[0].widget,
            Widget::Slider {
                min: 0.1,
                max: 32.0,
                step: None
            }
        );
    }

    #[test]
    fn vec3_is_aligned_to_sixteen_bytes() {
        let parsed = layout(
            "struct Params {\n    scale: f32, // @ui 1.0 [0.0, 2.0]\n    tint: vec3<f32>, // @ui color srgb(1.0, 1.0, 1.0)\n    warp: f32, // @ui 0.5 [0.0, 1.0]\n}",
        );
        assert_eq!(parsed.params[0].offset, 0);
        assert_eq!(parsed.params[1].offset, 16);
        assert_eq!(parsed.params[2].offset, 28);
        assert_eq!(parsed.size, 32);
    }

    #[test]
    fn vec2_alignment() {
        let parsed = layout(
            "struct Params {\n    a: f32, // @ui 0.0 [0.0, 1.0]\n    b: vec2<f32>, // @ui vec (1.0, 2.0) [0.0, 4.0]\n}",
        );
        assert_eq!(parsed.params[1].offset, 8);
        assert_eq!(parsed.params[1].default[0], 1.0);
        assert_eq!(parsed.params[1].default[1], 2.0);
        assert_eq!(parsed.size, 16);
    }

    #[test]
    fn srgb_colours_are_converted_to_linear() {
        let parsed = layout(
            "struct Params {\n    tint: vec3<f32>, // @ui \"Tint\" color srgb(0.5, 0.5, 0.5)\n}",
        );
        assert_eq!(parsed.params[0].label, "Tint");
        assert_eq!(parsed.params[0].widget, Widget::Color);
        let expected = srgb_to_linear(0.5);
        assert!((parsed.params[0].default[0] - expected).abs() < 1e-6);
        assert!(parsed.params[0].default[0] < 0.5);
    }

    #[test]
    fn integers_get_a_unit_step_and_pack_as_integers() {
        let parsed = layout("struct Params {\n    octaves: i32, // @ui 4 [1, 8]\n}");
        assert_eq!(
            parsed.params[0].widget,
            Widget::Slider {
                min: 1.0,
                max: 8.0,
                step: Some(1.0)
            }
        );
        let packed = parsed.pack(&[[4.0, 0.0, 0.0, 0.0]]);
        assert_eq!(i32::from_ne_bytes(packed[0..4].try_into().unwrap()), 4);
    }

    #[test]
    fn toggle_and_hidden_are_recognised() {
        let parsed = layout(
            "struct Params {\n    flag: u32, // @ui toggle true\n    secret: f32, // @ui hidden 3.5\n}",
        );
        assert_eq!(parsed.params[0].widget, Widget::Toggle);
        assert_eq!(parsed.params[0].default[0], 1.0);
        assert_eq!(parsed.params[1].widget, Widget::Hidden);
        assert_eq!(parsed.params[1].default[0], 3.5);
        assert_eq!(parsed.groups().len(), 1);
        assert_eq!(parsed.groups()[0].1, vec![0]);
    }

    #[test]
    fn group_comments_split_the_panel() {
        let parsed = layout(
            "struct Params {\n    // @group Colour\n    tint: vec3<f32>, // @ui color srgb(1.0, 1.0, 1.0)\n    // @group Shape\n    scale: f32, // @ui 2.0 [0.0, 8.0]\n}",
        );
        let groups = parsed.groups();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].0, "Colour");
        assert_eq!(groups[1].0, "Shape");
    }

    #[test]
    fn packing_writes_every_component_at_its_offset() {
        let parsed = layout(
            "struct Params {\n    tint: vec3<f32>, // @ui color linear(0.1, 0.2, 0.3)\n    scale: f32, // @ui 7.0 [0.0, 8.0]\n}",
        );
        let packed = parsed.pack(&parsed.defaults());
        assert_eq!(packed.len(), 16);
        let read = |at: usize| f32::from_ne_bytes(packed[at..at + 4].try_into().unwrap());
        assert!((read(0) - 0.1).abs() < 1e-6);
        assert!((read(4) - 0.2).abs() < 1e-6);
        assert!((read(8) - 0.3).abs() < 1e-6);
        assert!((read(12) - 7.0).abs() < 1e-6);
    }

    #[test]
    fn unsupported_types_are_reported_with_a_line_number() {
        let error = parse_params("struct Params {\n    m: mat4x4<f32>,\n}").unwrap_err();
        assert!(error.message.contains("mat4x4"));
    }

    #[test]
    fn shipped_materials_parse() {
        for source in [
            include_str!("../../assets/materials/water.wgsl"),
            include_str!("../../assets/materials/rocky.wgsl"),
        ] {
            let parsed = parse_params(source).expect("shipped material should parse");
            assert!(!parsed.params.is_empty());
            assert_eq!(parsed.size % 16, 0);
            assert_eq!(parsed.pack(&parsed.defaults()).len(), parsed.size);
        }
    }
}
