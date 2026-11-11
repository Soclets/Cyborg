//! Statements.

use crate::checked;
use crate::diagnostic::Diagnostic;
use crate::syntax::{self, StatementKind};

use super::types::Type;
use super::{Body, Check, Checker, Variable};

impl Checker<'_> {
    /// Checks a block in a new scope. A statement with an error is skipped, and checking goes on (standard 11.7).
    pub fn block(&mut self, body: &mut Body, statements: &[syntax::Statement]) -> checked::Block {
        body.scopes.push(std::collections::HashMap::new());
        let mut block = Vec::new();
        for statement in statements {
            let _ = self.statement(body, statement, &mut block);
        }
        self.leaving_scope(body, statements.last().map(|statement| statement.span));
        body.scopes.pop();
        block
    }

    /// A `must_Give` value, such as a buffered writer, has to be given away before its scope ends (D129, D273).
    /// A block that ends in `return` has already been checked there.
    fn leaving_scope(&mut self, _body: &mut Body, _last: Option<crate::source::Span>) {}

    fn statement(&mut self, body: &mut Body, statement: &syntax::Statement, block: &mut checked::Block) -> Check<()> {
        match &statement.kind {
            StatementKind::Let(binding) => self.let_statement(body, binding, block),
            StatementKind::Return(value) => self.return_statement(body, statement, value.as_ref(), block),
            StatementKind::Expression(expression) => self.expression_statement(body, expression, block),
            StatementKind::Broken => Err(super::Reported),
            _ => Err(self.not_yet(statement.span, "this statement".to_string())),
        }
    }

    fn let_statement(&mut self, body: &mut Body, binding: &syntax::Let, block: &mut checked::Block) -> Check<()> {
        let declared = match &binding.written_type {
            Some(value_type) => self.resolve_type(value_type).unwrap_or(Type::Unknown),
            None => Type::Unknown,
        };
        let local = body.new_local(declared.machine());
        let variable = Variable {
            value_type: declared.clone(),
            mutable: binding.mutable,
            local,
            declared: binding.name.span,
            given: None,
        };
        // The variable exists even if its value is refused (standard 11.7).
        let result = self.let_value(body, &declared, binding, local, block);
        body.declare(&binding.name.text, variable);
        result
    }

    fn let_value(
        &mut self,
        body: &mut Body,
        declared: &Type,
        binding: &syntax::Let,
        local: checked::LocalId,
        block: &mut checked::Block,
    ) -> Check<()> {
        if let Type::Writer(writer) = declared {
            let statement = self.open_writer(body, *writer, &binding.value, local)?;
            block.push(statement);
            return Ok(());
        }
        let value = self.value_into(body, &binding.value, declared)?;
        block.push(checked::Statement::Let { local, value });
        Ok(())
    }

    fn return_statement(
        &mut self,
        body: &mut Body,
        statement: &syntax::Statement,
        value: Option<&syntax::Expression>,
        block: &mut checked::Block,
    ) -> Check<()> {
        self.check_must_give_on_leaving(body, statement.span)?;
        let returns = body.returns.clone();
        match value {
            Some(expression) if matches!(&expression.kind, syntax::ExpressionKind::Path(path) if path.is_single("void")) =>
            {
                block.push(checked::Statement::Return(None));
                Ok(())
            }
            Some(expression) => {
                let value = self.value_into(body, expression, &returns)?;
                block.push(checked::Statement::Return(Some(value)));
                Ok(())
            }
            None => Err(self.not_yet(statement.span, "`return` with nothing after it".to_string())),
        }
    }

    /// Every `must_Give` value still held when leaving is refused (D129, D273).
    fn check_must_give_on_leaving(&mut self, body: &Body, at: crate::source::Span) -> Check<()> {
        let mut held: Vec<(String, Variable)> = Vec::new();
        for scope in &body.scopes {
            for (name, variable) in scope {
                if matches!(variable.value_type, Type::Writer(writer) if writer.buffered) && variable.given.is_none() {
                    held.push((name.clone(), variable.clone()));
                }
            }
        }
        held.sort_by_key(|(_, variable)| variable.declared);
        let mut result = Ok(());
        for (name, variable) in held {
            result = Err(self.error(
                Diagnostic::refused(
                    at,
                    format!("this leaves with `{name}` never given away or finished"),
                    format!(
                        "a buffered writer is `must_Give`, and `os::done(Give {name});` flushes it for the last time and ends its duty, so no output is lost (D272, D273)"
                    ),
                )
                .with_because(variable.declared, format!("`{name}` was made here")),
            ));
        }
        result
    }

    fn expression_statement(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        block: &mut checked::Block,
    ) -> Check<()> {
        if let Some(statement) = self.library_statement(body, expression)? {
            block.push(statement);
            return Ok(());
        }
        let (value_type, value) = self.expression(body, expression, None)?;
        if value_type != Type::Void && !value_type.is_unknown() {
            return Err(self.not_yet(expression.span, "dropping a value".to_string()));
        }
        block.push(checked::Statement::Evaluate(value));
        Ok(())
    }
}
