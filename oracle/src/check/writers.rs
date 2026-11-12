//! `os`'s terminal writers, the first argument of every print. (D271, D272, D273)
//!
//! A writer is a built-in struct type, `os::stdout.buffered`, `os::stdout.unbuffered`,
//! `os::stderr.buffered` or `os::stderr.unbuffered`. A buffered one is `must_Give`, and only
//! `os::done` meets that duty.

use super::expression::RawCall;
use super::hand::HandedOff;
use super::{Checker, Failure, ParameterInfo};
use crate::ast::{Argument, Expression, ExpressionKind, HandWord, ParameterWord};
use crate::ir::{self, Node, NodeKind};
use crate::report::{Outcome, Report, ReportExtra, Span};
use crate::types::Type;

const WHY_KIND: &str = "`os::stdout` and `os::stderr` say which kind they are: `.buffered(room: 1024 bytes)` writes in chunks and is finished with `os::done`, and `.unbuffered()` writes each print straight away (D273)";
const WHY_WRITER: &str = "`std::io::fmt_print` always takes a writer first, as in `fmt_print(Lend.mut $out, \"hi{newln}\");`, since the writer says where output goes (D272, D273)";

/// Whether a type is a terminal writer, and if so whether it's stderr's and whether it's
/// buffered. (D273)
pub(super) fn writer_kind(name: &str) -> Option<(bool, bool)> {
    match name {
        "os::stdout.buffered" => Some((false, true)),
        "os::stdout.unbuffered" => Some((false, false)),
        "os::stderr.buffered" => Some((true, true)),
        "os::stderr.unbuffered" => Some((true, false)),
        _ => None,
    }
}

/// A buffered writer can lose output, so the program must finish it. (D272, D273)
pub(super) fn is_buffered_writer(value_type: &Type) -> bool {
    matches!(value_type, Type::Struct(name) if writer_kind(name).is_some_and(|(_, buffered)| buffered))
}

/// Writing to a terminal can fail, the same as printing.
fn print_failed() -> Vec<Failure> {
    vec![Failure {
        short: "print_failed".to_string(),
        full: "print_failed".to_string(),
        data: Vec::new(),
    }]
}

