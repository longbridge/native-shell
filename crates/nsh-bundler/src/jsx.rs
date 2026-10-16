//! nsh's JSX compiler: lowers JSX to calls of the native renderer
//! (`nsh:rt`, `crates/nsh-js/src/dom.rs`).
//!
//! **Tags.** A lowercase tag is a built-in: the host elements of
//! [`ELEMENTS`] (`<view>`, `<text>`, …, `<web-view>`) and the control flow
//! `<if when else>` / `<each of key>`, unless a capitalized binding in scope
//! has that name in kebab case: `WeatherPanel` is also `<weather-panel>`,
//! an imported `Button` also `<button>`, and the app's own bindings win over
//! built-ins (`Text` in scope makes `<text>` that component, an imported
//! `Modal` makes `<modal>` it; an object of functions imported from one of
//! nsh's modules is no component: `Image` from `nsh/image` leaves `<image>`
//! the element). A capitalized
//! or member tag (`<Theme.Provider>`) names a binding as usual. Props keep
//! their camelCase names.
//!
//! **Host elements** become one function that makes the element, its props
//! and its host children, and returns it. Static values are set once
//! (`prop`, `append`); values that change are functions the renderer
//! follows natively (`bindProp`, `insert`). An expression changes when it
//! calls something or reads a member (outside nested functions), as in
//! Solid. Event handlers are set once when they are a function or a name,
//! else followed. Consecutive dynamic children share one `insert`, placed
//! before the next static child.
//!
//! **Components** are called (untracked) with a props object: static props
//! are values, the others getters (`get title() { return … }`), children
//! the `children` getter; spreads keep getters (`merge`).
//!
//! The code is generated as JS text with holes, parsed, and the app's own
//! expressions are moved into the holes: they keep their spans (the source
//! map points at them), generated nodes take their JSX element's.

use std::{cell::Cell, collections::HashMap};

use oxc::{
    allocator::{Allocator, TakeIn},
    ast::{
        ast::{
            Expression, ImportDeclarationSpecifier, JSXAttributeItem, JSXAttributeName,
            JSXAttributeValue, JSXChild, JSXElement, JSXElementName, JSXExpression, JSXFragment,
            JSXMemberExpression, JSXMemberExpressionObject, ModuleExportName, Program, Statement,
        },
        builder::AstBuilder,
    },
    ast_visit::{Visit, VisitMut, walk_mut},
    diagnostics::OxcDiagnostic,
    parser::Parser,
    semantic::Scoping,
    span::{GetSpan, SourceType, Span},
    syntax::scope::{ScopeFlags, ScopeId},
};

/// The built-in host elements apps write, and the core's tag for each.
pub(crate) const ELEMENTS: &[(&str, &str)] = &[
    ("view", "view"),
    ("text", "text"),
    ("image", "image"),
    ("scroll", "scroll"),
    ("input", "input"),
    ("canvas", "canvas"),
    ("web-view", "webview"),
    ("video", "video"),
    ("modal", "modal"),
    ("input-accessory", "inputaccessory"),
];

/// The built-ins that are the renderer's components, not host elements:
/// called with props like a component.
pub(crate) const RUNTIME_COMPONENTS: &[&str] = &["list"];

/// The renderer's functions compiled code calls (imported as `_$<name>`).
const HELPERS: &[&str] = &[
    "list",
    "element",
    "text",
    "append",
    "insert",
    "prop",
    "bindProp",
    "spread",
    "component",
    "cond",
    "each",
    "merge",
    "setup",
];

/// Prefix of generated names; apps do not use it.
const HOLE: &str = "__nsh_h";

/// Where a module's top-level bindings come from, for the types of the
/// components it names in kebab case.
#[derive(Default)]
struct Origins {
    /// Local name -> (import source, imported name).
    imports: HashMap<String, (String, String)>,
    /// Local names the module exports -> their export names.
    exports: HashMap<String, String>,
}

fn export_name(n: &ModuleExportName<'_>) -> String {
    match n {
        ModuleExportName::IdentifierName(n) => n.name.to_string(),
        ModuleExportName::IdentifierReference(n) => n.name.to_string(),
        ModuleExportName::StringLiteral(n) => n.value.to_string(),
    }
}

impl Origins {
    fn of(program: &Program<'_>) -> Origins {
        use oxc::ast::ast::{BindingPattern, Declaration, ExportDefaultDeclarationKind as K};
        let mut o = Origins::default();
        for statement in &program.body {
            match statement {
                Statement::ImportDeclaration(i) => {
                    for spec in i.specifiers.iter().flatten() {
                        let (local, imported) = match spec {
                            ImportDeclarationSpecifier::ImportSpecifier(s) => {
                                (s.local.name.to_string(), export_name(&s.imported))
                            }
                            ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                                (s.local.name.to_string(), "default".to_owned())
                            }
                            ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => continue,
                        };
                        o.imports
                            .insert(local, (i.source.value.to_string(), imported));
                    }
                }
                Statement::ExportDeclaration(e) => match &e.declaration {
                    Declaration::FunctionDeclaration(f) => {
                        if let Some(id) = &f.id {
                            o.exports.insert(id.name.to_string(), id.name.to_string());
                        }
                    }
                    Declaration::ClassDeclaration(c) => {
                        if let Some(id) = &c.id {
                            o.exports.insert(id.name.to_string(), id.name.to_string());
                        }
                    }
                    Declaration::VariableDeclaration(v) => {
                        for d in &v.declarations {
                            if let BindingPattern::BindingIdentifier(id) = &d.id {
                                o.exports.insert(id.name.to_string(), id.name.to_string());
                            }
                        }
                    }
                    _ => {}
                },
                Statement::ExportNamedDeclaration(e) => {
                    for spec in &e.specifiers {
                        o.exports
                            .insert(export_name(&spec.local), export_name(&spec.exported));
                    }
                }
                Statement::ExportDefaultDeclaration(e) => {
                    let id = match &e.declaration {
                        K::FunctionDeclaration(f) => f.id.as_ref(),
                        K::ClassDeclaration(c) => c.id.as_ref(),
                        _ => None,
                    };
                    if let Some(id) = id {
                        o.exports.insert(id.name.to_string(), "default".to_owned());
                    }
                }
                _ => {}
            }
        }
        o
    }
}

