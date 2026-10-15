//! The material project the editor is opened on: a directory holding a
//! `wesl.toml`, read through the wesl crate's implementation of its rules.

use std::fmt;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use wesl::error::TomlError;
use wesl::package::PackageModule;
use wesl::toml_cfg::{scan_from_config, DependenciesConfig, WeslToml};

use crate::shaders::PACKAGE;

/// Why a directory could not be opened as a material project.
#[derive(Debug)]
pub struct ProjectError(pub String);

impl fmt::Display for ProjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ProjectError {}

/// A material project: the directory its `wesl.toml` sits in, and the package
/// root that `package::` resolves against.
#[derive(Resource, Debug, Clone)]
pub struct MaterialProject {
    /// The directory holding `wesl.toml`, absolute.
    pub toml_dir: PathBuf,
    /// The package root, absolute; the editor's asset root while it is open.
    pub package_root: PathBuf,
    config: WeslToml,
}

/// One WESL module of a project, named by the `wesl.toml` rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectFile {
    /// The module path below `package::`, one entry per segment.
    pub components: Vec<String>,
    pub source: String,
}

impl ProjectFile {
    /// The asset path of the file, relative to the package root.
    pub fn asset_path(&self) -> String {
        format!("{}.wesl", self.components.join("/"))
    }

    /// The module path, written as an import would name it.
    pub fn module_path(&self) -> String {
        format!("package::{}", self.components.join("::"))
    }
}

impl MaterialProject {
    /// Opens the project whose `wesl.toml` is directly inside `dir`.
    ///
    /// `dir` may be relative to the current directory. Fails when `dir` does not
    /// exist, holds no `wesl.toml`, or the file does not parse; the error names
    /// the directory.
    pub fn open(dir: &Path) -> Result<Self, ProjectError> {
        let toml_dir = std::fs::canonicalize(dir)
            .map_err(|e| ProjectError(format!("cannot open project `{}`: {e}", dir.display())))?;
        let toml_path = toml_dir.join("wesl.toml");
        let config = WeslToml::from_file(&toml_path).map_err(|e| {
            ProjectError(format!(
                "`{}` is not a WESL project: {e}",
                toml_dir.display()
            ))
        })?;
        let package_root = toml_dir.join(&config.package.root);
        let package_root = std::fs::canonicalize(&package_root).map_err(|e| {
            ProjectError(format!(
                "package root `{}` of `{}`: {e}",
                package_root.display(),
                toml_path.display()
            ))
        })?;
        Ok(Self {
            toml_dir,
            package_root,
            config,
        })
    }

    /// The names of dependencies other than this package that the project's
    /// `wesl.toml` declares, including this package declared under another
    /// name. The editor resolves none of them, so a material importing one never
    /// compiles.
    pub fn foreign_dependencies(&self) -> Vec<String> {
        let DependenciesConfig::Manual(dependencies) = &self.config.dependencies else {
            return Vec::new();
        };
        let mut names: Vec<String> = dependencies
            .keys()
            .filter(|name| name.as_str() != PACKAGE)
            .cloned()
            .collect();
        names.sort();
        names
    }

    /// Every `.wesl` module the project's `wesl.toml` includes, in module path
    /// order, with the warnings for files the scan skipped.
    ///
    /// A project with no included file answers an empty list rather than an
    /// error; any other scan failure is an error.
    pub fn files(&self) -> Result<(Vec<ProjectFile>, Vec<String>), ProjectError> {
        let scan = match scan_from_config(PACKAGE, &self.toml_dir, &self.config) {
            Ok(scan) => scan,
            Err(TomlError::NoFilesMatched) => return Ok((Vec::new(), Vec::new())),
            Err(e) => {
                return Err(ProjectError(format!(
                    "cannot scan `{}`: {e}",
                    self.toml_dir.display()
                )))
            }
        };

        let mut files = Vec::new();
        for module in &scan.module.submodules {
            self.collect(module, &mut Vec::new(), &mut files);
        }
        files.sort_by(|a, b| a.components.cmp(&b.components));
        let warnings = scan.warnings.iter().map(ToString::to_string).collect();
        Ok((files, warnings))
    }

    fn collect(
        &self,
        module: &PackageModule,
        parents: &mut Vec<String>,
        out: &mut Vec<ProjectFile>,
    ) {
        parents.push(module.name.clone());
        let file = ProjectFile {
            components: parents.clone(),
            source: module.source.clone(),
        };
        if self.package_root.join(file.asset_path()).is_file() {
            out.push(file);
        }
        for submodule in &module.submodules {
            self.collect(submodule, parents, out);
        }
        parents.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project(name: &str, files: &[(&str, &str)], toml: &str) -> (PathBuf, MaterialProject) {
        let dir = std::env::temp_dir().join(format!("pbr_gen_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("wesl.toml"), toml).unwrap();
        for (path, source) in files {
            let path = dir.join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, source).unwrap();
        }
        let project = MaterialProject::open(&dir).unwrap();
        (dir, project)
    }

    // Pins that modules are named by the wesl.toml rules: nested paths, files
    // that are not identifiers skipped with a warning, nested packages left out.
    #[test]
    fn modules_are_named_as_the_language_server_names_them() {
        let (dir, project) = project(
            "names",
            &[
                ("shaders/brick.wesl", "a"),
                ("shaders/walls/stone.wesl", "b"),
                ("shaders/my-rock.wesl", "c"),
                ("shaders/helper.wgsl", "d"),
                ("shaders/nested/wesl.toml", "edition = \"2026_pre\""),
                ("shaders/nested/inner.wesl", "e"),
            ],
            "edition = \"2026_pre\"\n",
        );
        let (files, warnings) = project.files().unwrap();
        let paths: Vec<String> = files.iter().map(ProjectFile::module_path).collect();
        assert_eq!(paths, ["package::brick", "package::walls::stone"]);
        assert_eq!(files[1].asset_path(), "walls/stone.wesl");
        assert_eq!(files[1].source, "b");
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        std::fs::remove_dir_all(dir).unwrap();
    }

    // Pins that every dependency but this package is reported, since a material
    // importing one can never compile in the editor.
    #[test]
    fn dependencies_other_than_this_package_are_reported() {
        let (dir, project) = project(
            "dependencies",
            &[("shaders/a.wesl", "")],
            "edition = \"2026_pre\"\n[dependencies]\nbevy_pbr_generator = { path = \"..\" }\nother = { path = \"../other\" }\n",
        );
        assert_eq!(project.foreign_dependencies(), ["other"]);
        std::fs::remove_dir_all(dir).unwrap();
    }

    // Pins that a directory without wesl.toml is refused with an error naming
    // it, since the package root comes only from that file.
    #[test]
    fn a_directory_without_wesl_toml_is_refused() {
        let dir = std::env::temp_dir().join(format!("pbr_gen_no_toml_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let error = MaterialProject::open(&dir).unwrap_err();
        assert!(error.0.contains("not a WESL project"), "{error}");
        std::fs::remove_dir_all(dir).unwrap();
    }
}