impl Checker<'_> {
    /// trace: D271
    pub(super) fn require_os(&self, span: Span) -> Outcome<()> {
        if self.imports.contains("os") {
            return Ok(());
        }
        Err(Report::refused(
            span,
            "this file uses `os` without importing it",
            "a program that uses `os` says `need os` in its manifest and `import os;` in its files (D271, D82)",
        )
        .fix_elsewhere("import os;"))
    }

    /// `os::stdout.buffered` and the other writers, as types. Nothing else in `os` is designed.
    /// trace: D271, D273
    pub(super) fn resolve_os_type(rest: &str, span: Span, plain: bool) -> Outcome<Type> {
        // trace: D345, D359
        if writer_kind(&format!("os::{rest}")).is_some()
            || matches!(rest, "heap" | "threads" | "events")
        {
            if !plain {
                return Err(Report::refused(
                    span,
                    format!("`os::{rest}` is filled in and described by nothing"),
                    WHY_KIND,
                ));
            }
            return Ok(Type::Struct(format!("os::{rest}")));
        }
        if rest == "stdout" || rest == "stderr" {
            return Err(Report::refused(
                span,
                format!("`os::{rest}` says which kind it is"),
                WHY_KIND,
            ));
        }
        Err(Self::not_in_os(rest, span))
    }

    fn not_in_os(rest: &str, span: Span) -> Box<Report> {
        Report::not_designed(
            span,
            format!("`os::{rest}`, since only `os`'s terminal writers are designed"),
            "",
        )
    }

    /// `os::stdout.buffered(room: 1024 bytes)`, `os::stdout.unbuffered()`, `os::done(Give $out)`,
    /// and a bare `os::stdout()`, which is refused. (D273)
    /// trace: D271, D273
    pub(super) fn check_os_call(
        &mut self,
        rest: &str,
        arguments: &[Argument],
        span: Span,
        path_span: Span,
    ) -> Outcome<RawCall> {
        if self.is_pure() || self.context.in_contract {
            // trace: D86
            return Err(Report::refused(
                path_span,
                "a terminal writer is I/O, which a pure function refuses",
                "a pure function refuses I/O, and contracts can only call pure functions (D86)",
            ));
        }
        if rest == "done" {
            return self.check_done(arguments, span, path_span);
        }
        // trace: D332
        if rest == "heap" {
            return Self::check_heap_call(arguments, span);
        }
        // trace: D345, D346, D392
        if rest == "threads" || rest == "events" {
            return self.check_os_runner_call(rest, arguments, span);
        }
        let Some((stderr, buffered)) = writer_kind(&format!("os::{rest}")) else {
            if rest == "stdout" || rest == "stderr" {
                return Err(Report::refused(
                    path_span,
                    format!("`os::{rest}()` says which kind it is"),
                    WHY_KIND,
                ));
            }
            return Err(Self::not_in_os(rest, path_span));
        };
        let size = if buffered {
            Some(self.writer_size(arguments, span)?)
        } else {
            if let Some(extra) = arguments.first() {
                return Err(Report::refused(
                    extra.value.span,
                    format!("`os::{rest}()` takes nothing"),
                    WHY_KIND,
                ));
            }
            None
        };
        Ok(RawCall {
            node: Node {
                span,
                kind: NodeKind::Writer { stderr, size },
            },
            value_type: Type::Struct(format!("os::{rest}")),
            failures: Vec::new(),
            unnamed: None,
        })
    }

    /// A buffered writer's `room:`, a literal number of `bytes`. (D273, D386)
    fn writer_size(&self, arguments: &[Argument], span: Span) -> Outcome<usize> {
        let [argument] = arguments else {
            return Err(Report::refused(
                arguments.get(1).map_or(span, |extra| extra.value.span),
                "a buffered writer takes one argument, its `room:`",
                WHY_KIND,
            ));
        };
        let shown = self.text_of(argument.value.span).to_string();
        // `room:` is the space for what's put in it, the same word as an arena's. (D386)
        // trace: D386
        if let Some(label) = argument.label.as_ref().filter(|label| label.text == "size") {
            return Err(Report::refused(
                label.span,
                "a buffered writer says `room:`, the space for what it holds",
                "`room:` means the space for what's put in, for buffered writers and arenas alike (D386)",
            )
            .fix("room"));
        }
        if argument
            .label
            .as_ref()
            .is_none_or(|label| label.text != "room")
        {
            return Err(Report::refused(
                argument.value.span,
                "a buffered writer's room says `room:`",
                "a declared label must be written (D113, D273, D386)",
            )
            .fix(format!("room: {shown}")));
        }
        let number = match &argument.value.kind {
            ExpressionKind::WithUnit { number, unit } if unit.text == "bytes" => number,
            ExpressionKind::Integer { .. } => {
                return Err(Report::refused(
                    argument.value.span,
                    "a buffered writer's room says its unit, `bytes`",
                    WHY_KIND,
                )
                .fix(format!("{shown} bytes")));
            }
            _ => {
                return Err(Report::not_designed(
                    argument.value.span,
                    "a buffered writer's room that isn't a literal number of `bytes`",
                    "",
                ));
            }
        };
        match super::expression::literal_integer(number).and_then(|size| usize::try_from(size).ok())
        {
            Some(size) if size > 0 => Ok(size),
            _ => Err(Report::not_designed(
                argument.value.span,
                "a buffered writer whose room isn't a whole number of bytes above zero",
                "",
            )),
        }
    }

    /// The writer type an argument names, read without handing it over yet.
    fn writer_type_of(&self, value: &Expression) -> Option<Type> {
        let inner = match &value.kind {
            ExpressionKind::Hand { value, .. } => value,
            _ => value,
        };
        let ExpressionKind::Variable(name) = &inner.kind else {
            return None;
        };
        let slot = self.find_variable(name)?;
        let mut found = &self.context.variables.get(slot)?.held_type;
        if let Type::Borrow { inner, .. } = found {
            found = inner;
        }
        match found {
            Type::Struct(type_name) if writer_kind(type_name).is_some() => Some(found.clone()),
            _ => None,
        }
    }

    /// A print's or a flush's writer, lent to change, as a node that reads which writer it is.
    /// trace: D272, D273
    pub(super) fn writer_argument(
        &mut self,
        value: &Expression,
        word: ParameterWord,
        handed: &mut Vec<HandedOff>,
    ) -> Outcome<(Node, Type)> {
        let Some(writer_type) = self.writer_type_of(value) else {
            return Err(Report::refused(
                value.span,
                "this isn't a writer, and a print's first argument is the writer it goes to",
                WHY_WRITER,
            ));
        };
        let parameter = ParameterInfo {
            name: "out".to_string(),
            label: None,
            word,
            parameter_type: writer_type.clone(),
            default: None,
            label_has_unit: false,
        };
        let node = match self.check_argument(value, &parameter, handed)? {
            ir::Argument::Value(node) => node,
            ir::Argument::Alias(place) => Node {
                span: value.span,
                kind: NodeKind::Read(place),
            },
        };
        Ok((node, writer_type))
    }

    /// `os::done(Give $out)`: the last flush, which ends a buffered writer's duty. (D273)
    /// trace: D273
    fn check_done(
        &mut self,
        arguments: &[Argument],
        span: Span,
        path_span: Span,
    ) -> Outcome<RawCall> {
        let [argument] = arguments else {
            return Err(Report::refused(
                arguments.get(1).map_or(path_span, |extra| extra.value.span),
                "`os::done` takes one buffered writer",
                "`os::done(Give $out);` flushes a buffered writer for the last time and ends its duty (D273)",
            ));
        };
        // A runner ends the same way. (D392)
        if argument.label.is_none()
            && self
                .variable_type_of(&argument.value)
                .is_some_and(|found| super::concur::is_os_runner(&found))
        {
            return self.check_runner_done(argument, span);
        }
        if let Some(label) = &argument.label {
            return Err(Report::refused(
                label.span,
                "`os::done`'s writer has no label",
                "`os::done(Give $out);` flushes a buffered writer for the last time and ends its duty (D273)",
            ));
        }
        if let Some(Type::Struct(name)) = self.writer_type_of(&argument.value) {
            if !is_buffered_writer(&Type::Struct(name.clone())) {
                return Err(Report::refused(
                    argument.value.span,
                    format!("`{name}` writes each print straight away, so there's nothing for `os::done` to finish"),
                    "only a buffered writer is `must_Give`, and `os::done` is how it ends (D273)",
                ));
            }
        }
        let mut handed = Vec::new();
        let (writer, _) =
            self.writer_argument(&argument.value, ParameterWord::Given, &mut handed)?;
        Ok(RawCall {
            node: Node {
                span,
                kind: NodeKind::Flush {
                    writer: Box::new(writer),
                },
            },
            value_type: Type::Void,
            failures: print_failed(),
            unnamed: None,
        })
    }

    /// `$out.%%flush()` pushes a buffered writer's output now and keeps it going. Nothing else
    /// on a writer is designed. (D273)
    /// trace: D273
    #[allow(clippy::too_many_arguments)] // A method call has this many parts.
    pub(super) fn check_writer_method(
        &mut self,
        receiver: &Expression,
        writer_name: &str,
        kit: Option<&str>,
        name: &crate::ast::Name,
        changes: bool,
        arguments: &[Argument],
        span: Span,
    ) -> Outcome<RawCall> {
        if kit.is_some() || name.text != "flush" {
            return Err(Report::not_designed(
                name.span,
                format!("the method `{}` on `{writer_name}`", name.text),
                "",
            ));
        }
        if !writer_kind(writer_name).is_some_and(|(_, buffered)| buffered) {
            return Err(Report::refused(
                name.span,
                format!("`{writer_name}` writes each print straight away, so it has no `flush`"),
                "only a buffered writer holds output to push (D273)",
            ));
        }
        if !changes {
            return Err(Self::wrong_receiver_mark(name, true));
        }
        if let Some(extra) = arguments.first() {
            return Err(Report::refused(
                extra.value.span,
                "`flush` takes nothing",
                "`$out.%%flush();` pushes a buffered writer's output now and keeps it going (D273)",
            ));
        }
        let lent = Expression {
            span: receiver.span,
            kind: ExpressionKind::Hand {
                word: HandWord::LendMut,
                value: Box::new(receiver.clone()),
            },
        };
        let mut handed = Vec::new();
        let (writer, _) = self.writer_argument(&lent, ParameterWord::LentMut, &mut handed)?;
        Ok(RawCall {
            node: Node {
                span,
                kind: NodeKind::Flush {
                    writer: Box::new(writer),
                },
            },
            value_type: Type::Void,
            failures: print_failed(),
            unnamed: None,
        })
    }
}
