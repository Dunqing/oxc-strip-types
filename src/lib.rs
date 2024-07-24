//! # Oxc Strip Types
//!
//!
mod options;
use std::borrow::Cow;
use std::cell::RefCell;
use std::sync::Arc;

use oxc::allocator::Allocator;
use oxc::ast::ast::{
    Declaration, ExportNamedDeclaration, ExportSpecifier, ImportDeclaration, ImportSpecifier,
    Program, TSType, TSTypeAnnotation, TSTypeParameterDeclaration, TSTypeParameterInstantiation,
};
use oxc::ast::visit::walk::{
    walk_declaration, walk_export_named_declaration, walk_import_declaration,
};
use oxc::ast::Visit;
use oxc::diagnostics::{Error, OxcDiagnostic};
use oxc::parser::Parser;
use oxc::span::{GetSpan, SourceType, Span};

pub use options::StripTypesOptions;

pub struct StripTypes {
    source_text: Arc<String>,
    filename: String,
    allocator: Allocator,
    errors: RefCell<Vec<Error>>,
    options: StripTypesOptions,
}

pub struct StripTypesReturn {
    pub code: String,
    pub errors: Vec<Error>,
}

impl StripTypes {
    pub fn new(source_text: String, filename: String, options: StripTypesOptions) -> StripTypes {
        Self {
            source_text: Arc::new(source_text),
            filename,
            allocator: Allocator::default(),
            errors: RefCell::new(Vec::default()),
            options,
        }
    }

    pub fn parse(&self) -> Result<Program, ()> {
        let source_type = SourceType::from_path(&self.filename);
        if let Err(err) = source_type {
            self.errors
                .borrow_mut()
                .push(OxcDiagnostic::error(format!("Unsupported {:?}", err)).into());
            return Err(());
        }
        let ret = Parser::new(&self.allocator, &self.source_text, source_type.unwrap()).parse();
        if !ret.errors.is_empty() {}

        return Ok(ret.program);
    }

    pub fn strip(self) -> StripTypesReturn {
        let ret = self.parse();
        if ret.is_err() {
            return StripTypesReturn {
                code: self.source_text.to_string(),
                errors: self.errors.take(),
            };
        }

        let code = StripTypesImpl::new(self.source_text.to_string(), self.options.clone())
            .build(&ret.unwrap());
        StripTypesReturn { code, errors: Vec::default() }
    }
}

struct StripTypesImpl {
    code: String,
    options: StripTypesOptions,
    spans: Vec<Span>,
}

impl StripTypesImpl {
    pub fn new(code: String, options: StripTypesOptions) -> StripTypesImpl {
        Self { code, options, spans: Vec::default() }
    }

    pub fn build(mut self, program: &Program) -> String {
        self.visit_program(&program);
        self.replace_all();
        self.code
    }

    pub fn push(&mut self, node: &impl GetSpan) {
        let span = node.span();
        self.spans.push(span)
    }

    pub fn replace_all(&mut self) {
        for span in self.spans.iter().rev() {
            let range = (span.start as usize)..(span.end as usize);
            let with = if self.options.replace_with_space {
                Cow::Owned(" ".repeat((span.end - span.start) as usize))
            } else {
                Cow::Borrowed("")
            };
            self.code.replace_range(range, &with);
        }
    }
}

impl<'a> Visit<'a> for StripTypesImpl {
    fn visit_ts_type(&mut self, it: &TSType) {
        self.push(it);
    }

    fn visit_declaration(&mut self, it: &Declaration<'a>) {
        if it.is_typescript_syntax() {
            self.push(it);
        } else {
            walk_declaration(self, it);
        }
    }

    fn visit_import_declaration(&mut self, it: &ImportDeclaration<'a>) {
        if it.import_kind.is_type() {
            self.push(it);
        } else {
            walk_import_declaration(self, it);
        }
    }

    fn visit_import_specifier(&mut self, it: &ImportSpecifier<'a>) {
        if it.import_kind.is_type() {
            self.push(it);
        }
    }

    fn visit_export_named_declaration(&mut self, it: &ExportNamedDeclaration<'a>) {
        if it.is_typescript_syntax() {
            self.push(it);
        } else {
            walk_export_named_declaration(self, it)
        }
    }

    fn visit_export_specifier(&mut self, it: &ExportSpecifier<'a>) {
        if it.export_kind.is_type() {
            self.push(it);
        }
    }

    fn visit_ts_type_annotation(&mut self, it: &TSTypeAnnotation<'a>) {
        self.push(it);
    }

    fn visit_ts_type_parameter_declaration(&mut self, it: &TSTypeParameterDeclaration<'a>) {
        self.push(it);
    }

    fn visit_ts_type_parameter_instantiation(&mut self, it: &TSTypeParameterInstantiation<'a>) {
        self.push(it);
    }
}
