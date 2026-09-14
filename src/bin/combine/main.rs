//! `combine`: packs material export folders into one image per map, so a
//! single mesh can address each material through its own UV region.

mod compose;
mod layout;

use std::ffi::OsString;
use std::fmt;
use std::path::PathBuf;
use std::process::ExitCode;

use image::ColorType;

use crate::compose::Material;
use crate::layout::Layout;

const SIZE_LIMIT: u32 = 8192;
const DEFAULT_OUTPUT: &str = "output/combined";
const MANIFEST: &str = "atlas.json";
const USAGE: &str = "usage: combine [-o <dir>] <material folder>...

Combines material export folders, left to right in the order given, into one
PNG per map plus atlas.json. Defaults to -o output/combined.";

/// Everything that stops a combine. Each variant names the path or value at
/// fault.
#[derive(Debug)]
pub enum CombineError {
    Usage(String),
    NoMaterials,
    FolderNotFound(PathBuf),
    UnusableName(PathBuf),
    DuplicateName(String),
    MissingMap {
        folder: PathBuf,
        file: PathBuf,
    },
    Unreadable {
        file: PathBuf,
        reason: String,
    },
    NotSquare {
        file: PathBuf,
        width: u32,
        height: u32,
    },
    NotRgba8 {
        file: PathBuf,
        color: ColorType,
    },
    TooLarge {
        count: usize,
        cell: u32,
        limit: u32,
    },
    OutputIsInput(PathBuf),
    WriteFailed {
        file: PathBuf,
        reason: String,
    },
}

impl fmt::Display for CombineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) => write!(f, "{message}"),
            Self::NoMaterials => write!(f, "no material folder given"),
            Self::FolderNotFound(folder) => {
                write!(f, "material folder {} does not exist", folder.display())
            }
            Self::UnusableName(folder) => write!(
                f,
                "{} has no UTF-8 folder name to name the material after",
                folder.display()
            ),
            Self::DuplicateName(name) => {
                write!(f, "two material folders are both named {name:?}")
            }
            Self::MissingMap { folder, file } => write!(
                f,
                "material folder {} is missing {}",
                folder.display(),
                file.display()
            ),
            Self::Unreadable { file, reason } => {
                write!(f, "cannot read {}: {reason}", file.display())
            }
            Self::NotSquare {
                file,
                width,
                height,
            } => write!(f, "{} is {width}x{height}, not square", file.display()),
            Self::NotRgba8 { file, color } => {
                write!(f, "{} is {color:?}, not 8-bit RGBA", file.display())
            }
            Self::TooLarge { count, cell, limit } => write!(
                f,
                "{count} materials at {cell}px do not fit in {limit}x{limit}"
            ),
            Self::OutputIsInput(folder) => write!(
                f,
                "output directory {} is one of the material folders",
                folder.display()
            ),
            Self::WriteFailed { file, reason } => {
                write!(f, "cannot write {}: {reason}", file.display())
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Command {
    Help,
    Combine(CombineArgs),
}

#[derive(Debug, PartialEq, Eq)]
struct CombineArgs {
    output_dir: PathBuf,
    folders: Vec<PathBuf>,
}

fn main() -> ExitCode {
    let command = match parse_args(std::env::args_os().skip(1)) {
        Ok(command) => command,
        Err(error) => {
            eprintln!("combine: {error}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    let args = match command {
        Command::Help => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        Command::Combine(args) => args,
    };
    match combine(&args) {
        Ok((layout, materials)) => {
            print_regions(&args, &layout, &materials);
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("combine: {error}");
            ExitCode::FAILURE
        }
    }
}

fn parse_args(args: impl Iterator<Item = OsString>) -> Result<Command, CombineError> {
    let mut output_dir = None;
    let mut folders = Vec::new();
    let mut args = args;
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("-h" | "--help") => return Ok(Command::Help),
            Some("-o") => {
                let dir = args
                    .next()
                    .ok_or_else(|| CombineError::Usage("-o needs a directory".to_string()))?;
                if output_dir.replace(PathBuf::from(dir)).is_some() {
                    return Err(CombineError::Usage("-o given more than once".to_string()));
                }
            }
            Some(flag) if flag.starts_with('-') => {
                return Err(CombineError::Usage(format!("unknown option {flag}")));
            }
            _ => folders.push(PathBuf::from(arg)),
        }
    }
    if folders.is_empty() {
        return Err(CombineError::NoMaterials);
    }
    Ok(Command::Combine(CombineArgs {
        output_dir: output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_OUTPUT)),
        folders,
    }))
}

fn combine(args: &CombineArgs) -> Result<(Layout, Vec<Material>), CombineError> {
    let materials = compose::inspect(&args.folders)?;
    let layout = Layout::place(materials.len(), compose::cell_size(&materials), SIZE_LIMIT)?;
    compose::check_output(&args.output_dir, &materials)?;
    let manifest = manifest_json(&layout, &materials);
    compose::write_all(&args.output_dir, &materials, &layout, MANIFEST, &manifest)?;
    Ok((layout, materials))
}

