use std::collections::{HashSet, VecDeque};
use std::path::{Path, PathBuf};

use tree_sitter::{Node, Parser, Tree};
use tree_sitter_python::language;

use crate::bundler::types::{AnalysisResult, FileEntryCandidate};

#[derive(Debug, thiserror::Error)]
pub enum AnalyzeError {
    #[error("Failed to parse Python file: {0}")]
    Parse(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub fn analyze(project_dir: &Path, entry: &Path) -> Result<AnalysisResult, AnalyzeError> {
    let mut parser = Parser::new();
    parser
        .set_language(language())
        .map_err(|_| AnalyzeError::Parse("failed to set parser language".into()))?;

    let entry_vfs = pathdiff::diff_paths(entry, project_dir)
        .unwrap_or_else(|| PathBuf::from("__main__.py"))
        .to_string_lossy()
        .to_string();

    let mut queue: VecDeque<(String, PathBuf)> = VecDeque::new();
    queue.push_back((entry_vfs.clone(), entry.to_path_buf()));

    let mut seen: HashSet<String> = HashSet::new();
    let mut py_files = Vec::new();

    while let Some((vfs_path, host_path)) = queue.pop_front() {
        if !seen.insert(vfs_path.clone()) {
            continue;
        }

        // Only bundle .py files
        if host_path.extension().and_then(|s| s.to_str()) != Some("py") {
            continue;
        }

        let src = std::fs::read_to_string(&host_path)?;
        let tree = parser
            .parse(&src, None)
            .ok_or_else(|| AnalyzeError::Parse(host_path.display().to_string()))?;
        let imports = collect_imports(&tree, &src);

        py_files.push(FileEntryCandidate {
            vfs_path: vfs_path.clone(),
            host_path: host_path.clone(),
        });

        // Also ensure all __init__.py files in the parent chain are included
        let mut current_p = host_path.parent();
        while let Some(p) = current_p {
            if !p.starts_with(project_dir) {
                break;
            }
            if p == project_dir || p == project_dir.join("src") {
                break;
            }

            let init = p.join("__init__.py");
            if init.exists() {
                let init_vfs = module_to_vfs_path("", &init, project_dir);
                if seen.insert(init_vfs.clone()) {
                    py_files.push(FileEntryCandidate {
                        vfs_path: init_vfs,
                        host_path: init,
                    });
                }
            }
            current_p = p.parent();
        }

        for m in imports {
            // Try to resolve as a local project module; skip if not found (stdlib/third-party)
            if let Some(resolved) = resolve_module(&m, project_dir) {
                let vfs = module_to_vfs_path(&m, &resolved, project_dir);
                queue.push_back((vfs, resolved));
            }
        }
    }

    Ok(AnalysisResult {
        entry_vfs,
        py_files,
    })
}

fn collect_imports(tree: &Tree, src: &str) -> Vec<String> {
    let mut imports = Vec::new();
    collect_node(tree.root_node(), src, &mut imports);
    imports
}

fn collect_node(node: Node, src: &str, out: &mut Vec<String>) {
    let kind = node.kind();
    match kind {
        "import_statement" => {
            for child in node.children(&mut node.walk()) {
                if child.kind() == "dotted_name" || child.kind() == "aliased_import" {
                    let text = child.utf8_text(src.as_bytes()).unwrap_or("").trim();
                    if !text.is_empty() {
                        let base = text.split_whitespace().next().unwrap_or(text);
                        out.push(base.replace(" ", "").to_string());
                    }
                }
            }
            return; // don't recurse into children
        }
        "import_from_statement" => {
            for i in 0..node.child_count() {
                let child = node.child(i).unwrap();
                if child.kind() == "dotted_name" {
                    let text = child.utf8_text(src.as_bytes()).unwrap_or("").trim();
                    if !text.is_empty() {
                        out.push(text.to_string());
                    }
                    break; // stop after first dotted_name (the module we're from)
                }
            }
            return; // don't recurse into children
        }
        _ => {}
    }
    // Recurse for other node types
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_node(child, src, out);
    }
}

fn resolve_module(module: &str, project_dir: &Path) -> Option<PathBuf> {
    let relative_path = module.replace('.', "/");

    // Check root
    let root_py = project_dir.join(format!("{}.py", relative_path));
    if root_py.exists() {
        return Some(root_py);
    }
    let root_init = project_dir.join(&relative_path).join("__init__.py");
    if root_init.exists() {
        return Some(root_init);
    }

    // Check src/
    let src_dir = project_dir.join("src");
    let src_py = src_dir.join(format!("{}.py", relative_path));
    if src_py.exists() {
        return Some(src_py);
    }
    let src_init = src_dir.join(&relative_path).join("__init__.py");
    if src_init.exists() {
        return Some(src_init);
    }

    // Not a local module — skip (stdlib or third-party)
    None
}

fn module_to_vfs_path(_module: &str, resolved: &Path, project_dir: &Path) -> String {
    // Use the path relative to the project dir as the VFS path
    pathdiff::diff_paths(resolved, project_dir)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| {
            resolved
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        })
}
