//! Terminal/CLI documentation printer
//!
//! This module provides terminal output for documentation nodes,
//! with colored formatting similar to deno_doc's printer.
//!
//! # Example
//!
//! ```no_run
//! use forge_etch::printer::EtchPrinter;
//! use forge_etch::node::EtchNode;
//!
//! let nodes: Vec<EtchNode> = vec![];
//! let printer = EtchPrinter::new(&nodes, true, false);
//! println!("{}", printer);
//! ```

use crate::js_doc::{EtchDoc, JsDocTag};
use crate::node::{EtchNode, EtchNodeDef, EtchNodeKind, Location};
use crate::visibility::Visibility;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

/// Terminal documentation printer
///
/// Formats documentation nodes for terminal output with optional
/// colored syntax highlighting.
pub struct EtchPrinter<'a> {
    /// Nodes to print
    nodes: &'a [EtchNode],
    /// Whether to use colored output
    use_color: bool,
    /// Whether to include private symbols
    include_private: bool,
}

impl<'a> EtchPrinter<'a> {
    /// Create a new printer
    ///
    /// # Arguments
    /// * `nodes` - Documentation nodes to print
    /// * `use_color` - Whether to use ANSI colors
    /// * `include_private` - Whether to include private symbols
    pub fn new(nodes: &'a [EtchNode], use_color: bool, include_private: bool) -> Self {
        Self {
            nodes,
            use_color,
            include_private,
        }
    }

    /// Format documentation for Display trait
    pub fn format(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut sorted = self.nodes.to_vec();
        sorted.sort_by(|a, b| {
            let kind_cmp = self.kind_order(a).cmp(&self.kind_order(b));
            if kind_cmp == std::cmp::Ordering::Equal {
                a.name.cmp(&b.name)
            } else {
                kind_cmp
            }
        });

        // Filter by visibility
        let filtered: Vec<_> = sorted
            .iter()
            .filter(|n| self.include_private || n.visibility.should_document())
            .collect();

        for node in &filtered {
            // Location header
            if !node.location.is_unknown() {
                writeln!(
                    f,
                    "{}",
                    self.styled_gray(&format!(
                        "Defined in {}\n",
                        self.format_location(&node.location)
                    ))
                )?;
            }

            // Signature
            self.format_signature(f, node, 0)?;

            // JSDoc
            self.format_jsdoc(f, &node.doc, 1)?;

            // Members for compound types
            match &node.def {
                EtchNodeDef::Class { class_def } => self.format_class_members(f, class_def)?,
                EtchNodeDef::Interface { interface_def } => {
                    self.format_interface_members(f, interface_def)?
                }
                EtchNodeDef::Enum { enum_def } => self.format_enum_members(f, enum_def)?,
                EtchNodeDef::Namespace { namespace_def } => {
                    for elem in &namespace_def.elements {
                        self.format_signature(f, elem, 1)?;
                        self.format_jsdoc(f, &elem.doc, 2)?;
                    }
                }
                _ => {}
            }

            writeln!(f)?;
        }

        Ok(())
    }

    /// Print directly to stdout with colors
    pub fn print_to_stdout(&self) {
        let choice = if self.use_color {
            ColorChoice::Auto
        } else {
            ColorChoice::Never
        };
        let mut stdout = StandardStream::stdout(choice);
        if let Err(e) = self.write_colored(&mut stdout) {
            eprintln!("Error printing documentation: {}", e);
        }
    }

