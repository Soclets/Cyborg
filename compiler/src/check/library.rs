//! `std` and `os`, the parts the checker knows by name until `std`'s kit part is written in Cyborg (D22, D271).

use crate::checked;
use crate::diagnostic::Diagnostic;
use crate::source::Span;
use crate::syntax::{self, ExpressionKind, Ownership};

use super::expressions::{When, failure};
use super::format::{self, Part, ShapeWord, Takes};
use super::types::{Type, WriterKind};
use super::{Body, Check, Checker};

const NOT_DECIDED: &str = "not decided yet, and the standard doesn't say";
pub const WRITER_KINDS_WHY: &str = "`os::stdout` and `os::stderr` say which kind they are: `.buffered(size: 1024 bytes)` writes in chunks and is finished with `os::done`, and `.unbuffered()` writes each print straight away (D273)";

impl Checker<'_> {
    /// `os::stdout.buffered(size: 1024 bytes)` or `os::stderr.unbuffered()`, going into a writer (D273).
    pub fn open_writer(
        &mut self,
        body: &mut Body,
        writer: WriterKind,
        value: &syntax::Expression,
        local: checked::LocalId,
    ) -> Check<checked::Statement> {
        let ExpressionKind::Call { callee, arguments, .. } = &value.kind else {
            return Err(self.not_yet(value.span, "a writer that isn't made here".to_string()));
        };
        let ExpressionKind::Path(path) = &callee.kind else {
            return Err(self.not_yet(callee.span, "this writer".to_string()));
        };
        let made = self.resolve_path(path);
        self.require_import(&made, path.span)?;
        if made == "os::stdout" || made == "os::stderr" {
            return Err(self.error(Diagnostic::refused(
                callee.span.to(value.span),
                format!("`{made}()` says which kind it is"),
                WRITER_KINDS_WHY,
            )));
        }
        if made != writer.name() {
            return Err(self.not_yet(callee.span, format!("`{made}` going into `{}`", writer.name())));
        }
        let capacity = if writer.buffered {
            let [argument] = arguments.as_slice() else {
                return Err(self.not_yet(value.span, "a buffered writer without one `size:`".to_string()));
            };
            if argument.label.as_ref().map(|label| label.text.as_str()) != Some("size") {
                return Err(self.not_yet(argument.span, "a buffered writer's size without `size:`".to_string()));
            }
            Some(self.byte_count(body, &argument.value)?)
        } else {
            if let Some(argument) = arguments.first() {
                return Err(self.not_yet(argument.span, "an argument to an unbuffered writer".to_string()));
            }
            None
        };
        Ok(checked::Statement::OpenWriter { local, stream: writer.stream, capacity })
    }

    /// A size in bytes, like `1024 bytes`.
    fn byte_count(&mut self, _body: &mut Body, value: &syntax::Expression) -> Check<checked::Expression> {
        if let ExpressionKind::WithUnit { value: number, unit } = &value.kind
            && unit.is_single("bytes")
            && let ExpressionKind::Integer(text) = &number.kind
            && let Ok(count) = text.replace('_', "").parse::<i128>()
        {
            return Ok(checked::Expression {
                kind: checked::ExpressionKind::Integer(count),
                machine_type: checked::MachineType::I64,
            });
        }
        Err(self.not_yet(value.span, "this size".to_string()))
    }

    /// A call `std` or `os` knows as a statement: a print, a flush or `os::done`. Gives `None` for anything else.
    pub fn library_statement(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
    ) -> Check<Option<checked::Statement>> {
        if let ExpressionKind::MethodCall { receiver, method, arguments, .. } = &expression.kind
            && method.text() == "%%flush"
            && arguments.is_empty()
        {
            let (writer, local) = self.writer_variable(body, receiver)?;
            if !writer.buffered {
                let name_span = method.last().map_or(method.span, |segment| segment.name.span);
                return Err(self.error(Diagnostic::refused(
                    name_span,
                    format!("`{}` writes each print straight away, so it has no `flush`", writer.name()),
                    "only a buffered writer holds output to push (D273)",
                )));
            }
            return Ok(Some(checked::Statement::Flush { writer: local }));
        }
        let ExpressionKind::Call { callee, arguments, arguments_span } = &expression.kind else {
            return Ok(None);
        };
        let ExpressionKind::Path(path) = &callee.kind else {
            return Ok(None);
        };
        let resolved = self.resolve_path(path);
        if matches!(resolved.as_str(), "std::io::fmt_print" | "std::io::fmt_print.stored" | "os::done") {
            self.require_import(&resolved, path.span)?;
        }
        self.old_prints(&resolved, callee.span)?;
        match resolved.as_str() {
            "std::io::fmt_print" => self.print(body, callee.span, arguments, *arguments_span, false).map(Some),
            "std::io::fmt_print.stored" => self.print(body, callee.span, arguments, *arguments_span, true).map(Some),
            "os::done" => {
                let [argument] = arguments.as_slice() else {
                    return Err(self.not_yet(expression.span, "`os::done` without one writer".to_string()));
                };
                if argument.ownership.map(|(ownership, _)| ownership) != Some(Ownership::Give) {
                    return Err(self.not_yet(argument.span, "`os::done` without `Give`".to_string()));
                }
                let (writer, local) = self.writer_variable(body, &argument.value)?;
                if !writer.buffered {
                    return Err(self.not_yet(expression.span, "`os::done` on an unbuffered writer".to_string()));
                }
                if let ExpressionKind::Variable(name) = &argument.value.kind
                    && let Some(variable) = body.lookup_mut(&name.text)
                {
                    variable.given = Some(argument.span);
                }
                Ok(Some(checked::Statement::Done { writer: local }))
            }
            _ => Ok(None),
        }
    }

    /// Prints that are gone: `std::println` and every other print but `std::io::fmt_print` (D194, D197), and a print
    /// that says where it goes, since the writer does (D272).
    pub fn old_prints(&mut self, resolved: &str, span: Span) -> Check<()> {
        if resolved == "std::println" || resolved.starts_with("std::println.") {
            return Err(self.error(Diagnostic::refused(
                span,
                format!("`{resolved}` is gone, and `std` has one print"),
                "`std::io::fmt_print` is the one print, a format print, with `{newln}` for a newline (D194, D197)",
            )));
        }
        if matches!(resolved, "std::print" | "std::io::fmt_println" | "std::io::print") {
            return Err(self.error(Diagnostic::refused_fix(
                span,
                "std::io::fmt_print",
                "the print is `std::io::fmt_print` (D197)",
            )));
        }
        for stream in ["stdout", "stderr"] {
            if resolved == format!("std::io::fmt_print.{stream}") {
                return Err(self.error(Diagnostic::refused(
                    span,
                    format!("`.{stream}` is gone, since the writer says where output goes"),
                    format!(
                        "`std::io::fmt_print` always takes a writer first, like `Lend.mut $out` for `os::{stream}.buffered(size: 1024 bytes)` or `os::{stream}.unbuffered()` (D272, D273)"
                    ),
                )));
            }
        }
        Ok(())
    }

    /// A writer held in a variable.
    fn writer_variable(&mut self, body: &Body, value: &syntax::Expression) -> Check<(WriterKind, checked::LocalId)> {
        if let ExpressionKind::Variable(name) = &value.kind
            && let Some(variable) = body.lookup(&name.text)
        {
            if let Type::Writer(writer) = variable.value_type {
                return Ok((writer, variable.local));
            }
            if variable.value_type.is_unknown() {
                return Err(super::Reported);
            }
        }
        Err(self.not_yet(value.span, "printing to something that isn't a writer variable".to_string()))
    }

    /// `std::io::fmt_print(Lend.mut $out, "text {}", values…)` (D194, D197, D272).
    fn print(
        &mut self,
        body: &mut Body,
        callee: Span,
        arguments: &[syntax::Argument],
        arguments_span: Span,
        stored: bool,
    ) -> Check<checked::Statement> {
        let Some(writer_argument) = arguments.first() else {
            return Err(self.not_yet(callee.to(arguments_span), "a print with no writer".to_string()));
        };
        if matches!(writer_argument.value.kind, ExpressionKind::Text(_)) || writer_argument.label.is_some() {
            return Err(self.error(Diagnostic::refused(
                writer_argument.span,
                "this isn't a writer, and a print's first argument is the writer it goes to",
                "`std::io::fmt_print` always takes a writer first, as in `fmt_print(Lend.mut $out, \"hi{newln}\");`, since the writer says where output goes (D272, D273)",
            )));
        }
        if writer_argument.ownership.is_none() && matches!(writer_argument.value.kind, ExpressionKind::Variable(_)) {
            let written = self.slice(writer_argument.value.span).to_string();
            return Err(self.error(Diagnostic::refused_fix(
                writer_argument.span,
                format!("Lend.mut {written}"),
                "every hand-off shows on the page (D3)",
            )));
        }
        if writer_argument.ownership.map(|(ownership, _)| ownership) != Some(Ownership::LendMut) {
            return Err(self.not_yet(writer_argument.span, "a writer not lent with `Lend.mut`".to_string()));
        }
        let (_, writer) = self.writer_variable(body, &writer_argument.value)?;
        let Some(text_argument) = arguments.get(1) else {
            return Err(self.error(Diagnostic::not_designed(
                callee.to(arguments_span),
                "`std::io::fmt_print()` with no text",
                "not decided yet (standard 12.1)",
            )));
        };
        let ExpressionKind::Text(raw) = &text_argument.value.kind else {
            return Err(self.error(Diagnostic::not_designed(
                text_argument.span,
                "a format that isn't a literal",
                NOT_DECIDED,
            )));
        };
        let start = text_argument.value.span.start as usize;
        let mut errors = Vec::new();
        let parts = format::read(self.file, raw, start, &mut errors);
        let failed = !errors.is_empty();
        self.errors.extend(errors);
        let parts = parts.ok_or(super::Reported)?;
        if failed {
            return Err(super::Reported);
        }
        let values = &arguments[2..];
        for value in values {
            if let Some(label) = &value.label {
                return Err(self.error(Diagnostic::refused(
                    label.span,
                    "a print's text and values have no labels",
                    "`std` labels every other argument, except where every one has the same role, like a print's values (D114)",
                )));
            }
        }
        let format_span = text_argument.value.span;
        let holes: Vec<&format::Hole> =
            parts.iter().filter_map(|part| if let Part::Hole(hole) = part { Some(hole) } else { None }).collect();
        let numbered = holes.iter().any(|hole| matches!(hole.takes, Takes::Numbered(_)));
        let plain = holes.iter().any(|hole| matches!(hole.takes, Takes::Next));
        if numbered && plain {
            let mixed =
                holes.iter().find(|hole| matches!(hole.takes, Takes::Next)).map_or(format_span, |hole| hole.span);
            return Err(self.error(Diagnostic::refused(
                mixed,
                "this mixes plain holes and numbered ones",
                "`{1}`, `{2}` are numbered holes, and they can't be mixed with plain ones (D194)",
            )));
        }
        // Which value each hole prints.
        let mut taken = Vec::new();
        let mut next = 0;
        for hole in &holes {
            match &hole.takes {
                Takes::Next => {
                    taken.push(next);
                    next += 1;
                }
                Takes::Numbered(number) => taken.push(number - 1),
                Takes::Named { span, .. } => {
                    return Err(self.not_yet(*span, "a named hole".to_string()));
                }
            }
        }
        let needed = if numbered { taken.iter().max().map_or(0, |most| most + 1) } else { next };
        if needed != values.len() && (!numbered || needed > values.len()) {
            let noun = if needed == 1 { "value" } else { "values" };
            return Err(self.error(Diagnostic::refused(
                format_span,
                format!("this format takes {needed} {noun}, and it's given {}", values.len()),
                "a literal format is checked when building, including how many values it takes (D194)",
            )));
        }
        if numbered && let Some(unused) = (0..values.len()).find(|index| !taken.contains(index)) {
            return Err(self.error(Diagnostic::refused(
                format_span,
                format!("value {} has no hole", unused + 1),
                "a literal format is checked when building, and every value goes in a hole (D194)",
            )));
        }
        let mut pieces = Vec::new();
        let mut hole_index = 0;
        for part in &parts {
            match part {
                Part::Text(text) => pieces.push(checked::Piece::Text(text.clone())),
                Part::Hole(hole) => {
                    let value = &values[taken[hole_index]];
                    hole_index += 1;
                    pieces.push(self.print_value(body, hole, value, stored)?);
                }
            }
        }
        Ok(checked::Statement::Print { writer, pieces })
    }

    /// One value in a hole, with its shapes checked against its type.
    fn print_value(
        &mut self,
        body: &mut Body,
        hole: &format::Hole,
        argument: &syntax::Argument,
        stored: bool,
    ) -> Check<checked::Piece> {
        if let Some((_, span)) = argument.ownership {
            return Err(self.error(Diagnostic::not_designed(
                span.to(argument.value.span),
                "a word on a print's value",
                NOT_DECIDED,
            )));
        }
        let typed = self.expression(body, &argument.value, None)?;
        let (value_type, value) = (typed.value_type, typed.value);
        let mut shape = checked::Shape { stored, ..checked::Shape::default() };
        for word in &hole.shapes {
            let applies = match (word, &value_type) {
                (ShapeWord::Align(_) | ShapeWord::Width(..), _) => true,
                (ShapeWord::Zeros | ShapeWord::Sign | ShapeWord::Base(_), Type::Integer { .. }) => true,
                (ShapeWord::Places(_) | ShapeWord::Science, Type::Float { .. }) => true,
                (ShapeWord::Zeros | ShapeWord::Sign, Type::Float { .. }) => true,
                _ => false,
            };
            if !applies {
                return Err(self.error(Diagnostic::refused(
                    hole.shapes_span,
                    format!("`{}` doesn't apply to `{value_type}`", word.label()),
                    "a literal format is checked when building, including a shape that doesn't apply (D194)",
                )));
            }
            match word {
                ShapeWord::Align(align) => shape.align = Some(*align),
                ShapeWord::Width(width, unit) => {
                    shape.width = Some(*width);
                    shape.width_unit = match unit {
                        format::WidthUnit::Columns => checked::WidthUnit::Columns,
                        format::WidthUnit::ColumnsEastAsian => checked::WidthUnit::ColumnsEastAsian,
                        format::WidthUnit::Graphemes => checked::WidthUnit::Graphemes,
                        format::WidthUnit::Bytes => checked::WidthUnit::Bytes,
                    };
                }
                ShapeWord::Places(places) => shape.places = Some(*places),
                ShapeWord::Base(base) => shape.base = *base,
                ShapeWord::Science => shape.science = true,
                ShapeWord::Zeros => shape.zeros = true,
                ShapeWord::Sign => shape.sign = true,
                ShapeWord::Unit => {
                    return Err(self.not_yet(hole.span, "`unit` in a hole".to_string()));
                }
            }
        }
        if shape.zeros && shape.width.is_none() {
            return Err(self.error(Diagnostic::refused(
                hole.shapes_span,
                "`zeros` needs a width to fill",
                "`zeros` pads a value to its width (D194)",
            )));
        }
        if shape.width.is_some() && shape.align.is_none() && !shape.zeros {
            return Err(self.error(Diagnostic::not_designed(
                argument.value.span,
                "which way a value lines up when only its width is given",
                NOT_DECIDED,
            )));
        }
        if let Type::Integer { kind, .. } = &value_type
            && kind.signed
            && shape.base != checked::Base::Decimal
        {
            // A negative number in another base isn't decided, so a signed one is checked while running.
            return Ok(checked::Piece::Value {
                value,
                shape,
                failures: Some(Box::new(checked::FloatPrintFailures {
                    halfway: failure(
                        self,
                        Diagnostic::not_designed(
                            hole.shapes_span,
                            "how a negative number prints with `base:`",
                            NOT_DECIDED,
                        ),
                        &[],
                    ),
                    science: failure(self, Diagnostic::not_designed(hole.span, "", NOT_DECIDED), &[]),
                    nan: failure(self, Diagnostic::not_designed(hole.span, "", NOT_DECIDED), &[]),
                })),
            });
        }
        if let Type::Float { kind, .. } = &value_type {
            if stored && (shape.places.is_some() || shape.science) {
                return Err(self.error(Diagnostic::not_designed(
                    argument.value.span,
                    "`places:` or `form:` in a `.stored` print",
                    "not decided yet (standard 12.2)",
                )));
            }
            let failures = checked::FloatPrintFailures {
                halfway: failure(
                    self,
                    Diagnostic::not_designed(hole.shapes_span, "which way a value exactly halfway rounds", NOT_DECIDED),
                    &[When::Text("the value is ".into()), When::Operand(0)],
                ),
                science: failure(
                    self,
                    Diagnostic::not_designed(
                        hole.shapes_span,
                        "a negative exponent, or zero, in `form: science`",
                        "not decided yet (standard 12.2)",
                    ),
                    &[],
                ),
                nan: failure(
                    self,
                    Diagnostic::not_designed(hole.shapes_span, "how `NaN` prints with shapes on it", NOT_DECIDED),
                    &[],
                ),
            };
            let _ = kind;
            return Ok(checked::Piece::Value { value, shape, failures: Some(Box::new(failures)) });
        }
        match value_type {
            Type::Integer { .. } => Ok(checked::Piece::Value { value, shape, failures: None }),
            Type::Str => {
                if let (checked::ExpressionKind::Text(text), Some(width), Some(checked::Align::Center)) =
                    (&value.kind, shape.width, shape.align)
                    && width > shape.columns_of(text)
                    && (width - shape.columns_of(text)) % 2 == 1
                {
                    return Err(self.error(Diagnostic::not_designed(
                        hole.span,
                        "which side of a centred value gets the odd space",
                        NOT_DECIDED,
                    )));
                }
                if shape.width.is_some() && !matches!(value.kind, checked::ExpressionKind::Text(_)) {
                    return Err(
                        self.not_yet(argument.value.span, "a width on text known only while running".to_string())
                    );
                }
                Ok(checked::Piece::Value { value, shape, failures: None })
            }
            Type::Bool => Err(self.error(Diagnostic::not_designed(
                argument.value.span,
                "how a `bool` prints",
                "not decided yet (standard 12.3)",
            ))),
            Type::Unknown => Err(super::Reported),
            other => Err(self.not_yet(argument.value.span, format!("printing `{other}`"))),
        }
    }
}
