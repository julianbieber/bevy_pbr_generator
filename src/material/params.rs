//! The `@ui`-annotated `struct Params` block of a material shader: its parsed
//! field list, the std140 offsets those fields occupy, and the packing of live
//! values into the uniform buffer the compute pass binds.

use std::fmt;

/// Size of the params uniform buffer, in bytes.
///
/// Fixed rather than sized to the parsed struct: a pipeline is queued before the
/// struct has been parsed, and a layout that later disagreed with the real
/// binding size would fail wgpu validation. Parsing a `Params` block larger than
/// this is a [`ParamError`].
pub const PARAMS_BUFFER_SIZE: usize = 1024;

/// A parse failure in a material's `Params` block, worded for the status bar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamError(pub String);

impl fmt::Display for ParamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ParamError {}

fn err<T>(message: impl Into<String>) -> Result<T, ParamError> {
    Err(ParamError(message.into()))
}

/// The WGSL type of a parameter field. Restricted to the scalars and vectors for
/// which std140 and std430 offsets agree, so the same packing serves both.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamType {
    F32,
    I32,
    U32,
    Vec2,
    Vec3,
    Vec4,
}

impl ParamType {
    fn parse(text: &str) -> Option<Self> {
        let compact: String = text.chars().filter(|c| !c.is_whitespace()).collect();
        match compact.as_str() {
            "f32" => Some(Self::F32),
            "i32" => Some(Self::I32),
            "u32" => Some(Self::U32),
            "vec2<f32>" => Some(Self::Vec2),
            "vec3<f32>" => Some(Self::Vec3),
            "vec4<f32>" => Some(Self::Vec4),
            _ => None,
        }
    }

    /// How many scalar components the type holds.
    pub fn components(self) -> usize {
        match self {
            Self::F32 | Self::I32 | Self::U32 => 1,
            Self::Vec2 => 2,
            Self::Vec3 => 3,
            Self::Vec4 => 4,
        }
    }

    /// Whether the components are written to the buffer as integers rather than
    /// floats.
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

/// How a field is presented in the parameter panel.
#[derive(Debug, Clone, PartialEq)]
pub enum Widget {
    /// One slider per component, all sharing the same bounds.
    Slider {
        min: f32,
        max: f32,
        step: Option<f32>,
    },
    /// A colour swatch and picker. The stored value is linear; the picker works
    /// in sRGB and converts on both edges.
    Color,
    /// A switch writing 0 or 1.
    Toggle,
    /// Not shown; the default is used as-is.
    Hidden,
}

/// One field of a material's `Params` block.
///
/// `default` holds `ty.components()` meaningful entries and is already in the
/// form written to the buffer — in particular a [`Widget::Color`] default has
/// been converted from the sRGB literal in the annotation to linear.
#[derive(Debug, Clone, PartialEq)]
pub struct ParamField {
    pub name: String,
    pub label: String,
    pub group: Option<String>,
    pub ty: ParamType,
    pub widget: Widget,
    pub default: [f32; 4],
    pub offset: usize,
}

/// The parsed `Params` block: every field in declaration order, with the std140
/// offsets they occupy.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParamsLayout {
    pub fields: Vec<ParamField>,
    /// Size of the struct in bytes, rounded up to 16. Always at most
    /// [`PARAMS_BUFFER_SIZE`].
    pub size: usize,
}

impl ParamsLayout {
    /// The default value of every field, in declaration order — the starting
    /// point for a freshly selected material.
    pub fn defaults(&self) -> Vec<[f32; 4]> {
        self.fields.iter().map(|f| f.default).collect()
    }

    /// Packs `values` into a [`PARAMS_BUFFER_SIZE`]-byte buffer at the parsed
    /// offsets. Entries beyond `self.fields` are ignored, and fields beyond
    /// `values` keep their default, so a stale value list from a previous
    /// version of the shader still produces a valid buffer.
    pub fn pack(&self, values: &[[f32; 4]]) -> Vec<u8> {
        let mut buffer = vec![0u8; PARAMS_BUFFER_SIZE];
        for (index, field) in self.fields.iter().enumerate() {
            let value = values.get(index).copied().unwrap_or(field.default);
            for (component, scalar) in value.iter().take(field.ty.components()).enumerate() {
                let raw = if field.ty.is_integer() {
                    (*scalar as i32).to_ne_bytes()
                } else {
                    scalar.to_ne_bytes()
                };
                let at = field.offset + component * 4;
                buffer[at..at + 4].copy_from_slice(&raw);
            }
        }
        buffer
    }
}