/// A component a module names in kebab case, for the editor's JSX types.
pub(crate) struct KebabUse {
    pub(crate) tag: String,
    /// Where its type can be imported from: (an import's specifier, or
    /// `None` for this module; the export); `None` when it is neither
    /// imported nor exported.
    // Read by the bundle's `.nsh/types`.
    #[cfg_attr(not(feature = "bundle"), allow(dead_code))]
    pub(crate) source: Option<(Option<String>, String)>,
}

/// Compiles the JSX of `program` (TypeScript already removed; `scoping`
/// its scopes). `prologue` is JS run first (the entry's setup). Returns the
/// components named in kebab case. Errors come back rendered with a code
/// frame of `source`.
pub(crate) fn transform<'a>(
    allocator: &'a Allocator,
    program: &mut Program<'a>,
    scoping: &Scoping,
    source: &str,
    prologue: Option<&str>,
    modules: &crate::compile::NativeModules,
) -> Result<Vec<KebabUse>, String> {
    let components: Vec<&str> = modules.components.iter().map(String::as_str).collect();
    let components = components.as_slice();
    let origins = Origins::of(program);
    let native_objects = origins
        .imports
        .iter()
        .filter(|(_, (module, name))| modules.object(module, name))
        .map(|(local, _)| local.clone())
        .collect();
    let mut compiler = Compiler {
        allocator,
        scoping,
        scopes: Vec::new(),
        errors: Vec::new(),
        used: [false; HELPERS.len()],
        vars: Cell::new(0),
        components,
        kebab: Vec::new(),
        native_objects,
    };
    compiler.visit_program(program);
    if let Some(err) = compiler.errors.into_iter().next() {
        return Err(err.render_with_source_code(source.to_owned()));
    }
    let mut uses: Vec<KebabUse> = Vec::new();
    for (tag, binding, top_level) in std::mem::take(&mut compiler.kebab) {
        if uses.iter().any(|u| u.tag == tag) {
            continue;
        }
        let source = if !top_level {
            None
        } else if let Some((module, imported)) = origins.imports.get(&binding) {
            Some((Some(module.clone()), imported.clone()))
        } else {
            origins
                .exports
                .get(&binding)
                .map(|exported| (None, exported.clone()))
        };
        uses.push(KebabUse { tag, source });
    }
    let mut head = String::new();
    if let Some(prologue) = prologue {
        compiler.used[helper("setup")] = true;
        head.push_str(prologue);
    }
    let imports: Vec<String> = HELPERS
        .iter()
        .zip(compiler.used)
        .filter(|(_, used)| *used)
        .map(|(name, _)| format!("{name} as _${name}"))
        .collect();
    if imports.is_empty() {
        return Ok(uses);
    }
    let text = format!(
        "import {{ {} }} from \"nsh:rt\";\n{head}",
        imports.join(", ")
    );
    let text = allocator.alloc_str(&text);
    let parsed = Parser::new(allocator, text, SourceType::mjs()).parse();
    if let Some(err) = parsed.diagnostics.into_iter().next() {
        return Err(format!("nsh compiler: {err}"));
    }
    let mut body = parsed.program.body;
    let mut splice = Splice {
        holes: Vec::new(),
        span: Span::default(),
    };
    for statement in body.iter_mut() {
        splice.visit_statement(statement);
    }
    let rest = program.body.take_in(&AstBuilder::new(allocator));
    body.extend(rest);
    program.body = body;
    Ok(uses)
}

fn helper(name: &str) -> usize {
    HELPERS.iter().position(|h| *h == name).expect("a helper")
}

/// `WeatherPanel` -> `weather-panel`, `HTTPPanel` -> `http-panel`.
/// Whether `tag` is a built-in (an element, `if`, `each`, `list`).
fn is_built_in(tag: &str) -> bool {
    matches!(tag, "if" | "each")
        || ELEMENTS.iter().any(|(name, _)| *name == tag)
        || RUNTIME_COMPONENTS.contains(&tag)
}

pub(crate) fn kebab(name: &str) -> String {
    let chars: Vec<char> = name.chars().collect();
    let mut out = String::with_capacity(name.len() + 4);
    for (i, &c) in chars.iter().enumerate() {
        if c.is_ascii_uppercase() && i > 0 {
            let prev = chars[i - 1];
            let next_lower = chars.get(i + 1).is_some_and(|n| n.is_ascii_lowercase());
            if prev.is_ascii_lowercase()
                || prev.is_ascii_digit()
                || (prev.is_ascii_uppercase() && next_lower)
            {
                out.push('-');
            }
        }
        out.push(c.to_ascii_lowercase());
    }
    out
}

/// A JS string literal.
fn quote(s: &str) -> String {
    serde_json::to_string(s).expect("a string")
}

/// An object key: bare when it is an identifier, quoted otherwise.
fn key(name: &str) -> String {
    let ident = !name.is_empty()
        && name
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_' || c == '$')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$');
    if ident { name.to_owned() } else { quote(name) }
}

