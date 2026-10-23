//! Setting up a material project from the installed binary alone: the copy of
//! the WESL package its language server resolves, and new material files.

use std::fmt;
use std::path::{Path, PathBuf};

use wesl::toml_cfg::{DependenciesConfig, DependencySpec, WeslToml};

use crate::material::project::MaterialProject;
use crate::shaders::PACKAGE;

/// The package modules this binary was built with, as file name and source.
pub const PACKAGE_FILES: [(&str, &str); 2] = [
    ("pbr_maps.wesl", include_str!("shaders/pbr_maps.wesl")),
    ("noise.wesl", include_str!("shaders/noise.wesl")),
];

/// The directory, beside a project's `wesl.toml`, that [`init`] copies the
/// package into.
pub const VENDOR_DIR: &str = PACKAGE;

const TEMPLATE: &str = include_str!("../example/shaders/_template.wesl");

const VENDOR_TOML: &str =
    "# Written by `bevy_pbr_generator init`, which overwrites this directory.\n\
edition = \"2026_pre\"\n\
root = \".\"\n";

/// Why a project could not be set up. The message names the path at fault.
#[derive(Debug)]
pub struct ScaffoldError(pub String);

impl fmt::Display for ScaffoldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ScaffoldError {}

/// What [`init`] did, for the caller to report.
#[derive(Debug, PartialEq, Eq)]
pub struct InitReport {
    /// The `wesl.toml` written, or `None` when one already existed.
    pub created_toml: Option<PathBuf>,
    /// The directory the package was copied into.
    pub vendored: PathBuf,
    /// Set when an existing `wesl.toml` does not point its `bevy_pbr_generator`
    /// dependency at [`VENDOR_DIR`]; the language server then resolves another
    /// copy than the editor runs.
    pub dependency_mismatch: bool,
}

/// Makes `dir` a material project for this binary's version of the package.
///
/// Creates `dir` if missing. Writes a `wesl.toml` with a `shaders/` package
/// root only when `dir` has none; an existing one is left untouched. Always
/// overwrites the package files under `dir/`[`VENDOR_DIR`], so running it again
/// after upgrading the binary brings the language server up to date.
pub fn init(dir: &Path) -> Result<InitReport, ScaffoldError> {
    std::fs::create_dir_all(dir).map_err(|e| io(dir, e))?;
    let toml_path = dir.join("wesl.toml");
    let created_toml = if toml_path.exists() {
        None
    } else {
        let root = dir.join("shaders");
        std::fs::create_dir_all(&root).map_err(|e| io(&root, e))?;
        let toml = format!(
            "edition = \"2026_pre\"\n\
             root = \"shaders\"\n\
             \n\
             [dependencies]\n\
             {PACKAGE} = {{ path = \"{VENDOR_DIR}\" }}\n"
        );
        std::fs::write(&toml_path, toml).map_err(|e| io(&toml_path, e))?;
        Some(toml_path.clone())
    };

    let vendored = dir.join(VENDOR_DIR);
    std::fs::create_dir_all(&vendored).map_err(|e| io(&vendored, e))?;
    let files = [("wesl.toml", VENDOR_TOML)]
        .into_iter()
        .chain(PACKAGE_FILES);
    for (name, source) in files {
        let path = vendored.join(name);
        std::fs::write(&path, source).map_err(|e| io(&path, e))?;
    }

    let config = WeslToml::from_file(&toml_path)
        .map_err(|e| ScaffoldError(format!("`{}`: {e}", toml_path.display())))?;
    let dependency_mismatch = !matches!(
        dependency_path(&config).as_deref(),
        Some(path) if Path::new(path) == Path::new(VENDOR_DIR)
    );
    Ok(InitReport {
        created_toml,
        vendored,
        dependency_mismatch,
    })
}

/// Writes `<package root>/<name>.wesl` in the project at `dir` from the
/// material template, declared as `@material <name>`, and answers its path.
///
/// Fails when `dir` is not a material project, `name` is not a WESL identifier
/// (it names a module), or the file already exists.
pub fn new_material(dir: &Path, name: &str) -> Result<PathBuf, ScaffoldError> {
    if !is_identifier(name) {
        return Err(ScaffoldError(format!(
            "`{name}` is not a WESL identifier, so it cannot name a module"
        )));
    }
    let project = MaterialProject::open(dir).map_err(|e| ScaffoldError(e.0))?;
    let target = project.package_root.join(format!("{name}.wesl"));
    if target.exists() {
        return Err(ScaffoldError(format!(
            "`{}` already exists",
            target.display()
        )));
    }
    let source = TEMPLATE.replacen("@material Template", &format!("@material {name}"), 1);
    std::fs::write(&target, source).map_err(|e| io(&target, e))?;
    Ok(target)
}