/// Converts one sRGB-encoded channel to linear, as the `srgb(...)` colour
/// annotation and the colour picker both require.
pub fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Inverse of [`srgb_to_linear`], for showing a stored colour in the picker.
pub fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// Parses the `struct Params` block out of a material shader.
///
/// Returns an error if the block is missing, if a field has a type outside
/// [`ParamType`], if its `@ui` annotation is absent or malformed, or if the
/// resulting struct exceeds [`PARAMS_BUFFER_SIZE`]. The error text names the
/// offending field and is written for display in the status bar.
pub fn parse_params(source: &str) -> Result<ParamsLayout, ParamError> {
    let body = params_block(source)?;

    let mut fields: Vec<ParamField> = Vec::new();
    let mut group: Option<String> = None;
    let mut offset = 0usize;

    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(comment) = line.strip_prefix("//") {
            if let Some(name) = comment.trim().strip_prefix("@group ") {
                group = Some(name.trim().to_string());
            }
            continue;
        }

        let (code, annotation) = match line.split_once("//") {
            Some((code, annotation)) => (code, Some(annotation.trim())),
            None => (line, None),
        };

        let code = code.trim().trim_end_matches(',').trim();
        if code.is_empty() {
            continue;
        }

        let Some((name, type_text)) = code.split_once(':') else {
            return err(format!("`{code}` is not a `name: type` field"));
        };
        let name = name.trim().to_string();

        let Some(ty) = ParamType::parse(type_text) else {
            return err(format!(
                "`{name}` has type `{}`, which is not a supported parameter type",
                type_text.trim()
            ));
        };

        let Some(annotation) = annotation else {
            return err(format!("`{name}` has no `@ui` annotation"));
        };
        let Some(spec) = annotation.strip_prefix("@ui") else {
            return err(format!("`{name}` has no `@ui` annotation"));
        };

        let (label, widget, default) = parse_annotation(&name, ty, spec.trim())?;

        offset = align_up(offset, ty.align());
        fields.push(ParamField {
            name,
            label,
            group: group.clone(),
            ty,
            widget,
            default,
            offset,
        });
        offset += ty.size();
    }

    if fields.is_empty() {
        return err("`struct Params` declares no fields");
    }

    let size = align_up(offset, 16);
    if size > PARAMS_BUFFER_SIZE {
        return err(format!(
            "`struct Params` is {size} bytes, over the {PARAMS_BUFFER_SIZE}-byte limit"
        ));
    }

    Ok(ParamsLayout { fields, size })
}

fn align_up(value: usize, align: usize) -> usize {
    value.div_ceil(align) * align
}

fn params_block(source: &str) -> Result<&str, ParamError> {
    let mut search = source;
    let mut consumed = 0usize;

    // `struct Params` may appear inside a comment or as a prefix of a longer
    // name, so keep looking until a candidate is followed by an opening brace.
    loop {
        let Some(found) = search.find("struct Params") else {
            return err("no `struct Params` block found");
        };
        let after = &search[found + "struct Params".len()..];
        if let Some(open) = after.find('{') {
            if after[..open].trim().is_empty() {
                let body_start = consumed + found + "struct Params".len() + open + 1;
                let rest = &source[body_start..];
                let Some(close) = rest.find('}') else {
                    return err("`struct Params` block is not closed");
                };
                return Ok(&rest[..close]);
            }
        }
        consumed += found + "struct Params".len();
        search = after;
    }
}