    /// Write with colors to a WriteColor implementor
    fn write_colored<W: WriteColor>(&self, w: &mut W) -> io::Result<()> {
        let mut sorted = self.nodes.to_vec();
        sorted.sort_by(|a, b| {
            let kind_cmp = self.kind_order(a).cmp(&self.kind_order(b));
            if kind_cmp == std::cmp::Ordering::Equal {
                a.name.cmp(&b.name)
            } else {
                kind_cmp
            }
        });

        let filtered: Vec<_> = sorted
            .iter()
            .filter(|n| self.include_private || n.visibility.should_document())
            .collect();

        for node in &filtered {
            // Location header
            if !node.location.is_unknown() {
                w.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_italic(true))?;
                writeln!(w, "Defined in {}\n", self.format_location(&node.location))?;
                w.reset()?;
            }

            // Signature
            self.write_signature_colored(w, node, 0)?;

            // JSDoc
            self.write_jsdoc_colored(w, &node.doc, 1)?;

            // Members
            match &node.def {
                EtchNodeDef::Class { class_def } => {
                    self.write_class_members_colored(w, class_def)?
                }
                EtchNodeDef::Interface { interface_def } => {
                    self.write_interface_members_colored(w, interface_def)?
                }
                EtchNodeDef::Enum { enum_def } => self.write_enum_members_colored(w, enum_def)?,
                _ => {}
            }

            writeln!(w)?;
        }