/// Whether the copy of the package the project's language server resolves
/// differs from the one this binary embeds.
///
/// Answers `false` when the project declares no path dependency on the package
/// or the copy cannot be read, since there is then nothing to compare.
pub fn package_is_stale(project: &MaterialProject) -> bool {
    let Ok(config) = WeslToml::from_file(project.toml_dir.join("wesl.toml")) else {
        return false;
    };
    let Some(path) = dependency_path(&config) else {
        return false;
    };
    let Ok(dependency) = MaterialProject::open(&project.toml_dir.join(path)) else {
        return false;
    };
    PACKAGE_FILES.iter().any(|(name, source)| {
        std::fs::read_to_string(dependency.package_root.join(name))
            .is_ok_and(|on_disk| on_disk != *source)
    })
}

fn dependency_path(config: &WeslToml) -> Option<String> {
    let DependenciesConfig::Manual(dependencies) = &config.dependencies else {
        return None;
    };
    match dependencies.get(PACKAGE)? {
        DependencySpec::Path(path) => Some(path.clone()),
        _ => None,
    }
}

fn is_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn io(path: &Path, e: std::io::Error) -> ScaffoldError {
    ScaffoldError(format!("`{}`: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("pbr_gen_scaffold_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    // Pins that a fresh init yields a project the editor opens, whose only
    // dependency is the vendored copy, and which is not reported stale.
    #[test]
    fn init_makes_an_openable_project_with_a_current_package() {
        let dir = scratch("init");
        let report = init(&dir).unwrap();
        assert_eq!(report.created_toml, Some(dir.join("wesl.toml")));
        assert!(!report.dependency_mismatch);
        let project = MaterialProject::open(&dir).unwrap();
        assert!(project.foreign_dependencies().is_empty());
        assert!(!package_is_stale(&project));
        std::fs::remove_dir_all(dir).unwrap();
    }

    // Pins that re-running init after an upgrade refreshes the copy but never
    // rewrites a wesl.toml the user may have edited.
    #[test]
    fn init_again_refreshes_the_package_and_keeps_wesl_toml() {
        let dir = scratch("reinit");
        init(&dir).unwrap();
        std::fs::write(dir.join("wesl.toml"), "edition = \"2026_pre\"\nroot = \"shaders\"\n# mine\n[dependencies]\nbevy_pbr_generator = { path = \"bevy_pbr_generator\" }\n").unwrap();
        std::fs::write(dir.join(VENDOR_DIR).join("noise.wesl"), "old").unwrap();
        assert!(package_is_stale(&MaterialProject::open(&dir).unwrap()));

        let report = init(&dir).unwrap();
        assert_eq!(report.created_toml, None);
        assert!(std::fs::read_to_string(dir.join("wesl.toml"))
            .unwrap()
            .contains("# mine"));
        assert!(!package_is_stale(&MaterialProject::open(&dir).unwrap()));
        std::fs::remove_dir_all(dir).unwrap();
    }

    // Pins that init flags a wesl.toml whose dependency points elsewhere, since
    // the language server would then disagree with the editor.
    #[test]
    fn init_reports_a_dependency_pointing_elsewhere() {
        let dir = scratch("elsewhere");
        std::fs::create_dir_all(dir.join("shaders")).unwrap();
        std::fs::write(dir.join("wesl.toml"), "edition = \"2026_pre\"\n[dependencies]\nbevy_pbr_generator = { path = \"../checkout\" }\n").unwrap();
        assert!(init(&dir).unwrap().dependency_mismatch);
        std::fs::remove_dir_all(dir).unwrap();
    }

    // Pins that a new material lands at the package root under its own name
    // and refuses to overwrite or take a name no module can have.
    #[test]
    fn new_material_writes_the_template_once_under_a_valid_name() {
        let dir = scratch("new");
        init(&dir).unwrap();
        let path = new_material(&dir, "slate").unwrap();
        assert_eq!(
            path,
            MaterialProject::open(&dir)
                .unwrap()
                .package_root
                .join("slate.wesl")
        );
        assert!(std::fs::read_to_string(&path)
            .unwrap()
            .starts_with("// @material slate\n"));
        assert!(new_material(&dir, "slate").is_err());
        assert!(new_material(&dir, "my-slate").is_err());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