fn parse_annotation(
    name: &str,
    ty: ParamType,
    spec: &str,
) -> Result<(String, Widget, [f32; 4]), ParamError> {
    let (label, rest) = match spec.strip_prefix('"') {
        Some(after) => match after.split_once('"') {
            Some((label, rest)) => (label.to_string(), rest.trim()),
            None => return err(format!("`{name}` has an unterminated label")),
        },
        None => (name.replace('_', " "), spec),
    };

    if rest == "hidden" {
        return Ok((label, Widget::Hidden, [0.0; 4]));
    }

    if let Some(value) = rest.strip_prefix("toggle") {
        let value = value.trim();
        let on = match value {
            "true" => 1.0,
            "false" => 0.0,
            _ => {
                return err(format!(
                    "`{name}` toggle wants `true` or `false`, got `{value}`"
                ))
            }
        };
        if ty.components() != 1 {
            return err(format!("`{name}` is a toggle but not a scalar"));
        }
        return Ok((label, Widget::Toggle, [on, 0.0, 0.0, 0.0]));
    }

    if let Some(rest) = rest.strip_prefix("color") {
        let rest = rest.trim();
        let Some(inner) = rest.strip_prefix("srgb") else {
            return err(format!("`{name}` colour wants `srgb(...)`"));
        };
        let (components, tail) = parse_group(name, inner.trim(), '(', ')')?;
        if !tail.trim().is_empty() {
            return err(format!("`{name}` has trailing text after its colour"));
        }
        if !matches!(ty, ParamType::Vec3 | ParamType::Vec4) {
            return err(format!("`{name}` is a colour but not a vec3 or vec4"));
        }
        if components.len() != 3 {
            return err(format!(
                "`{name}` colour wants 3 components, got {}",
                components.len()
            ));
        }
        let mut default = [0.0f32; 4];
        for (i, c) in components.iter().enumerate() {
            default[i] = srgb_to_linear(*c);
        }
        if ty == ParamType::Vec4 {
            default[3] = 1.0;
        }
        return Ok((label, Widget::Color, default));
    }

    let rest = rest.strip_prefix("vec").map(str::trim).unwrap_or(rest);

    let (components, rest) = if rest.starts_with('(') {
        parse_group(name, rest, '(', ')')?
    } else {
        let end = rest
            .find(|c: char| c == '[' || c.is_whitespace())
            .unwrap_or(rest.len());
        let (value, tail) = rest.split_at(end);
        (vec![parse_number(name, value)?], tail)
    };

    if components.len() != ty.components() {
        return err(format!(
            "`{name}` wants {} default components, got {}",
            ty.components(),
            components.len()
        ));
    }

    let rest = rest.trim();
    if !rest.starts_with('[') {
        return err(format!("`{name}` has no `[min, max]` range"));
    }
    let (bounds, rest) = parse_group(name, rest, '[', ']')?;
    if bounds.len() != 2 {
        return err(format!(
            "`{name}` wants a `[min, max]` range, got {} values",
            bounds.len()
        ));
    }
    if bounds[0] > bounds[1] {
        return err(format!("`{name}` has a range whose min exceeds its max"));
    }

    let rest = rest.trim();
    let step = if let Some(value) = rest.strip_prefix("step") {
        Some(parse_number(name, value.trim())?)
    } else if rest.is_empty() {
        None
    } else {
        return err(format!("`{name}` has trailing text `{rest}`"));
    };

    let mut default = [0.0f32; 4];
    for (i, c) in components.iter().enumerate() {
        default[i] = c.clamp(bounds[0], bounds[1]);
    }

    Ok((
        label,
        Widget::Slider {
            min: bounds[0],
            max: bounds[1],
            step,
        },
        default,
    ))
}

fn parse_group<'a>(
    name: &str,
    text: &'a str,
    open: char,
    close: char,
) -> Result<(Vec<f32>, &'a str), ParamError> {
    let Some(body) = text.strip_prefix(open) else {
        return err(format!("`{name}` expected `{open}`"));
    };
    let Some(end) = body.find(close) else {
        return err(format!("`{name}` has an unclosed `{open}`"));
    };
    let (inner, rest) = body.split_at(end);
    let mut values = Vec::new();
    for piece in inner.split(',') {
        values.push(parse_number(name, piece.trim())?);
    }
    Ok((values, &rest[1..]))
}