        Ok(())
    }

    /// Order for sorting by kind
    fn kind_order(&self, node: &EtchNode) -> i64 {
        match node.kind() {
            EtchNodeKind::Module => 0,
            EtchNodeKind::Op => 1,
            EtchNodeKind::Function => 2,
            EtchNodeKind::Variable => 3,
            EtchNodeKind::Class => 4,
            EtchNodeKind::Interface => 5,
            EtchNodeKind::Struct => 6,
            EtchNodeKind::Enum => 7,
            EtchNodeKind::TypeAlias => 8,
            EtchNodeKind::Namespace => 9,
            EtchNodeKind::Import => 10,
            EtchNodeKind::Reference => 11,
        }
    }

    /// Format location string
    fn format_location(&self, loc: &Location) -> String {
        format!("{}:{}:{}", loc.filename, loc.line, loc.col + 1)
    }

    /// Format signature for a node
    fn format_signature(&self, f: &mut Formatter<'_>, node: &EtchNode, indent: usize) -> FmtResult {
        let ind = Indent(indent);

        match &node.def {
            EtchNodeDef::Op { op_def } => {
                write!(f, "{}", ind)?;
                if node.visibility == Visibility::Private {
                    write!(f, "{} ", self.styled_gray("private"))?;
                }
                if op_def.is_async {
                    write!(f, "{} ", self.styled_cyan("async"))?;
                }
                write!(
                    f,
                    "{} {}",
                    self.styled_magenta("function"),
                    self.styled_bold(&node.name)
                )?;
                self.format_params(f, &op_def.params)?;
                writeln!(f, ": {}", self.styled_cyan(&op_def.return_type))
            }

            EtchNodeDef::Function { function_def } => {
                write!(f, "{}", ind)?;
                if node.visibility == Visibility::Private {
                    write!(f, "{} ", self.styled_gray("private"))?;
                }
                if function_def.is_async {
                    write!(f, "{} ", self.styled_cyan("async"))?;
                }
                write!(
                    f,
                    "{}{} {}",
                    self.styled_magenta("function"),
                    if function_def.is_generator { "*" } else { "" },
                    self.styled_bold(&node.name)
                )?;
                if !function_def.type_params.is_empty() {
                    write!(f, "<")?;
                    let params: Vec<String> = function_def
                        .type_params
                        .iter()
                        .map(|p| p.to_typescript())
                        .collect();
                    write!(f, "{}", params.join(", "))?;
                    write!(f, ">")?;
                }
                self.format_params(f, &function_def.params)?;
                if let Some(ref ret) = function_def.return_type {
                    writeln!(f, ": {}", self.styled_cyan(&ret.to_typescript()))
                } else {
                    writeln!(f)
                }
            }

            EtchNodeDef::Class { class_def } => {
                write!(f, "{}", ind)?;
                if node.visibility == Visibility::Private {
                    write!(f, "{} ", self.styled_gray("private"))?;
                }
                if class_def.is_abstract {
                    write!(f, "{} ", self.styled_magenta("abstract"))?;
                }
                write!(
                    f,
                    "{} {}",
                    self.styled_magenta("class"),
                    self.styled_bold(&node.name)
                )?;
                if !class_def.type_params.is_empty() {
                    write!(f, "<")?;
                    let params: Vec<String> = class_def
                        .type_params
                        .iter()
                        .map(|p| p.to_typescript())
                        .collect();
                    write!(f, "{}", params.join(", "))?;
                    write!(f, ">")?;
                }
                if let Some(ref ext) = class_def.extends {
                    write!(
                        f,
                        " {} {}",
                        self.styled_magenta("extends"),
                        self.styled_cyan(&ext.to_typescript())
                    )?;
                }
                if !class_def.implements.is_empty() {
                    write!(f, " {} ", self.styled_magenta("implements"))?;
                    let impls: Vec<String> = class_def
                        .implements
                        .iter()
                        .map(|i| i.to_typescript())
                        .collect();
                    write!(f, "{}", impls.join(", "))?;
                }
                writeln!(f)
            }

            EtchNodeDef::Interface { interface_def } => {
                write!(f, "{}", ind)?;
                if node.visibility == Visibility::Private {
                    write!(f, "{} ", self.styled_gray("private"))?;
                }
                write!(
                    f,
                    "{} {}",
                    self.styled_magenta("interface"),
                    self.styled_bold(&node.name)
                )?;
                if !interface_def.type_params.is_empty() {
                    write!(f, "<")?;
                    let params: Vec<String> = interface_def
                        .type_params
                        .iter()
                        .map(|p| p.to_typescript())
                        .collect();
                    write!(f, "{}", params.join(", "))?;
                    write!(f, ">")?;
                }
                if !interface_def.extends.is_empty() {
                    write!(f, " {} ", self.styled_magenta("extends"))?;
                    let exts: Vec<String> = interface_def
                        .extends
                        .iter()
                        .map(|e| e.to_typescript())
                        .collect();
                    write!(f, "{}", exts.join(", "))?;
                }
                writeln!(f)
            }

            EtchNodeDef::Enum { enum_def } => {
                write!(f, "{}", ind)?;
                if node.visibility == Visibility::Private {
                    write!(f, "{} ", self.styled_gray("private"))?;
                }
                if enum_def.is_const {
                    write!(f, "{} ", self.styled_magenta("const"))?;
                }
                writeln!(
                    f,
                    "{} {}",
                    self.styled_magenta("enum"),
                    self.styled_bold(&node.name)
                )
            }

            EtchNodeDef::TypeAlias { type_alias_def } => {
                write!(f, "{}", ind)?;
                if node.visibility == Visibility::Private {
                    write!(f, "{} ", self.styled_gray("private"))?;
                }
                write!(
                    f,
                    "{} {}",
                    self.styled_magenta("type"),
                    self.styled_bold(&node.name)
                )?;
                if !type_alias_def.type_params.is_empty() {
                    write!(f, "<")?;
                    let params: Vec<String> = type_alias_def
                        .type_params
                        .iter()
                        .map(|p| p.to_typescript())
                        .collect();
                    write!(f, "{}", params.join(", "))?;
                    write!(f, ">")?;
                }
                writeln!(
                    f,
                    " = {}",
                    self.styled_cyan(&type_alias_def.ts_type.to_typescript())
                )
            }

            EtchNodeDef::Variable { variable_def } => {
                write!(f, "{}", ind)?;
                if node.visibility == Visibility::Private {
                    write!(f, "{} ", self.styled_gray("private"))?;
                }
                let kw = if variable_def.kind.is_const() {
                    "const"
                } else {
                    "let"
                };
                write!(
                    f,
                    "{} {}",
                    self.styled_magenta(kw),
                    self.styled_bold(&node.name)
                )?;
                if let Some(ref ty) = variable_def.ts_type {
                    writeln!(f, ": {}", self.styled_cyan(&ty.to_typescript()))
                } else {
                    writeln!(f)
                }
            }

            EtchNodeDef::Struct { struct_def } => {
                write!(f, "{}", ind)?;
                writeln!(
                    f,
                    "{} {} (from Rust: {})",
                    self.styled_magenta("interface"),
                    self.styled_bold(&struct_def.ts_name),
                    self.styled_gray(&struct_def.rust_name)
                )
            }

            EtchNodeDef::Namespace { .. } => {
                write!(f, "{}", ind)?;
                writeln!(
                    f,
                    "{} {}",
                    self.styled_magenta("namespace"),
                    self.styled_bold(&node.name)
                )
            }

            EtchNodeDef::Module { module_def } => {
                write!(f, "{}", ind)?;
                writeln!(
                    f,
                    "{} {} ({})",
                    self.styled_magenta("module"),
                    self.styled_bold(&module_def.name),
                    self.styled_cyan(&module_def.specifier)
                )
            }

            EtchNodeDef::Import { import_def } => {
                write!(f, "{}", ind)?;
                writeln!(
                    f,
                    "{} {} {} {}",
                    self.styled_magenta("import"),
                    self.styled_bold(import_def.imported.as_deref().unwrap_or("*")),
                    self.styled_magenta("from"),
                    self.styled_cyan(&format!("\"{}\"", import_def.src))
                )
            }

            EtchNodeDef::Reference { reference_def } => {
                write!(f, "{}", ind)?;
                writeln!(
                    f,
                    "{} {}: {}",
                    self.styled_magenta("reference"),
                    self.styled_bold(&node.name),
                    self.styled_gray(&self.format_location(&reference_def.target))
                )
            }

            EtchNodeDef::ModuleDoc => Ok(()),
        }
    }

    /// Format parameters
    fn format_params(
        &self,
        f: &mut Formatter<'_>,
        params: &[crate::params::ParamDef],
    ) -> FmtResult {
        write!(f, "(")?;
        let param_strs: Vec<String> = params.iter().map(|p| p.to_typescript()).collect();
        write!(f, "{}", param_strs.join(", "))?;
        write!(f, ")")
    }

    /// Format JSDoc
    fn format_jsdoc(&self, f: &mut Formatter<'_>, doc: &EtchDoc, indent: usize) -> FmtResult {
        let ind = Indent(indent);

        if let Some(ref desc) = doc.description {
            for line in desc.lines() {
                writeln!(f, "{}{}", ind, self.styled_gray(line))?;
            }
        }

        if !doc.tags.is_empty() && doc.description.is_some() {
            writeln!(f)?;
        }

        for tag in &doc.tags {
            self.format_jsdoc_tag(f, tag, indent)?;
        }

        Ok(())
    }

    /// Format a single JSDoc tag
    fn format_jsdoc_tag(&self, f: &mut Formatter<'_>, tag: &JsDocTag, indent: usize) -> FmtResult {
        let ind = Indent(indent);

        match tag {
            JsDocTag::Param {
                name,
                type_ref,
                doc,
                optional,
                default,
            } => {
                write!(f, "{}{}", ind, self.styled_magenta("@param"))?;
                if let Some(ref t) = type_ref {
                    write!(f, " {{{}}}", self.styled_cyan(t))?;
                }
                if *optional {
                    write!(f, " [?]")?;
                } else if let Some(ref d) = default {
                    write!(f, " [{}]", self.styled_cyan(d))?;
                }
                writeln!(f, " {}", self.styled_bold(name))?;
                if let Some(ref d) = doc {
                    for line in d.lines() {
                        writeln!(f, "{}  {}", ind, self.styled_gray(line))?;
                    }
                }
            }

            JsDocTag::Returns { type_ref, doc } => {
                write!(f, "{}{}", ind, self.styled_magenta("@returns"))?;
                if let Some(ref t) = type_ref {
                    write!(f, " {{{}}}", self.styled_cyan(t))?;
                }
                writeln!(f)?;
                if let Some(ref d) = doc {
                    for line in d.lines() {
                        writeln!(f, "{}  {}", ind, self.styled_gray(line))?;
                    }
                }
            }

            JsDocTag::Example { doc, caption } => {
                writeln!(f, "{}{}", ind, self.styled_magenta("@example"))?;
                if let Some(ref c) = caption {
                    writeln!(f, "{}  {}", ind, c)?;
                }
                for line in doc.lines() {
                    writeln!(f, "{}  {}", ind, self.styled_gray(line))?;
                }
            }

            JsDocTag::Deprecated { doc } => {
                writeln!(f, "{}{}", ind, self.styled_magenta("@deprecated"))?;
                if let Some(ref d) = doc {
                    for line in d.lines() {
                        writeln!(f, "{}  {}", ind, self.styled_gray(line))?;
                    }
                }
            }

            JsDocTag::See { reference } => {
                writeln!(
                    f,
                    "{}{} {}",
                    ind,
                    self.styled_magenta("@see"),
                    self.styled_cyan(reference)
                )?;
            }

            JsDocTag::Since { version } => {
                writeln!(
                    f,
                    "{}{} {}",
                    ind,
                    self.styled_magenta("@since"),
                    self.styled_cyan(version)
                )?;
            }

            JsDocTag::Category { name } => {
                writeln!(f, "{}{} {}", ind, self.styled_magenta("@category"), name)?;
            }

            JsDocTag::Internal => {
                writeln!(f, "{}{}", ind, self.styled_magenta("@internal"))?;
            }

            JsDocTag::Experimental => {
                writeln!(f, "{}{}", ind, self.styled_magenta("@experimental"))?;
            }

            JsDocTag::Throws { type_ref, doc } => {
                write!(f, "{}{}", ind, self.styled_magenta("@throws"))?;
                if let Some(ref t) = type_ref {
                    write!(f, " {{{}}}", self.styled_cyan(t))?;
                }
                writeln!(f)?;
                if let Some(ref d) = doc {
                    for line in d.lines() {
                        writeln!(f, "{}  {}", ind, self.styled_gray(line))?;
                    }
                }
            }

            JsDocTag::Template {
                name, constraint, ..
            } => {
                write!(
                    f,
                    "{}{} {}",
                    ind,
                    self.styled_magenta("@template"),
                    self.styled_bold(name)
                )?;
                if let Some(ref c) = constraint {
                    write!(f, " extends {}", self.styled_cyan(c))?;
                }
                writeln!(f)?;
            }

            _ => {
                // Handle other tags generically
                writeln!(f, "{}{:?}", ind, tag)?;
            }
        }

        Ok(())
    }

    /// Format class members
    fn format_class_members(
        &self,
        f: &mut Formatter<'_>,
        class_def: &crate::class::ClassDef,
    ) -> FmtResult {
        let ind = Indent(1);

        // Constructors
        for ctor in &class_def.constructors {
            write!(f, "{}{}", ind, self.styled_magenta("constructor"))?;
            write!(f, "(")?;
            let params: Vec<String> = ctor.params.iter().map(|p| p.to_typescript()).collect();
            write!(f, "{}", params.join(", "))?;
            writeln!(f, ")")?;
        }

        // Properties
        for prop in &class_def.properties {
            write!(f, "{}", ind)?;
            if let Some(ref acc) = prop.accessibility {
                write!(f, "{} ", self.styled_gray(acc))?;
            }
            if prop.is_static {
                write!(f, "{} ", self.styled_magenta("static"))?;
            }
            if prop.readonly {
                write!(f, "{} ", self.styled_magenta("readonly"))?;
            }
            write!(f, "{}", self.styled_bold(&prop.name))?;
            if prop.is_optional {
                write!(f, "?")?;
            }
            if let Some(ref ty) = prop.ts_type {
                write!(f, ": {}", self.styled_cyan(&ty.to_typescript()))?;
            }
            writeln!(f)?;
        }

        // Methods
        for method in &class_def.methods {
            write!(f, "{}", ind)?;
            if let Some(ref acc) = method.accessibility {
                write!(f, "{} ", self.styled_gray(acc))?;
            }
            if method.is_static {
                write!(f, "{} ", self.styled_magenta("static"))?;
            }
            if method.is_abstract {
                write!(f, "{} ", self.styled_magenta("abstract"))?;
            }
            write!(f, "{}", self.styled_bold(&method.name))?;
            write!(f, "(")?;
            let params: Vec<String> = method.params.iter().map(|p| p.to_typescript()).collect();
            write!(f, "{}", params.join(", "))?;
            write!(f, ")")?;
            if let Some(ref ret) = method.return_type {
                write!(f, ": {}", self.styled_cyan(&ret.to_typescript()))?;
            }
            writeln!(f)?;
        }

        Ok(())
    }

    /// Format interface members
    fn format_interface_members(
        &self,
        f: &mut Formatter<'_>,
        interface_def: &crate::interface::InterfaceDef,
    ) -> FmtResult {
        let ind = Indent(1);

        // Properties
        for prop in &interface_def.properties {
            write!(f, "{}", ind)?;
            if prop.readonly {
                write!(f, "{} ", self.styled_magenta("readonly"))?;
            }
            write!(f, "{}", self.styled_bold(&prop.name))?;
            if prop.optional {
                write!(f, "?")?;
            }
            if let Some(ref ty) = prop.ts_type {
                write!(f, ": {}", self.styled_cyan(&ty.to_typescript()))?;
            }
            writeln!(f)?;
        }

        // Methods
        for method in &interface_def.methods {
            write!(f, "{}{}", ind, self.styled_bold(&method.name))?;
            write!(f, "(")?;
            let params: Vec<String> = method.params.iter().map(|p| p.to_typescript()).collect();
            write!(f, "{}", params.join(", "))?;
            write!(f, ")")?;
            if let Some(ref ret) = method.return_type {
                write!(f, ": {}", self.styled_cyan(&ret.to_typescript()))?;
            }
            writeln!(f)?;
        }

        Ok(())
    }

    /// Format enum members
    fn format_enum_members(
        &self,
        f: &mut Formatter<'_>,
        enum_def: &crate::r#enum::EnumDef,
    ) -> FmtResult {
        let ind = Indent(1);

        for member in &enum_def.members {
            write!(f, "{}{}", ind, self.styled_bold(&member.name))?;
            if let Some(ref val) = member.init {
                write!(f, " = {}", self.styled_cyan(&val.to_typescript()))?;
            }
            writeln!(f)?;
        }

        Ok(())
    }

    // === Colored output methods ===

    /// Write signature with colors
    fn write_signature_colored<W: WriteColor>(
        &self,
        w: &mut W,
        node: &EtchNode,
        indent: usize,
    ) -> io::Result<()> {
        let ind = "  ".repeat(indent);

        match &node.def {
            EtchNodeDef::Op { op_def } => {
                write!(w, "{}", ind)?;
                if node.visibility == Visibility::Private {
                    self.write_gray(w, "private ")?;
                }
                if op_def.is_async {
                    self.write_cyan(w, "async ")?;
                }
                self.write_magenta(w, "function ")?;
                self.write_bold(w, &node.name)?;
                write!(w, "(")?;
                let params: Vec<String> = op_def.params.iter().map(|p| p.to_typescript()).collect();
                write!(w, "{}", params.join(", "))?;
                write!(w, "): ")?;
                self.write_cyan(w, &op_def.return_type)?;
                writeln!(w)
            }

            EtchNodeDef::Function { function_def } => {
                write!(w, "{}", ind)?;
                if node.visibility == Visibility::Private {
                    self.write_gray(w, "private ")?;
                }
                if function_def.is_async {
                    self.write_cyan(w, "async ")?;
                }
                self.write_magenta(w, "function")?;
                if function_def.is_generator {
                    write!(w, "*")?;
                }
                write!(w, " ")?;
                self.write_bold(w, &node.name)?;
                if !function_def.type_params.is_empty() {
                    write!(w, "<")?;
                    let params: Vec<String> = function_def
                        .type_params
                        .iter()
                        .map(|p| p.to_typescript())
                        .collect();
                    write!(w, "{}", params.join(", "))?;
                    write!(w, ">")?;
                }
                write!(w, "(")?;
                let params: Vec<String> = function_def
                    .params
                    .iter()
                    .map(|p| p.to_typescript())
                    .collect();
                write!(w, "{}", params.join(", "))?;
                write!(w, ")")?;
                if let Some(ref ret) = function_def.return_type {
                    write!(w, ": ")?;
                    self.write_cyan(w, &ret.to_typescript())?;
                }
                writeln!(w)
            }

            _ => {
                // For other types, use simple format
                writeln!(w, "{}{} {}", ind, node.kind().display_name(), node.name)
            }
        }
    }

    /// Write JSDoc with colors
    fn write_jsdoc_colored<W: WriteColor>(
        &self,
        w: &mut W,
        doc: &EtchDoc,
        indent: usize,
    ) -> io::Result<()> {
        let ind = "  ".repeat(indent);

        if let Some(ref desc) = doc.description {
            for line in desc.lines() {
                write!(w, "{}", ind)?;
                self.write_gray(w, line)?;
                writeln!(w)?;
            }
        }

        if !doc.tags.is_empty() && doc.description.is_some() {
            writeln!(w)?;
        }

        for tag in &doc.tags {
            write!(w, "{}", ind)?;
            match tag {
                JsDocTag::Param { name, type_ref, .. } => {
                    self.write_magenta(w, "@param")?;
                    if let Some(ref t) = type_ref {
                        write!(w, " {{")?;
                        self.write_cyan(w, t)?;
                        write!(w, "}}")?;
                    }
                    write!(w, " ")?;
                    self.write_bold(w, name)?;
                    writeln!(w)?;
                }
                JsDocTag::Returns { type_ref, .. } => {
                    self.write_magenta(w, "@returns")?;
                    if let Some(ref t) = type_ref {
                        write!(w, " {{")?;
                        self.write_cyan(w, t)?;
                        write!(w, "}}")?;
                    }
                    writeln!(w)?;
                }
                JsDocTag::Deprecated { .. } => {
                    self.write_magenta(w, "@deprecated")?;
                    writeln!(w)?;
                }
                JsDocTag::Example { .. } => {
                    self.write_magenta(w, "@example")?;
                    writeln!(w)?;
                }
                _ => {
                    writeln!(w, "{:?}", tag)?;
                }
            }
        }

        Ok(())
    }

    /// Write class members with colors
    fn write_class_members_colored<W: WriteColor>(
        &self,
        w: &mut W,
        class_def: &crate::class::ClassDef,
    ) -> io::Result<()> {
        let ind = "  ";

        for prop in &class_def.properties {
            write!(w, "{}", ind)?;
            self.write_bold(w, &prop.name)?;
            if let Some(ref ty) = prop.ts_type {
                write!(w, ": ")?;
                self.write_cyan(w, &ty.to_typescript())?;
            }
            writeln!(w)?;
        }

        for method in &class_def.methods {
            write!(w, "{}", ind)?;
            self.write_bold(w, &method.name)?;
            write!(w, "(")?;
            let params: Vec<String> = method.params.iter().map(|p| p.to_typescript()).collect();
            write!(w, "{}", params.join(", "))?;
            write!(w, ")")?;
            if let Some(ref ret) = method.return_type {
                write!(w, ": ")?;
                self.write_cyan(w, &ret.to_typescript())?;
            }
            writeln!(w)?;
        }

        Ok(())
    }

    /// Write interface members with colors
    fn write_interface_members_colored<W: WriteColor>(
        &self,
        w: &mut W,
        interface_def: &crate::interface::InterfaceDef,
    ) -> io::Result<()> {
        let ind = "  ";

        for prop in &interface_def.properties {
            write!(w, "{}", ind)?;
            self.write_bold(w, &prop.name)?;
            if let Some(ref ty) = prop.ts_type {
                write!(w, ": ")?;
                self.write_cyan(w, &ty.to_typescript())?;
            }
            writeln!(w)?;
        }

        Ok(())
    }

    /// Write enum members with colors
    fn write_enum_members_colored<W: WriteColor>(
        &self,
        w: &mut W,
        enum_def: &crate::r#enum::EnumDef,
    ) -> io::Result<()> {
        let ind = "  ";

        for member in &enum_def.members {
            write!(w, "{}", ind)?;
            self.write_bold(w, &member.name)?;
            if let Some(ref val) = member.init {
                write!(w, " = ")?;
                self.write_cyan(w, &val.to_typescript())?;
            }
            writeln!(w)?;
        }

        Ok(())
    }

    // === Color helper methods ===

    fn write_cyan<W: WriteColor>(&self, w: &mut W, s: &str) -> io::Result<()> {
        w.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))?;
        write!(w, "{}", s)?;
        w.reset()
    }

    fn write_magenta<W: WriteColor>(&self, w: &mut W, s: &str) -> io::Result<()> {
        w.set_color(ColorSpec::new().set_fg(Some(Color::Magenta)))?;
        write!(w, "{}", s)?;
        w.reset()
    }

    fn write_bold<W: WriteColor>(&self, w: &mut W, s: &str) -> io::Result<()> {
        w.set_color(ColorSpec::new().set_bold(true))?;
        write!(w, "{}", s)?;
        w.reset()
    }

    fn write_gray<W: WriteColor>(&self, w: &mut W, s: &str) -> io::Result<()> {
        w.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true))?;
        write!(w, "{}", s)?;
        w.reset()
    }

    // === Style helpers for Display trait (no colors in fmt) ===

    fn styled_cyan(&self, s: &str) -> String {
        if self.use_color {
            format!("\x1b[36m{}\x1b[0m", s)
        } else {
            s.to_string()
        }
    }

    fn styled_magenta(&self, s: &str) -> String {
        if self.use_color {
            format!("\x1b[35m{}\x1b[0m", s)
        } else {
            s.to_string()
        }
    }

    fn styled_bold(&self, s: &str) -> String {
        if self.use_color {
            format!("\x1b[1m{}\x1b[0m", s)
        } else {
            s.to_string()
        }
    }

    fn styled_gray(&self, s: &str) -> String {
        if self.use_color {
            format!("\x1b[90m{}\x1b[0m", s)
        } else {
            s.to_string()
        }
    }
}

impl Display for EtchPrinter<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.format(f)
    }
}

/// Indentation helper
struct Indent(usize);

impl Display for Indent {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        for _ in 0..self.0 {
            write!(f, "  ")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::function::FunctionDef;
    use crate::types::EtchType;

    #[test]
    fn test_printer_creation() {
        let nodes = vec![];
        let printer = EtchPrinter::new(&nodes, true, false);
        assert!(printer.use_color);
        assert!(!printer.include_private);
    }

    #[test]
    fn test_printer_display() {
        let nodes = vec![EtchNode {
            name: "testFunc".to_string(),
            def: EtchNodeDef::Function {
                function_def: FunctionDef {
                    is_async: true,
                    return_type: Some(EtchType::promise(EtchType::void())),
                    ..Default::default()
                },
            },
            ..Default::default()
        }];

        let printer = EtchPrinter::new(&nodes, false, false);
        let output = format!("{}", printer);

        assert!(output.contains("testFunc"));
        assert!(output.contains("async"));
        assert!(output.contains("function"));
    }

    #[test]
    fn test_indent() {
        let ind = Indent(2);
        assert_eq!(format!("{}", ind), "    ");
    }
}
