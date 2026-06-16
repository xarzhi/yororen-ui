use crate::ast::{AstElement, AstNode};
use crate::error::{XmlError, XmlErrorKind};

/// Try to resolve a relative `path` against the
/// `source_file` (the path of the `.rs` file that
/// invoked the enclosing `xml!` macro). Absolute paths
/// pass through; relative paths are joined to the
/// source file's parent directory.
///
/// When `source_file` is `None` (the runtime loader /
/// unit-test path), we fall back to the current
/// working directory — this preserves the behaviour
/// tests rely on.
pub(crate) fn resolve_include_path(
    path: &str,
    source_file: Option<&str>,
) -> Result<std::path::PathBuf, XmlError> {
    use std::path::Path;
    let p = Path::new(path);
    if p.is_absolute() {
        return Ok(p.to_path_buf());
    }
    match source_file {
        Some(src) => {
            let source = Path::new(src);
            // `proc_macro::Span::file()` returns a
            // forward-slash path on every platform
            // (proc-macros run on a host-agnostic layer),
            // but be defensive and strip any leading
            // junk that some toolchains prepend.
            let dir = source
                .parent()
                .filter(|d| !d.as_os_str().is_empty())
                .unwrap_or_else(|| Path::new("."));
            Ok(dir.join(path))
        }
        None => Ok(Path::new(".").join(path)),
    }
}

/// Read and parse the XML file referenced by an `<Include>` element.
///
/// Errors are reported against the included file itself and wrapped
/// in a pre-rendered diagnostic so the caller's [`LocationTracker`]
/// does not accidentally map offsets into the parent XML.
pub(crate) fn parse_include(
    element: &AstElement,
    source_file: Option<&str>,
) -> Result<(std::path::PathBuf, AstElement), XmlError> {
    let src_attr = element
        .attributes
        .iter()
        .find(|a| a.name == "src")
        .ok_or_else(|| {
            XmlError::new(
                XmlErrorKind::UnknownAttribute,
                element.span,
                "<Include> requires a `src=\"...\"` attribute",
            )
            .at(element.byte_offset)
        })?;
    if src_attr.expr.is_some() {
        return Err(XmlError::new(
            XmlErrorKind::Unsupported,
            src_attr.span,
            "<Include src> requires a string literal, not a brace expression",
        )
        .at(src_attr.byte_offset));
    }
    let path = src_attr.raw.as_str();
    let resolved = resolve_include_path(path, source_file)?;
    let contents = std::fs::read_to_string(&resolved).map_err(|e| {
        XmlError::new(
            XmlErrorKind::ParseError,
            element.span,
            format!(
                "could not read `{}` (resolved to `{}`): {e}",
                path,
                resolved.display()
            ),
        )
    })?;

    let line_starts = crate::parser::line_starts(&contents);
    let included_location = crate::parser::LocationTracker {
        line_starts: &line_starts,
        xml: &contents,
        outer_span: element.span,
    };
    match crate::parser::parse(&contents, element.span, &included_location) {
        Ok(root) => Ok((resolved, root)),
        Err(err) => {
            let diagnostic = err.render_with(Some(&included_location));
            let rendered = format!(
                "{diagnostic}\n  = note: in included file `{}`",
                resolved.display()
            );
            Err(XmlError::new(
                XmlErrorKind::ParseError,
                element.span,
                format!("in included file `{}`", resolved.display()),
            )
            .rendered(rendered))
        }
    }
}

/// Recursively expand every `<Include src="…">` in the AST so
/// templates and other compile-time constructs can be shared
/// across XML files. Detects include cycles.
///
/// By default the included file's root element is preserved as a
/// child of the `<Include>`'s parent — this lets a section file
/// declare its own container (e.g. `<Column gap="4">`). If the
/// included root is a `<Fragment>`, its children are spliced in
/// place so shared layout files and template libraries stay
/// transparent.
///
/// Every resolved include path is pushed to `included_paths` so
/// the proc-macro can register the file with Cargo via
/// `include_str!`; otherwise Cargo has no way to know that
/// editing an included XML file should recompile the crate.
pub(crate) fn expand_includes(
    element: &mut AstElement,
    source_file: Option<&str>,
    visited: &mut std::collections::HashSet<std::path::PathBuf>,
    included_paths: &mut Vec<std::path::PathBuf>,
) -> Result<(), XmlError> {
    let mut i = 0;
    while i < element.children.len() {
        let include_info = if let AstNode::Element(child) = &element.children[i] {
            if child.tag == "Include" {
                let span = child.span;
                Some((span, parse_include(child, source_file)?))
            } else {
                None
            }
        } else {
            None
        };

        if let Some((span, (path, mut included_root))) = include_info {
            if !visited.insert(path.clone()) {
                return Err(XmlError::new(
                    XmlErrorKind::Unsupported,
                    span,
                    format!(
                        "cyclic <Include> detected: `{}` is already being included",
                        path.display()
                    ),
                ));
            }
            included_paths.push(path.clone());
            // Recurse with the *original* `source_file` (the
            // macro call site), NOT the included file's path.
            // This keeps `<Include src>` resolution anchored to
            // the same base directory at every nesting level —
            // matching the `xml_file!` convention where paths
            // like `ui/shared/x.xml` resolve the same way no
            // matter how deeply the file is nested via Include.
            // (Cycle detection still uses each file's absolute
            // `path` via `visited`, so re-parenting the source
            // base does not weaken the cycle guard.)
            expand_includes(&mut included_root, source_file, visited, included_paths)?;
            visited.remove(&path);
            let nodes = if included_root.tag == "Fragment" {
                included_root.children
            } else {
                vec![AstNode::Element(included_root)]
            };
            element.children.splice(i..i + 1, nodes);
            // Re-process the newly spliced children from this index.
            continue;
        }

        if let AstNode::Element(child) = &mut element.children[i] {
            expand_includes(child, source_file, visited, included_paths)?;
        }
        i += 1;
    }
    Ok(())
}