fn parse_number(name: &str, text: &str) -> Result<f32, ParamError> {
    text.trim()
        .parse::<f32>()
        .map_err(|_| ParamError(format!("`{name}` has `{text}` where a number was expected")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout(fields: &str) -> ParamsLayout {
        parse_params(&format!("struct Params {{\n{fields}\n}}\n")).expect("should parse")
    }

    fn field<'a>(layout: &'a ParamsLayout, name: &str) -> &'a ParamField {
        layout
            .fields
            .iter()
            .find(|f| f.name == name)
            .expect("field should exist")
    }

    // Pins the std140 rule the packer depends on: a vec3 leaves a 4-byte hole
    // that the following scalar fills, so offsets are not simply cumulative
    // sizes.
    #[test]
    fn vec3_is_followed_by_a_scalar_in_its_padding() {
        let l = layout(
            "    tint: vec3<f32>, // @ui color srgb(1.0, 1.0, 1.0)\n\
             \x20   scale: f32,     // @ui 1.0 [0.0, 2.0]",
        );
        assert_eq!(field(&l, "tint").offset, 0);
        assert_eq!(field(&l, "scale").offset, 12);
        assert_eq!(l.size, 16);
    }

    // Pins each type's alignment against a layout that exercises all of them.
    #[test]
    fn offsets_follow_std140_alignment() {
        let l = layout(
            "    a: f32,         // @ui 0.0 [0.0, 1.0]\n\
             \x20   b: vec2<f32>,   // @ui (0.0, 0.0) [0.0, 1.0]\n\
             \x20   c: f32,         // @ui 0.0 [0.0, 1.0]\n\
             \x20   d: vec4<f32>,   // @ui vec (0.0, 0.0, 0.0, 0.0) [0.0, 1.0]\n\
             \x20   e: i32,         // @ui 0 [0, 4]",
        );
        assert_eq!(field(&l, "a").offset, 0);
        assert_eq!(field(&l, "b").offset, 8);
        assert_eq!(field(&l, "c").offset, 16);
        assert_eq!(field(&l, "d").offset, 32);
        assert_eq!(field(&l, "e").offset, 48);
        assert_eq!(l.size, 64);
    }

    // The colour annotation is written in sRGB but the pipeline is linear
    // throughout, so the stored default must already be converted.
    #[test]
    fn colour_defaults_are_stored_linear() {
        let l = layout("    tint: vec3<f32>, // @ui color srgb(0.5, 0.5, 0.5)");
        let stored = field(&l, "tint").default[0];
        assert!((stored - srgb_to_linear(0.5)).abs() < 1e-6);
        assert!(
            stored < 0.5,
            "linear value should be darker than its sRGB literal"
        );
    }

    // A vec4 colour has no alpha in the annotation; it must still be opaque
    // rather than the zero the buffer is otherwise cleared to.
    #[test]
    fn vec4_colour_defaults_to_opaque() {
        let l = layout("    tint: vec4<f32>, // @ui color srgb(0.0, 0.0, 0.0)");
        assert_eq!(field(&l, "tint").default[3], 1.0);
    }

    // Every widget form in the grammar must parse, since a material using one
    // that does not is rejected wholesale.
    #[test]
    fn every_widget_form_parses() {
        let l = layout(
            "    s: f32,       // @ui 1.0 [0.0, 2.0]\n\
             \x20   t: f32,       // @ui 1.0 [0.0, 2.0] step 0.25\n\
             \x20   u: vec2<f32>, // @ui (1.0, 2.0) [0.0, 4.0]\n\
             \x20   v: vec3<f32>, // @ui color srgb(1.0, 0.0, 0.0)\n\
             \x20   w: vec3<f32>, // @ui vec (1.0, 2.0, 3.0) [0.0, 4.0]\n\
             \x20   x: u32,       // @ui toggle true\n\
             \x20   y: i32,       // @ui hidden\n\
             \x20   z: f32,       // @ui \"Custom\" 1.0 [0.0, 2.0]",
        );
        assert!(matches!(
            field(&l, "t").widget,
            Widget::Slider {
                step: Some(s),
                ..
            } if s == 0.25
        ));
        assert_eq!(field(&l, "u").default[1], 2.0);
        assert_eq!(field(&l, "v").widget, Widget::Color);
        assert_eq!(field(&l, "w").default[2], 3.0);
        assert_eq!(field(&l, "x").widget, Widget::Toggle);
        assert_eq!(field(&l, "x").default[0], 1.0);
        assert_eq!(field(&l, "y").widget, Widget::Hidden);
        assert_eq!(field(&l, "z").label, "Custom");
    }

    // Without an explicit label the panel shows the field name with underscores
    // turned into spaces, so a material need not label every field.
    #[test]
    fn label_defaults_to_the_field_name() {
        let l = layout("    wave_scale: f32, // @ui 1.0 [0.0, 2.0]");
        assert_eq!(field(&l, "wave_scale").label, "wave scale");
    }

    // `@group` opens a section that applies to every following field until the
    // next one, which is what makes the panel collapsible.
    #[test]
    fn group_applies_until_the_next_group() {
        let l = layout(
            "    // @group Colour\n\
             \x20   a: f32, // @ui 0.0 [0.0, 1.0]\n\
             \x20   b: f32, // @ui 0.0 [0.0, 1.0]\n\
             \x20   // @group Waves\n\
             \x20   c: f32, // @ui 0.0 [0.0, 1.0]",
        );
        assert_eq!(field(&l, "a").group.as_deref(), Some("Colour"));
        assert_eq!(field(&l, "b").group.as_deref(), Some("Colour"));
        assert_eq!(field(&l, "c").group.as_deref(), Some("Waves"));
    }

    // Integers are written as integer bits, not as the float the value is
    // carried in, or the shader reads a wildly wrong number.
    #[test]
    fn integers_pack_as_integer_bits() {
        let l = layout("    octaves: i32, // @ui 4 [1, 8]");
        let packed = l.pack(&l.defaults());
        assert_eq!(i32::from_ne_bytes(packed[0..4].try_into().unwrap()), 4);
    }

    // A shader edit can leave the UI holding fewer values than the new struct
    // has fields; the missing ones must fall back to defaults rather than
    // panicking or writing zeroes.
    #[test]
    fn packing_a_short_value_list_falls_back_to_defaults() {
        let l = layout(
            "    a: f32, // @ui 1.0 [0.0, 2.0]\n\
             \x20   b: f32, // @ui 2.0 [0.0, 4.0]",
        );
        let packed = l.pack(&[[9.0, 0.0, 0.0, 0.0]]);
        assert_eq!(f32::from_ne_bytes(packed[0..4].try_into().unwrap()), 9.0);
        assert_eq!(f32::from_ne_bytes(packed[4..8].try_into().unwrap()), 2.0);
    }

    // The buffer is a fixed size regardless of the struct, because the bind
    // group layout is built before the struct is known.
    #[test]
    fn packing_always_fills_the_fixed_buffer() {
        let l = layout("    a: f32, // @ui 1.0 [0.0, 2.0]");
        assert_eq!(l.pack(&l.defaults()).len(), PARAMS_BUFFER_SIZE);
    }

    // A default outside its own declared range would put the slider in an
    // impossible state on first load.
    #[test]
    fn defaults_are_clamped_into_range() {
        let l = layout("    a: f32, // @ui 9.0 [0.0, 1.0]");
        assert_eq!(field(&l, "a").default[0], 1.0);
    }

    // Each of these is a mistake a material author will actually make, and each
    // must be reported rather than silently producing a wrong buffer.
    #[test]
    fn malformed_annotations_are_rejected() {
        for source in [
            "struct Other { a: f32 }",
            "struct Params { a: f32, }",
            "struct Params { a: mat4x4<f32>, // @ui 1.0 [0.0, 2.0]\n }",
            "struct Params { a: f32, // @ui 1.0\n }",
            "struct Params { a: f32, // @ui 1.0 [2.0, 0.0]\n }",
            "struct Params { a: vec3<f32>, // @ui color srgb(1.0, 1.0)\n }",
            "struct Params { a: f32, // @ui color srgb(1.0, 1.0, 1.0)\n }",
            "struct Params { a: f32, // @ui toggle yes\n }",
            "struct Params { a: vec2<f32>, // @ui 1.0 [0.0, 2.0]\n }",
            "struct Params { }",
        ] {
            assert!(
                parse_params(source).is_err(),
                "should have been rejected: {source}"
            );
        }
    }

    // `struct Params` also names the binding declaration on the following line,
    // so the scanner must not stop at the first textual match.
    #[test]
    fn the_binding_declaration_is_not_mistaken_for_the_block() {
        let source = "\
struct Params {
    a: f32, // @ui 1.0 [0.0, 2.0]
}
@group(1) @binding(0) var<uniform> params: Params;
";
        assert_eq!(parse_params(source).expect("should parse").fields.len(), 1);
    }
}