/// JSX text as React and Solid render it (Babel's algorithm): lines
/// trimmed except at the text's ends, whitespace-only lines dropped, the
/// rest joined with a space; entities decoded. One line keeps its spaces
/// (`{a} {b}`).
fn clean_text(raw: &str) -> Option<String> {
    let lines: Vec<&str> = raw.split(['\n', '\r']).collect();
    let last_non_empty = lines
        .iter()
        .rposition(|l| l.contains(|c| c != ' ' && c != '\t'))
        .unwrap_or(0);
    let mut out = String::new();
    for (i, line) in lines.iter().enumerate() {
        let line = line.replace('\t', " ");
        let mut t: &str = &line;
        if i != 0 {
            t = t.trim_start();
        }
        if i != lines.len() - 1 {
            t = t.trim_end();
        }
        if t.is_empty() {
            continue;
        }
        out.push_str(t);
        if i != last_non_empty {
            out.push(' ');
        }
    }
    (!out.is_empty()).then(|| decode_entities(&out))
}

fn decode_entities(s: &str) -> String {
    if !s.contains('&') {
        return s.to_owned();
    }
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(at) = rest.find('&') {
        out.push_str(&rest[..at]);
        rest = &rest[at..];
        let Some(end) = rest[..rest.len().min(12)].find(';') else {
            out.push('&');
            rest = &rest[1..];
            continue;
        };
        let entity = &rest[1..end];
        let decoded = match entity {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            "nbsp" => Some('\u{a0}'),
            _ => entity
                .strip_prefix("#x")
                .or_else(|| entity.strip_prefix("#X"))
                .and_then(|h| u32::from_str_radix(h, 16).ok())
                .or_else(|| entity.strip_prefix('#').and_then(|d| d.parse().ok()))
                .and_then(char::from_u32),
        };
        match decoded {
            Some(c) => {
                out.push(c);
                rest = &rest[end + 1..];
            }
            None => {
                out.push('&');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// Whether an expression's value may change: it calls something or reads a
/// member, outside nested functions.
fn is_dynamic(e: &Expression<'_>) -> bool {
    #[derive(Default)]
    struct Find(bool);
    impl<'a> Visit<'a> for Find {
        fn visit_call_expression(&mut self, _: &oxc::ast::ast::CallExpression<'a>) {
            self.0 = true;
        }
        fn visit_new_expression(&mut self, _: &oxc::ast::ast::NewExpression<'a>) {
            self.0 = true;
        }
        fn visit_static_member_expression(
            &mut self,
            _: &oxc::ast::ast::StaticMemberExpression<'a>,
        ) {
            self.0 = true;
        }
        fn visit_computed_member_expression(
            &mut self,
            _: &oxc::ast::ast::ComputedMemberExpression<'a>,
        ) {
            self.0 = true;
        }
        fn visit_private_field_expression(
            &mut self,
            _: &oxc::ast::ast::PrivateFieldExpression<'a>,
        ) {
            self.0 = true;
        }
        fn visit_tagged_template_expression(
            &mut self,
            _: &oxc::ast::ast::TaggedTemplateExpression<'a>,
        ) {
            self.0 = true;
        }
        fn visit_function(&mut self, _: &oxc::ast::ast::Function<'a>, _: ScopeFlags) {}
        fn visit_arrow_function_expression(
            &mut self,
            _: &oxc::ast::ast::ArrowFunctionExpression<'a>,
        ) {
        }
    }
    let mut f = Find::default();
    f.visit_expression(e);
    f.0
}

fn is_function(e: &Expression<'_>) -> bool {
    matches!(
        e.without_parentheses(),
        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
    )
}

/// `onPress`, `onChangeText`: an event handler prop.
fn is_event(name: &str) -> bool {
    name.len() > 2 && name.starts_with("on") && name.as_bytes()[2].is_ascii_uppercase()
}

/// What a tag names.
enum Tag {
    /// A built-in host element (the core's tag).
    Host(&'static str),
    /// A component: the JS expression naming it.
    Component(String),
    If,
    Each,
}

/// The holes of one generated snippet.
struct Gen<'a> {
    holes: Vec<Expression<'a>>,
}

impl<'a> Gen<'a> {
    fn hole(&mut self, e: Expression<'a>) -> String {
        self.holes.push(e);
        format!("{HOLE}{}", self.holes.len() - 1)
    }
}

/// How a child's value is written.
#[derive(Clone, Copy, PartialEq)]
enum Mode {
    /// Read once (in a getter, which is read again): expressions as they
    /// are.
    Getter,
    /// Kept: expressions that change become functions.
    Value,
}

struct Compiler<'a, 's> {
    allocator: &'a Allocator,
    scoping: &'s Scoping,
    scopes: Vec<ScopeId>,
    errors: Vec<OxcDiagnostic>,
    used: [bool; HELPERS.len()],
    vars: Cell<usize>,
    /// Components of the UI sets apps import (`Button`), for hints.
    components: &'s [&'s str],
    /// Kebab tags that named bindings: (tag, binding, in the module scope).
    kebab: Vec<(String, String, bool)>,
    /// Objects of functions the module imports from nsh's modules: they do
    /// not replace built-ins.
    native_objects: std::collections::HashSet<String>,
}

impl<'a> VisitMut<'a> for Compiler<'a, '_> {
    fn enter_scope(&mut self, _flags: ScopeFlags, scope_id: &Cell<Option<ScopeId>>) {
        match scope_id.get().or_else(|| self.scopes.last().copied()) {
            Some(id) => self.scopes.push(id),
            None => self.scopes.push(self.scoping.root_scope_id()),
        }
    }

    fn leave_scope(&mut self) {
        self.scopes.pop();
    }

    fn visit_expression(&mut self, e: &mut Expression<'a>) {
        if matches!(e, Expression::JSXElement(_) | Expression::JSXFragment(_)) {
            let span = e.span();
            let jsx = e.take_in(&AstBuilder::new(self.allocator));
            let mut g = Gen { holes: Vec::new() };
            let text = match jsx {
                Expression::JSXElement(el) => self.element(el.unbox(), &mut g),
                Expression::JSXFragment(f) => self.fragment(f.unbox(), &mut g),
                _ => unreachable!(),
            };
            *e = self.parse(&text, g.holes, span);
        } else {
            walk_mut::walk_expression(self, e);
        }
    }
}

impl<'a> Compiler<'a, '_> {
    fn error(&mut self, message: impl Into<String>, span: Span) {
        self.errors
            .push(OxcDiagnostic::error(message.into()).with_label(span));
    }

    fn use_helper(&mut self, name: &str) -> String {
        self.used[helper(name)] = true;
        format!("_${name}")
    }

    fn var(&self) -> String {
        let n = self.vars.get();
        self.vars.set(n + 1);
        format!("{HOLE}v{n}")
    }

    /// The capitalized binding in scope whose kebab-case name is `tag`, and
    /// whether it is the module's.
    fn kebab_binding(&self, tag: &str) -> Option<(String, bool)> {
        let mut scope = self.scopes.last().copied();
        while let Some(id) = scope {
            for name in self.scoping.get_bindings(id).keys() {
                let name = name.as_str();
                if name.starts_with(|c: char| c.is_ascii_uppercase()) && kebab(name) == tag {
                    let top = id == self.scoping.root_scope_id();
                    if top && self.native_objects.contains(name) && is_built_in(tag) {
                        return None;
                    }
                    return Some((name.to_owned(), top));
                }
            }
            scope = self.scoping.scope_parent_id(id);
        }
        None
    }

    fn tag(&mut self, name: &JSXElementName<'a>) -> Option<Tag> {
        match name {
            JSXElementName::Identifier(id) => {
                let tag = id.name.as_str();
                if let Some((binding, top)) = self.kebab_binding(tag) {
                    self.kebab.push((tag.to_owned(), binding.clone(), top));
                    return Some(Tag::Component(binding));
                }
                match tag {
                    "if" => return Some(Tag::If),
                    "each" => return Some(Tag::Each),
                    _ => {}
                }
                if let Some(&(_, core)) = ELEMENTS.iter().find(|(name, _)| *name == tag) {
                    return Some(Tag::Host(core));
                }
                if let Some(&name) = RUNTIME_COMPONENTS.iter().find(|n| **n == tag) {
                    return Some(Tag::Component(self.use_helper(name)));
                }
                let hint = self
                    .components
                    .iter()
                    .find(|c| kebab(c) == tag)
                    .map(|c| format!(": import {{ {c} }} from \"nsh/ui/mobile\""))
                    .unwrap_or_else(|| {
                        let names: Vec<&str> = ELEMENTS.iter().map(|(n, _)| *n).collect();
                        format!(
                            " (built-in elements: {}; `if`, `each`; or a component in scope, by its name in kebab case)",
                            names.join(", ")
                        )
                    });
                self.error(format!("unknown element <{tag}>{hint}"), id.span);
                None
            }
            JSXElementName::IdentifierReference(r) => Some(Tag::Component(r.name.to_string())),
            JSXElementName::MemberExpression(m) => Some(Tag::Component(member(m))),
            JSXElementName::NamespacedName(n) => {
                self.error("namespaced tags are not supported", n.span);
                None
            }
            JSXElementName::ThisExpression(t) => {
                self.error("`this` is not a component", t.span);
                None
            }
        }
    }

    /// Parses generated code and moves the holes in.
    fn parse(&mut self, text: &str, holes: Vec<Expression<'a>>, span: Span) -> Expression<'a> {
        let text = self.allocator.alloc_str(text);
        match Parser::new(self.allocator, text, SourceType::mjs()).parse_expression() {
            Ok(mut e) => {
                let mut splice = Splice {
                    holes: holes.into_iter().map(Some).collect(),
                    span,
                };
                splice.visit_expression(&mut e);
                e
            }
            Err(errors) => {
                let detail = errors
                    .into_iter()
                    .next()
                    .map(|e| e.to_string())
                    .unwrap_or_default();
                self.error(
                    format!("nsh compiler: generated invalid code ({detail}): {text}"),
                    span,
                );
                Expression::new_null_literal(span, &AstBuilder::new(self.allocator))
            }
        }
    }

    /// An element as an expression.
    fn element(&mut self, el: JSXElement<'a>, g: &mut Gen<'a>) -> String {
        match self.tag(&el.opening_element.name) {
            Some(Tag::Host(core)) => {
                let mut body = String::new();
                let v = self.host(el, core, g, &mut body);
                format!("(() => {{ {body}return {v}; }})()")
            }
            Some(Tag::Component(callee)) => self.component(el, &callee, g),
            Some(Tag::If) => self.if_(el, g),
            Some(Tag::Each) => self.each(el, g),
            None => "undefined".to_owned(),
        }
    }

    /// A fragment: the array of its children.
    fn fragment(&mut self, f: JSXFragment<'a>, g: &mut Gen<'a>) -> String {
        let values = self.children(f.children, g, Mode::Value);
        format!("[{}]", values.join(", "))
    }

    /// An attribute's or child's expression, its own JSX compiled first.
    fn expression(&mut self, e: JSXExpression<'a>) -> Option<Expression<'a>> {
        if matches!(e, JSXExpression::EmptyExpression(_)) {
            return None;
        }
        let mut e = e.into_expression();
        self.visit_expression(&mut e);
        Some(e)
    }

    /// A host element: its statements go to `body`; returns its variable.
    fn host(
        &mut self,
        el: JSXElement<'a>,
        core: &str,
        g: &mut Gen<'a>,
        body: &mut String,
    ) -> String {
        let v = self.var();
        let element = self.use_helper("element");
        body.push_str(&format!("const {v} = {element}({});", quote(core)));
        let has_children = el.children.iter().any(|c| match c {
            JSXChild::Text(t) => clean_text(&t.value).is_some(),
            JSXChild::ExpressionContainer(c) => {
                !matches!(c.expression, JSXExpression::EmptyExpression(_))
            }
            _ => true,
        });
        let opening = el.opening_element.unbox();
        for attr in opening.attributes {
            match attr {
                JSXAttributeItem::Attribute(a) => {
                    let a = a.unbox();
                    let Some(name) = self.attr_name(&a.name) else {
                        continue;
                    };
                    if name == "children" && has_children {
                        continue;
                    }
                    match a.value {
                        None => body.push_str(&self.prop(&v, &name, "true")),
                        Some(JSXAttributeValue::StringLiteral(s)) => {
                            body.push_str(&self.prop(&v, &name, &quote(&s.value)));
                        }
                        Some(JSXAttributeValue::ExpressionContainer(c)) => {
                            if let Some(e) = self.expression(c.unbox().expression) {
                                self.host_attr(&v, &name, e, g, body);
                            }
                        }
                        Some(JSXAttributeValue::Element(child)) => {
                            let value = self.element(child.unbox(), g);
                            body.push_str(&self.prop(&v, &name, &value));
                        }
                        Some(JSXAttributeValue::Fragment(f)) => {
                            let value = self.fragment(f.unbox(), g);
                            body.push_str(&self.prop(&v, &name, &value));
                        }
                    }
                }
                JSXAttributeItem::SpreadAttribute(s) => {
                    let mut e = s.unbox().argument;
                    self.visit_expression(&mut e);
                    let spread = self.use_helper("spread");
                    body.push_str(&format!("{spread}({v}, {});", g.hole(e)));
                }
            }
        }
        self.host_children(&v, el.children, g, body);
        v
    }

    fn attr_name(&mut self, name: &JSXAttributeName<'a>) -> Option<String> {
        match name {
            JSXAttributeName::Identifier(id) => Some(id.name.to_string()),
            JSXAttributeName::NamespacedName(n) => {
                self.error("namespaced props are not supported", n.span);
                None
            }
        }
    }

    fn prop(&mut self, v: &str, name: &str, value: &str) -> String {
        let prop = self.use_helper("prop");
        format!("{prop}({v}, {}, {value});", quote(name))
    }

    fn host_attr(
        &mut self,
        v: &str,
        name: &str,
        e: Expression<'a>,
        g: &mut Gen<'a>,
        body: &mut String,
    ) {
        if name == "ref" {
            // `ref={el}` assigns the variable, `ref={fn}` calls it.
            if let Expression::Identifier(id) = e.without_parentheses() {
                let id = id.name.to_string();
                body.push_str(&format!(
                    "typeof {id} === \"function\" ? {id}({v}) : ({id} = {v});"
                ));
                return;
            }
            let hole = g.hole(e);
            body.push_str(&self.prop(v, name, &hole));
            return;
        }
        let dynamic = if is_event(name) {
            !is_function(&e) && !matches!(e.without_parentheses(), Expression::Identifier(_))
        } else {
            is_dynamic(&e)
        };
        let hole = g.hole(e);
        if dynamic {
            let bind = self.use_helper("bindProp");
            body.push_str(&format!("{bind}({v}, {}, () => {hole});", quote(name)));
        } else {
            body.push_str(&self.prop(v, name, &hole));
        }
    }

    /// A host element's children: static ones made and appended in order,
    /// runs of dynamic ones inserted before the next static one.
    fn host_children(
        &mut self,
        v: &str,
        children: oxc::allocator::Vec<'a, JSXChild<'a>>,
        g: &mut Gen<'a>,
        body: &mut String,
    ) {
        enum Item {
            Static(String),
            Dynamic(String),
        }
        let mut items = Vec::new();
        for child in children {
            match child {
                JSXChild::Text(t) => {
                    if let Some(text) = clean_text(&t.value) {
                        let tv = self.var();
                        let make = self.use_helper("text");
                        body.push_str(&format!("const {tv} = {make}({});", quote(&text)));
                        items.push(Item::Static(tv));
                    }
                }
                JSXChild::Element(el) => {
                    let el = el.unbox();
                    match self.tag(&el.opening_element.name) {
                        Some(Tag::Host(core)) => {
                            let cv = self.host(el, core, g, body);
                            items.push(Item::Static(cv));
                        }
                        Some(Tag::Component(callee)) => {
                            items.push(Item::Dynamic(self.component(el, &callee, g)));
                        }
                        Some(Tag::If) => items.push(Item::Dynamic(self.if_(el, g))),
                        Some(Tag::Each) => items.push(Item::Dynamic(self.each(el, g))),
                        None => {}
                    }
                }
                JSXChild::Fragment(f) => {
                    items.push(Item::Dynamic(self.fragment(f.unbox(), g)));
                }
                JSXChild::ExpressionContainer(c) => {
                    if let Some(e) = self.expression(c.unbox().expression) {
                        items.push(Item::Dynamic(self.value(e, g, Mode::Value)));
                    }
                }
                JSXChild::Spread(s) => {
                    self.error("spread children are not supported", s.span);
                }
            }
        }
        let append = items
            .iter()
            .any(|i| matches!(i, Item::Static(_)))
            .then(|| self.use_helper("append"));
        for item in &items {
            if let (Item::Static(cv), Some(append)) = (item, &append) {
                body.push_str(&format!("{append}({v}, {cv});"));
            }
        }
        let mut run: Vec<&str> = Vec::new();
        for (i, item) in items.iter().enumerate() {
            match item {
                Item::Dynamic(value) => {
                    run.push(value);
                    let next_static = items[i + 1..].iter().find_map(|n| match n {
                        Item::Static(cv) => Some(cv.as_str()),
                        Item::Dynamic(_) => None,
                    });
                    let run_ends = !matches!(items.get(i + 1), Some(Item::Dynamic(_)));
                    if run_ends {
                        let value = match run.as_slice() {
                            [one] => (*one).to_owned(),
                            many => format!("[{}]", many.join(", ")),
                        };
                        let insert = self.use_helper("insert");
                        match next_static {
                            Some(marker) => {
                                body.push_str(&format!("{insert}({v}, {value}, {marker});"))
                            }
                            None => body.push_str(&format!("{insert}({v}, {value});")),
                        }
                        run.clear();
                    }
                }
                Item::Static(_) => {}
            }
        }
    }

    /// An expression as a child value: a function when it changes and the
    /// value is kept (`Mode::Value`).
    fn value(&mut self, e: Expression<'a>, g: &mut Gen<'a>, mode: Mode) -> String {
        let dynamic = mode == Mode::Value && is_dynamic(&e) && !is_function(&e);
        let hole = g.hole(e);
        if dynamic {
            format!("() => {hole}")
        } else {
            hole
        }
    }

    /// Children as values (text, elements, expressions), in order.
    fn children(
        &mut self,
        children: oxc::allocator::Vec<'a, JSXChild<'a>>,
        g: &mut Gen<'a>,
        mode: Mode,
    ) -> Vec<String> {
        let mut values = Vec::new();
        for child in children {
            match child {
                JSXChild::Text(t) => {
                    if let Some(text) = clean_text(&t.value) {
                        values.push(quote(&text));
                    }
                }
                JSXChild::Element(el) => values.push(self.element(el.unbox(), g)),
                JSXChild::Fragment(f) => values.push(self.fragment(f.unbox(), g)),
                JSXChild::ExpressionContainer(c) => {
                    if let Some(e) = self.expression(c.unbox().expression) {
                        values.push(self.value(e, g, mode));
                    }
                }
                JSXChild::Spread(s) => self.error("spread children are not supported", s.span),
            }
        }
        values
    }

    /// A component call with its props object.
    fn component(&mut self, el: JSXElement<'a>, callee: &str, g: &mut Gen<'a>) -> String {
        let mut parts: Vec<String> = Vec::new();
        let mut props: Vec<String> = Vec::new();
        let opening = el.opening_element.unbox();
        for attr in opening.attributes {
            match attr {
                JSXAttributeItem::Attribute(a) => {
                    let a = a.unbox();
                    let Some(name) = self.attr_name(&a.name) else {
                        continue;
                    };
                    let k = key(&name);
                    match a.value {
                        None => props.push(format!("{k}: true")),
                        Some(JSXAttributeValue::StringLiteral(s)) => {
                            props.push(format!("{k}: {}", quote(&s.value)));
                        }
                        Some(JSXAttributeValue::ExpressionContainer(c)) => {
                            if let Some(e) = self.expression(c.unbox().expression) {
                                // `ref={el}` assigns the variable, `ref={fn}`
                                // calls it, with what the component gives.
                                if name == "ref"
                                    && let Expression::Identifier(id) = e.without_parentheses()
                                {
                                    let id = id.name.to_string();
                                    props.push(format!(
                                        "ref({HOLE}r) {{ typeof {id} === \"function\" ? {id}({HOLE}r) : ({id} = {HOLE}r); }}"
                                    ));
                                    continue;
                                }
                                let dynamic = is_dynamic(&e);
                                let hole = g.hole(e);
                                props.push(if dynamic {
                                    format!("get {k}() {{ return {hole}; }}")
                                } else {
                                    format!("{k}: {hole}")
                                });
                            }
                        }
                        Some(JSXAttributeValue::Element(child)) => {
                            let value = self.element(child.unbox(), g);
                            props.push(format!("get {k}() {{ return {value}; }}"));
                        }
                        Some(JSXAttributeValue::Fragment(f)) => {
                            let value = self.fragment(f.unbox(), g);
                            props.push(format!("get {k}() {{ return {value}; }}"));
                        }
                    }
                }
                JSXAttributeItem::SpreadAttribute(s) => {
                    let mut e = s.unbox().argument;
                    self.visit_expression(&mut e);
                    if !props.is_empty() {
                        parts.push(format!("{{ {} }}", props.join(", ")));
                        props.clear();
                    }
                    parts.push(g.hole(e));
                }
            }
        }
        let values = self.children(el.children, g, Mode::Getter);
        if !values.is_empty() {
            let value = match values.as_slice() {
                [one] => one.clone(),
                many => format!("[{}]", many.join(", ")),
            };
            props.push(format!("get children() {{ return {value}; }}"));
        }
        if !props.is_empty() || parts.is_empty() {
            parts.push(format!("{{ {} }}", props.join(", ")));
        }
        let props = if parts.len() == 1 {
            parts.pop().expect("one")
        } else {
            let merge = self.use_helper("merge");
            format!("{merge}({})", parts.join(", "))
        };
        let component = self.use_helper("component");
        format!("{component}({callee}, {props})")
    }

    /// The attributes of a control flow element, by name (each once).
    fn flow_attrs(
        &mut self,
        el_name: &str,
        attrs: oxc::allocator::Vec<'a, JSXAttributeItem<'a>>,
        allowed: &[&str],
        g: &mut Gen<'a>,
    ) -> Vec<(String, String)> {
        let mut out = Vec::new();
        for attr in attrs {
            match attr {
                JSXAttributeItem::Attribute(a) => {
                    let a = a.unbox();
                    let Some(name) = self.attr_name(&a.name) else {
                        continue;
                    };
                    if !allowed.contains(&name.as_str()) {
                        self.error(
                            format!(
                                "<{el_name}> takes {}, not `{name}`",
                                allowed
                                    .iter()
                                    .map(|a| format!("`{a}`"))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                            a.span,
                        );
                        continue;
                    }
                    let value = match a.value {
                        None => "true".to_owned(),
                        Some(JSXAttributeValue::StringLiteral(s)) => quote(&s.value),
                        Some(JSXAttributeValue::ExpressionContainer(c)) => {
                            match self.expression(c.unbox().expression) {
                                Some(e) => g.hole(e),
                                None => continue,
                            }
                        }
                        Some(JSXAttributeValue::Element(child)) => self.element(child.unbox(), g),
                        Some(JSXAttributeValue::Fragment(f)) => self.fragment(f.unbox(), g),
                    };
                    out.push((name, value));
                }
                JSXAttributeItem::SpreadAttribute(s) => {
                    self.error(format!("<{el_name}> takes no spread props"), s.span);
                }
            }
        }
        out
    }

    /// `<if when={c} else={…}>…</if>`.
    fn if_(&mut self, el: JSXElement<'a>, g: &mut Gen<'a>) -> String {
        let span = el.span;
        let opening = el.opening_element.unbox();
        let attrs = self.flow_attrs("if", opening.attributes, &["when", "else"], g);
        let get = |name: &str| {
            attrs
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, v)| v.clone())
        };
        let Some(when) = get("when") else {
            self.error("<if> needs `when`", span);
            return "undefined".to_owned();
        };
        let values = self.children(el.children, g, Mode::Value);
        let then = match values.as_slice() {
            [] => "undefined".to_owned(),
            [one] => one.clone(),
            many => format!("[{}]", many.join(", ")),
        };
        let cond = self.use_helper("cond");
        match get("else") {
            Some(otherwise) => {
                format!("{cond}(() => {when}, () => {then}, () => {otherwise})")
            }
            None => format!("{cond}(() => {when}, () => {then})"),
        }
    }

    /// `<each of={items} key={(item) => item.id}>{(item, index) => …}</each>`.
    fn each(&mut self, el: JSXElement<'a>, g: &mut Gen<'a>) -> String {
        let span = el.span;
        let opening = el.opening_element.unbox();
        let attrs = self.flow_attrs("each", opening.attributes, &["of", "key"], g);
        let get = |name: &str| {
            attrs
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, v)| v.clone())
        };
        let Some(of) = get("of") else {
            self.error("<each> needs `of`", span);
            return "undefined".to_owned();
        };
        let mut render = None;
        for child in el.children {
            match child {
                JSXChild::Text(t) if clean_text(&t.value).is_none() => {}
                JSXChild::ExpressionContainer(c) if render.is_none() => {
                    if let Some(e) = self.expression(c.unbox().expression) {
                        render = Some(g.hole(e));
                    }
                }
                other => {
                    self.error(
                        "<each> takes one child: a function rendering an item, `{(item, index) => …}`",
                        other.span(),
                    );
                    return "undefined".to_owned();
                }
            }
        }
        let Some(render) = render else {
            self.error(
                "<each> takes one child: a function rendering an item, `{(item, index) => …}`",
                span,
            );
            return "undefined".to_owned();
        };
        let each = self.use_helper("each");
        match get("key") {
            Some(key) => format!("{each}(() => {of}, {render}, {key})"),
            None => format!("{each}(() => {of}, {render})"),
        }
    }
}

fn member(m: &JSXMemberExpression<'_>) -> String {
    let object = match &m.object {
        JSXMemberExpressionObject::IdentifierReference(r) => r.name.to_string(),
        JSXMemberExpressionObject::MemberExpression(inner) => member(inner),
        JSXMemberExpressionObject::ThisExpression(_) => "this".to_owned(),
    };
    format!("{object}.{}", m.property.name)
}

/// Moves the holes into generated code and gives generated nodes `span`.
struct Splice<'a> {
    holes: Vec<Option<Expression<'a>>>,
    span: Span,
}

impl<'a> VisitMut<'a> for Splice<'a> {
    fn visit_expression(&mut self, e: &mut Expression<'a>) {
        if let Expression::Identifier(id) = e
            && let Some(i) = id
                .name
                .as_str()
                .strip_prefix(HOLE)
                .and_then(|n| n.parse::<usize>().ok())
            && let Some(hole) = self.holes.get_mut(i).and_then(Option::take)
        {
            *e = hole;
            return;
        }
        walk_mut::walk_expression(self, e);
    }

    fn visit_span(&mut self, span: &mut Span) {
        *span = self.span;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile(source: &str) -> Result<String, String> {
        crate::compile::compile_native(
            std::path::Path::new("App.tsx"),
            SourceType::tsx(),
            source,
            None,
            None,
            &crate::compile::NativeModules::new(&[nsh_ui_mobile::registry()]),
        )
        .map(|(code, ..)| code)
    }

    /// The code with whitespace collapsed, for comparisons.
    fn flat(code: &str) -> String {
        code.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    #[test]
    fn host_elements_static_and_dynamic_props() {
        let code = flat(&compile(
            r#"const n = 1; let f = (x: number) => x;
            export const a = <view testID="a" accessible pointerEvents={n} accessibilityLabel={f(n)} onPress={f} onLongPress={n > 0 ? f : undefined} />;"#,
        )
        .unwrap());
        assert!(
            code.contains(r#"prop(__nsh_hv0, "testID", "a");"#),
            "{code}"
        );
        assert!(
            code.contains(r#"prop(__nsh_hv0, "accessible", true);"#),
            "{code}"
        );
        assert!(
            code.contains(r#"prop(__nsh_hv0, "pointerEvents", n);"#),
            "{code}"
        );
        assert!(
            code.contains(r#"bindProp(__nsh_hv0, "accessibilityLabel", () => f(n));"#),
            "{code}"
        );
        assert!(code.contains(r#"prop(__nsh_hv0, "onPress", f);"#), "{code}");
        assert!(
            code.contains(r#"bindProp(__nsh_hv0, "onLongPress", () => n > 0 ? f : undefined);"#),
            "{code}"
        );
        assert!(code.starts_with(r#"import { element as _$element, prop as _$prop, bindProp as _$bindProp } from "nsh:rt";"#), "{code}");
    }

    #[test]
    fn kebab_tags_name_components_in_scope() {
        let code = flat(
            &compile(
                r#"function WeatherPanel(p: { city: string }) { return <view />; }
            export const a = <weather-panel city="Oslo" />;
            export const b = <WeatherPanel city="Bergen" />;"#,
            )
            .unwrap(),
        );
        assert!(
            code.contains(r#"_$component(WeatherPanel, { city: "Oslo" })"#),
            "{code}"
        );
        assert!(
            code.contains(r#"_$component(WeatherPanel, { city: "Bergen" })"#),
            "{code}"
        );
    }

    #[test]
    fn bindings_in_scope_override_built_ins() {
        let code = flat(
            &compile(
                r#"export function Outer() {
                const Text = (p: { children?: unknown }) => p.children;
                return <view><text>mine</text></view>;
            }
            export const builtIn = <text>core</text>;"#,
            )
            .unwrap(),
        );
        assert!(
            code.contains(r#"_$component(Text, { get children() { return "mine"; } })"#),
            "{code}"
        );
        assert!(code.contains(r#"_$element("text")"#), "{code}");
    }

    #[test]
    fn nsh_modules_do_not_replace_built_ins() {
        let code = flat(
            &compile(
                r#"import { Image } from "nsh/image";
            import { Button } from "nsh/ui/mobile";
            export const a = <view><image src="x" /><button title="b" /></view>;
            Image.prefetch("x");"#,
            )
            .unwrap(),
        );
        assert!(code.contains(r#"_$element("image")"#), "{code}");
        assert!(code.contains("_$component(Button"), "{code}");
        // A component of nsh's replaces the built-in of its name.
        let code = flat(
            &compile(
                r#"import { Modal } from "nsh/ui/mobile";
            export const a = <modal visible={false}><text>m</text></modal>;"#,
            )
            .unwrap(),
        );
        assert!(code.contains("_$component(Modal"), "{code}");
    }

    #[test]
    fn unknown_tags_say_what_to_import() {
        let err = compile(r#"export const a = <button title="x" />;"#).unwrap_err();
        assert!(err.contains("unknown element <button>"), "{err}");
        assert!(
            err.contains(r#"import { Button } from "nsh/ui/mobile""#),
            "{err}"
        );
        let err = compile("export const a = <frobnicate />;").unwrap_err();
        assert!(err.contains("built-in elements: view, text"), "{err}");
    }

    #[test]
    fn component_props_children_and_spreads() {
        let code = flat(&compile(
            r#"declare const Card: any, s: () => number, rest: object;
            export const a = <Card title={s()} n={1} {...rest} footer={<text>f</text>}>one {s()}</Card>;"#,
        )
        .unwrap());
        assert!(
            code.contains("_$merge({ get title() { return s(); }, n: 1 }, rest, {"),
            "{code}"
        );
        assert!(code.contains("get footer() { return (() => {"), "{code}");
        assert!(
            code.contains(r#"get children() { return ["one ", s()]; }"#),
            "{code}"
        );
    }

    #[test]
    fn refs_assign_variables_or_call_functions() {
        let code = flat(&compile(
            r#"let el: unknown; export const a = <view ref={el} />; export const b = <view ref={(n) => (el = n)} />;"#,
        )
        .unwrap());
        assert!(
            code.contains(r#"typeof el === "function" ? el(__nsh_hv0) : el = __nsh_hv0;"#),
            "{code}"
        );
        assert!(
            code.contains(r#"_$prop(__nsh_hv1, "ref", (n) => el = n);"#),
            "{code}"
        );
    }

    #[test]
    fn control_flow() {
        let code = flat(
            &compile(
                r#"declare const on: () => boolean, items: () => { id: string }[];
            export const a = <view>
                <if when={on()} else={<text>off</text>}>on</if>
                <each of={items()} key={(i) => i.id}>{(item) => <text>{item().id}</text>}</each>
            </view>;"#,
            )
            .unwrap(),
        );
        assert!(
            code.contains("_$cond(() => on(), () => \"on\", () => (() => {"),
            "{code}"
        );
        assert!(
            code.contains("_$each(() => items(), (item) => (() => {"),
            "{code}"
        );
        assert!(code.contains("(i) => i.id)"), "{code}");
        let err = compile("export const a = <each of={[]}><text /></each>;").unwrap_err();
        assert!(err.contains("<each> takes one child"), "{err}");
        let err = compile("export const a = <if>x</if>;").unwrap_err();
        assert!(err.contains("<if> needs `when`"), "{err}");
    }

    #[test]
    fn imports_only_kebab_tags_use_stay_and_type_only_ones_go() {
        let code = flat(
            &compile(
                r#"import { Stepper, StepperRef, Card } from "nsh/ui/mobile";
            let r: StepperRef | undefined;
            export const a = <stepper ref={r} />;"#,
            )
            .unwrap(),
        );
        assert!(
            code.contains(r#"import { Stepper } from "nsh/ui/mobile";"#),
            "{code}"
        );
        assert!(
            code.contains(
                "ref(__nsh_hr) { typeof r === \"function\" ? r(__nsh_hr) : r = __nsh_hr; }"
            ),
            "{code}"
        );
    }

    #[test]
    fn kebab_names() {
        assert_eq!(kebab("WeatherPanel"), "weather-panel");
        assert_eq!(kebab("HTTPPanel"), "http-panel");
        assert_eq!(kebab("WebView"), "web-view");
        assert_eq!(kebab("Button"), "button");
        assert_eq!(kebab("Panel2D"), "panel2-d");
    }

    #[test]
    fn jsx_text_like_react() {
        assert_eq!(clean_text("  hello  ").as_deref(), Some("  hello  "));
        assert_eq!(
            clean_text("\n   hello\n   world  \n  ").as_deref(),
            Some("hello world")
        );
        assert_eq!(clean_text("\n   \n  "), None);
        assert_eq!(clean_text(" ").as_deref(), Some(" "));
        assert_eq!(clean_text(" \n ").as_deref(), None);
        assert_eq!(
            clean_text("a &amp; b &#65;&#x42;").as_deref(),
            Some("a & b AB")
        );
    }
}