fn manifest_json(layout: &Layout, materials: &[Material]) -> String {
    let entries: Vec<String> = materials
        .iter()
        .enumerate()
        .map(|(index, material)| {
            let region = layout.region(index);
            format!(
                "    {{ \"name\": {}, \"u_min\": {}, \"u_max\": {}, \"v_min\": {}, \"v_max\": {} }}",
                json_string(&material.name),
                region.u_min,
                region.u_max,
                region.v_min,
                region.v_max
            )
        })
        .collect();
    format!(
        "{{\n  \"columns\": {},\n  \"rows\": {},\n  \"cell_size\": {},\n  \"width\": {},\n  \"height\": {},\n  \"materials\": [\n{}\n  ]\n}}\n",
        layout.columns,
        layout.rows,
        layout.cell,
        layout.width(),
        layout.height(),
        entries.join(",\n")
    )
}

fn json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if u32::from(c) < 0x20 => out.push_str(&format!("\\u{:04x}", u32::from(c))),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn print_regions(args: &CombineArgs, layout: &Layout, materials: &[Material]) {
    println!(
        "wrote {}x{} ({}x{} cells of {}px) to {}",
        layout.width(),
        layout.height(),
        layout.columns,
        layout.rows,
        layout.cell,
        args.output_dir.display()
    );
    for (index, material) in materials.iter().enumerate() {
        let region = layout.region(index);
        println!(
            "  {:<24} u {}..{}  v {}..{}",
            material.name, region.u_min, region.u_max, region.v_min, region.v_max
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compose::tests::{export_folder, scratch};

    fn args(list: &[&str]) -> Result<Command, CombineError> {
        parse_args(list.iter().map(OsString::from))
    }

    // Folders are kept in the order given, since that order fixes every UV region.
    #[test]
    fn parses_folders_in_order_with_default_output() {
        assert_eq!(
            args(&["output/water", "output/rocky"]).unwrap(),
            Command::Combine(CombineArgs {
                output_dir: PathBuf::from(DEFAULT_OUTPUT),
                folders: vec!["output/water".into(), "output/rocky".into()],
            })
        );
        assert!(matches!(
            args(&["-o", "atlas", "water"]).unwrap(),
            Command::Combine(CombineArgs { output_dir, .. }) if output_dir == std::path::Path::new("atlas")
        ));
    }

    // Malformed command lines are usage errors, not silently ignored flags.
    #[test]
    fn rejects_malformed_command_lines() {
        assert!(matches!(args(&[]), Err(CombineError::NoMaterials)));
        assert!(matches!(
            args(&["water", "-o"]),
            Err(CombineError::Usage(_))
        ));
        assert!(matches!(
            args(&["-x", "water"]),
            Err(CombineError::Usage(_))
        ));
        assert!(matches!(
            args(&["-o", "a", "-o", "b", "water"]),
            Err(CombineError::Usage(_))
        ));
        assert_eq!(args(&["water", "--help"]).unwrap(), Command::Help);
    }

    // Folder names reach the manifest verbatim, so JSON metacharacters must be escaped.
    #[test]
    fn json_strings_are_escaped() {
        assert_eq!(json_string("water"), "\"water\"");
        assert_eq!(json_string("a\"b\\c\n"), "\"a\\\"b\\\\c\\u000a\"");
    }

    // End to end: two materials of different resolutions become a 2:1 row, resized to the larger, with matching manifest.
    #[test]
    fn combines_two_materials_into_a_row() {
        let root = scratch("end-to-end");
        let water = export_folder(&root, "water", 8, [0, 0, 255, 255]);
        let rocky = export_folder(&root, "rocky", 4, [128, 128, 128, 255]);
        let output = root.join("combined");
        let (layout, _) = combine(&CombineArgs {
            output_dir: output.clone(),
            folders: vec![water, rocky],
        })
        .unwrap();
        assert_eq!((layout.columns, layout.rows, layout.cell), (2, 1, 8));

        let base = image::open(output.join("base_color.png"))
            .unwrap()
            .into_rgba8();
        assert_eq!(base.dimensions(), (16, 8));
        assert_eq!(base.get_pixel(3, 3).0, [0, 0, 255, 255]);
        assert_eq!(base.get_pixel(12, 3).0, [128, 128, 128, 255]);

        let manifest = std::fs::read_to_string(output.join(MANIFEST)).unwrap();
        assert!(manifest.contains(
            "{ \"name\": \"water\", \"u_min\": 0, \"u_max\": 0.5, \"v_min\": 0, \"v_max\": 1 }"
        ));
        assert!(manifest.contains(
            "{ \"name\": \"rocky\", \"u_min\": 0.5, \"u_max\": 1, \"v_min\": 0, \"v_max\": 1 }"
        ));
        let files = std::fs::read_dir(&output).unwrap().count();
        assert_eq!(files, bevy_pbr_generator::gpu::maps::MAP_COUNT + 1);
    }
}
