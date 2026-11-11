//! Machine code, through Lyftar (rule 1).
//!
//! The runtime's IR is read into the module first, so the program's code calls straight into it and LLVM's passes
//! can inline it. Every local gets a stack slot in the function's entry block, which LLVM turns into registers.

use std::collections::HashMap;

use lyftar::enums::{IntegerPredicate, Linkage, UnnamedAddress};
use lyftar::intrinsics::Intrinsic;
use lyftar::passes::PassOptions;
use lyftar::target::{
    CodeGenerationFileType, CodeGenerationOptimizationLevel, CodeModel, RelocationMode, Target as LlvmTarget,
};
use lyftar::{Builder, with_context};

use crate::checked::{
    Argument, Arithmetic, Base, Block, CheckFailure, Comparison, Expression, ExpressionKind, MachineType, Passing,
    Piece, Program, Shape, Statement,
};
use crate::target::{Mode, Target};

const OS_RUNTIME: &str = include_str!("runtime/os.ll");
const BARE_RUNTIME: &str = include_str!("runtime/bare.ll");
const BOARD_STARTUP: &str = include_str!("runtime/arduino_uno_r4_wifi/startup.ll");

/// The linker script for the Arduino Uno R4 `WiFi` (D261, D281).
pub const BOARD_MEMORY: &str = include_str!("runtime/arduino_uno_r4_wifi/memory.ld");

/// Builds the object file for a checked program.
///
/// # Errors
/// What LLVM reported, which is a bug in this compiler, since a checked program is always valid.
pub fn object_file(program: &Program, target: Target, mode: Mode) -> Result<Vec<u8>> {
    with_context(|context| {
        let triple = target.triple();
        let level = match mode {
            Mode::Literal => CodeGenerationOptimizationLevel::None,
            Mode::Working => CodeGenerationOptimizationLevel::Less,
            _ => CodeGenerationOptimizationLevel::Aggressive,
        };
        let relocation = if target.has_os() { RelocationMode::PositionIndependentCode } else { RelocationMode::Static };
        let machine = LlvmTarget::from_triple(triple)
            .map_err(|error| error.message().to_string())?
            .create_target_machine(triple, target.cpu(), "", level, relocation, CodeModel::Default);
        let runtime = if target.has_os() { OS_RUNTIME.to_string() } else { format!("{BARE_RUNTIME}\n{BOARD_STARTUP}") };
        let mut module =
            context.parse_ir_text(&runtime).map_err(|error| format!("the runtime: {}", error.message()))?;
        {
            let view = context.module(&module);
            view.set_target_machine(&machine);
            let mut generator =
                Generator { context, module: view, mode, bare_metal: !target.has_os(), functions: Vec::new() };
            generator.program(program)?;
            view.verify().map_err(|error| format!("the generated code: {}", error.message()))?;
        }
        context
            .exclusive_module(&mut module)
            .run_passes(mode.pipeline(), Some(&machine), &PassOptions::default())
            .map_err(|error| error.message().to_string())?;
        let buffer = machine
            .emit_to_memory_buffer(context.exclusive_module(&mut module), CodeGenerationFileType::Object)
            .map_err(|error| error.message().to_string())?;
        Ok(buffer.as_bytes().to_vec())
    })
}

type Value<'context, 'module> = lyftar::Value<'context, 'module>;
type F32 = lyftar::Float<lyftar::Ieee754Binary<lyftar::Bits<32>>>;
type F64 = lyftar::Float<lyftar::Ieee754Binary<lyftar::Bits<64>>>;
type BasicBlock<'context, 'module> = lyftar::basic_block::BasicBlock<'context, 'module>;
type Function<'context, 'module> = lyftar::Value<'context, 'module, lyftar::Function>;
type Result<T> = std::result::Result<T, String>;

struct Generator<'context, 'module> {
    context: &'module lyftar::context::Context<'context>,
    module: &'module lyftar::module::Module<'context>,
    mode: Mode,
    /// With no operating system, a failed check traps, since there's nowhere decided to print to.
    bare_metal: bool,
    functions: Vec<Function<'context, 'module>>,
}

/// Where each local lives while generating one function.
struct Frame<'context, 'module> {
    function: Function<'context, 'module>,
    slots: Vec<Value<'context, 'module>>,
    loops: HashMap<usize, BasicBlock<'context, 'module>>,
    builder: Builder<'context, 'module>,
    /// Whether this is `fn.main`, which returns 0 to the system.
    is_main: bool,
}

/// A value with its machine type, which a failure's `when:` line prints.
type Operand<'context, 'module> = (Value<'context, 'module>, MachineType);

impl<'context, 'module> Generator<'context, 'module> {
    fn runtime(&self, name: &str) -> Result<Function<'context, 'module>> {
        self.module.get_function(name).ok_or_else(|| format!("the runtime has no `{name}`"))
    }

    fn machine_type(&self, machine_type: &MachineType) -> lyftar::types::Type<'context> {
        match machine_type {
            MachineType::Integer { bits, .. } => self.context.integer_type(*bits).to_dynamic(),
            MachineType::Float { bits: 32 } => self.context.get_type::<F32>().to_dynamic(),
            MachineType::Float { .. } => self.context.get_type::<F64>().to_dynamic(),
            MachineType::Bool => self.context.integer_type(1).to_dynamic(),
            MachineType::Void => self.context.void_type(),
            MachineType::Text => self
                .context
                .struct_type(&[self.context.pointer_type(0), self.context.integer_type(64).to_dynamic()], false),
            MachineType::Writer => {
                self.context.get_named_struct("writer").unwrap_or_else(|| self.context.integer_type(8).to_dynamic())
            }
        }
    }

    fn integer(&self, bits: u32, value: u64) -> Value<'context, 'module> {
        self.context.integer_type(bits).constant_integer(value, false).to_dynamic()
    }

    fn program(&mut self, program: &Program) -> Result<()> {
        let pointer = self.context.pointer_type(0);
        for (index, function) in program.functions.iter().enumerate() {
            let is_main = index == program.entry;
            let returns =
                if is_main { self.context.integer_type(32).to_dynamic() } else { self.machine_type(&function.returns) };
            let parameters: Vec<_> = function
                .parameters
                .iter()
                .map(|parameter| match parameter.passing {
                    Passing::Value => self.machine_type(&function.locals[parameter.local]),
                    Passing::Address => pointer,
                })
                .collect();
            let declared =
                self.module.add_function(&function.symbol, self.context.function_type(returns, &parameters, false));
            if !is_main {
                declared.set_linkage(Linkage::Internal);
            }
            self.functions.push(declared);
        }
        for (index, function) in program.functions.iter().enumerate() {
            self.function(index, function, index == program.entry)?;
        }
        Ok(())
    }

    fn function(&mut self, index: usize, function: &crate::checked::Function, is_main: bool) -> Result<()> {
        let declared = *self.functions.get(index).ok_or("a function that wasn't declared")?;
        let entry = declared.append_basic_block("entry");
        let builder = Builder::at_end_of(entry);
        let mut slots = Vec::with_capacity(function.locals.len());
        for local in &function.locals {
            let slot_type = match local {
                MachineType::Void => self.context.integer_type(8).to_dynamic(),
                other => self.machine_type(other),
            };
            slots.push(builder.stack_allocate(slot_type, "").to_dynamic());
        }
        for (position, parameter) in function.parameters.iter().enumerate() {
            let incoming = declared.parameter(u32::try_from(position).unwrap_or(u32::MAX));
            match parameter.passing {
                Passing::Value => {
                    builder.store(incoming, slots[parameter.local]);
                }
                Passing::Address => slots[parameter.local] = incoming,
            }
        }
        let mut frame = Frame { function: declared, slots, loops: HashMap::new(), builder, is_main };
        self.block(&mut frame, &function.body)?;
        // The end of a body the checker let run off its end can't be reached, since every function returns.
        if frame.builder.insertion_basic_block().terminator().is_none() {
            if is_main {
                frame.builder.return_value(self.context.integer_type(32).constant_integer(0, false));
            } else if matches!(function.returns, MachineType::Void) {
                frame.builder.return_void();
            } else {
                frame.builder.unreachable();
            }
        }
        Ok(())
    }

    /// Starts a new block after a terminator, so code after a `return` or `break` has somewhere to go.
    fn ensure_open(frame: &mut Frame<'context, 'module>) {
        if frame.builder.insertion_basic_block().terminator().is_some() {
            let dead = frame.function.append_basic_block("after");
            frame.builder.position_at_end(dead);
        }
    }

    fn block(&mut self, frame: &mut Frame<'context, 'module>, block: &Block) -> Result<()> {
        for statement in block {
            Self::ensure_open(frame);
            self.statement(frame, statement)?;
        }
        Ok(())
    }

    fn branch_if_open(frame: &mut Frame<'context, 'module>, to: BasicBlock<'context, 'module>) {
        if frame.builder.insertion_basic_block().terminator().is_none() {
            frame.builder.branch(to);
        }
    }

    fn statement(&mut self, frame: &mut Frame<'context, 'module>, statement: &Statement) -> Result<()> {
        match statement {
            Statement::Let { local, value } | Statement::Set { local, value } => {
                let value = self.expression(frame, value)?;
                frame.builder.store(value, frame.slots[*local]);
            }
            Statement::Evaluate(value) => {
                self.expression(frame, value)?;
            }
            Statement::If { condition, then, otherwise } => {
                let condition = self.expression(frame, condition)?;
                let then_block = frame.function.append_basic_block("then");
                let otherwise_block = frame.function.append_basic_block("otherwise");
                let after = frame.function.append_basic_block("after_if");
                frame.builder.conditional_branch(condition, then_block, otherwise_block);
                frame.builder.position_at_end(then_block);
                self.block(frame, then)?;
                Self::branch_if_open(frame, after);
                frame.builder.position_at_end(otherwise_block);
                self.block(frame, otherwise)?;
                Self::branch_if_open(frame, after);
                frame.builder.position_at_end(after);
            }
            Statement::Loop { id, body } => {
                let top = frame.function.append_basic_block("loop");
                let after = frame.function.append_basic_block("after_loop");
                frame.loops.insert(*id, after);
                frame.builder.branch(top);
                frame.builder.position_at_end(top);
                self.block(frame, body)?;
                Self::branch_if_open(frame, top);
                frame.builder.position_at_end(after);
            }
            Statement::Break(id) => {
                let after = *frame.loops.get(id).ok_or("a break outside its loop")?;
                frame.builder.branch(after);
            }
            Statement::Return(value) => match value {
                Some(value) => {
                    let value = self.expression(frame, value)?;
                    frame.builder.return_value(value);
                }
                None if frame.is_main => {
                    frame.builder.return_value(self.context.integer_type(32).constant_integer(0, false));
                }
                None => {
                    frame.builder.return_void();
                }
            },
            Statement::OpenWriter { local, stream, capacity } => {
                let open = self.runtime("__cyborg_writer_open")?;
                let descriptor = self.integer(32, u64::from(stream.descriptor().unsigned_abs()));
                let capacity = match capacity {
                    Some(capacity) => self.expression(frame, capacity)?,
                    None => self.integer(64, 0),
                };
                frame.builder.call(open, &[frame.slots[*local], descriptor, capacity], "");
            }
            Statement::Print { writer, pieces } => {
                let writer = frame.slots[*writer];
                self.pieces(frame, writer, pieces)?;
            }
            Statement::Flush { writer } => {
                let flush = self.runtime("__cyborg_writer_flush")?;
                frame.builder.call(flush, &[frame.slots[*writer]], "");
            }
            Statement::Done { writer } => {
                let done = self.runtime("__cyborg_writer_done")?;
                frame.builder.call(done, &[frame.slots[*writer]], "");
            }
            Statement::Check { condition, failure, operands } => {
                let mut values = Vec::new();
                for operand in operands {
                    values.push((self.expression(frame, operand)?, operand.machine_type.clone()));
                }
                let condition = self.expression(frame, condition)?;
                self.check(frame, condition, failure, &values)?;
            }
        }
        Ok(())
    }

    /// Crashes with `failure`'s message unless `holds` is true. Checks stay in every mode for now, since the modes
    /// that drop them need Precog's proofs first.
    fn check(
        &mut self,
        frame: &mut Frame<'context, 'module>,
        holds: Value<'context, 'module>,
        failure: &CheckFailure,
        operands: &[Operand<'context, 'module>],
    ) -> Result<()> {
        let fine = frame.function.append_basic_block("fine");
        let failed = frame.function.append_basic_block("failed");
        frame.builder.conditional_branch(holds, fine, failed);
        frame.builder.position_at_end(failed);
        if self.bare_metal {
            frame.builder.call(self.runtime("__cyborg_trap")?, &[] as &[Value<'context, 'module>], "");
        } else {
            let writer_type = self.machine_type(&MachineType::Writer);
            let writer = frame.builder.stack_allocate(writer_type, "").to_dynamic();
            frame.builder.call(self.runtime("__cyborg_crash_begin")?, &[writer], "");
            self.text(frame, writer, &failure.before)?;
            for (text, operand) in &failure.when {
                self.text(frame, writer, text)?;
                let (value, machine_type) =
                    operands.get(*operand).cloned().ok_or("a failure names a missing operand")?;
                self.print_number(frame, writer, value, &machine_type, &Shape::default())?;
            }
            self.text(frame, writer, &failure.after)?;
            frame.builder.call(self.runtime("__cyborg_exit")?, &[self.integer(32, 1)], "");
        }
        frame.builder.unreachable();
        frame.builder.position_at_end(fine);
        Ok(())
    }

    fn pieces(
        &mut self,
        frame: &mut Frame<'context, 'module>,
        writer: Value<'context, 'module>,
        pieces: &[Piece],
    ) -> Result<()> {
        for piece in pieces {
            match piece {
                Piece::Text(text) => self.text(frame, writer, text)?,
                Piece::Value { value, shape } => {
                    let generated = self.expression(frame, value)?;
                    match &value.machine_type {
                        MachineType::Text => {
                            let columns = match &value.kind {
                                ExpressionKind::Text(text) => shape.columns_of(text),
                                _ => return Err("printing a string that isn't a literal".to_string()),
                            };
                            let data = frame.builder.extract_value(generated, 0, "").to_dynamic();
                            let length = frame.builder.extract_value(generated, 1, "").to_dynamic();
                            let align = match shape.align {
                                Some(crate::checked::Align::Left) => 0,
                                Some(crate::checked::Align::Center) => 2,
                                _ => 1,
                            };
                            let arguments = [
                                writer,
                                data,
                                length,
                                self.integer(64, columns),
                                self.integer(64, shape.width.unwrap_or(0)),
                                self.integer(32, align),
                            ];
                            frame.builder.call(self.runtime("__cyborg_print_text")?, &arguments, "");
                        }
                        other => self.print_number(frame, writer, generated, other, shape)?,
                    }
                }
            }
        }
        Ok(())
    }

    /// A constant string's bytes, as a private global.
    fn constant_bytes(&self, text: &str) -> Value<'context, 'module> {
        let constant = self.context.constant_string(text.as_bytes(), false);
        let global = self.module.add_global("", constant.get_type());
        global.set_initializer(constant);
        global.set_global_constant(true);
        global.set_linkage(Linkage::Private);
        global.set_unnamed_address(UnnamedAddress::Global);
        global.to_dynamic()
    }

    /// Writes constant text.
    fn text(
        &mut self,
        frame: &mut Frame<'context, 'module>,
        writer: Value<'context, 'module>,
        text: &str,
    ) -> Result<()> {
        if text.is_empty() {
            return Ok(());
        }
        let bytes = self.constant_bytes(text);
        let length = self.integer(64, text.len() as u64);
        frame.builder.call(self.runtime("__cyborg_writer_write")?, &[writer, bytes, length], "");
        Ok(())
    }

    /// Prints a number or a `bool` with its shape.
    fn print_number(
        &mut self,
        frame: &mut Frame<'context, 'module>,
        writer: Value<'context, 'module>,
        value: Value<'context, 'module>,
        machine_type: &MachineType,
        shape: &Shape,
    ) -> Result<()> {
        let MachineType::Integer { bits, signed } = *machine_type else {
            return Err(format!("printing {machine_type:?} isn't generated yet"));
        };
        let i64_type = self.context.integer_type(64);
        let wide = match bits.cmp(&64) {
            std::cmp::Ordering::Equal => value,
            std::cmp::Ordering::Greater => frame.builder.truncate(value, i64_type, "").to_dynamic(),
            std::cmp::Ordering::Less if signed => frame.builder.sign_extend(value, i64_type, "").to_dynamic(),
            std::cmp::Ordering::Less => frame.builder.zero_extend(value, i64_type, "").to_dynamic(),
        };
        let zero = self.integer(64, 0);
        let (magnitude, negative) = if signed {
            let negative = frame.builder.integer_compare(IntegerPredicate::SignedLessThan, wide, zero, "").to_dynamic();
            let flipped = frame.builder.integer_subtract(zero, wide, "").to_dynamic();
            (frame.builder.select(negative, flipped, wide, "").to_dynamic(), negative)
        } else {
            (wide, self.integer(1, 0))
        };
        let (base, upper) = match shape.base {
            Base::Decimal => (10, false),
            Base::Binary => (2, false),
            Base::Octal => (8, false),
            Base::Hex => (16, false),
            Base::HexUpper => (16, true),
        };
        let align = match shape.align {
            Some(crate::checked::Align::Left) => 0,
            Some(crate::checked::Align::Center) => 2,
            _ => 1,
        };
        let arguments = [
            writer,
            magnitude,
            negative,
            self.integer(64, base),
            self.integer(1, u64::from(upper)),
            self.integer(64, shape.width.unwrap_or(0)),
            self.integer(32, align),
            self.integer(1, u64::from(shape.zeros)),
            self.integer(1, u64::from(shape.sign)),
        ];
        frame.builder.call(self.runtime("__cyborg_print_integer")?, &arguments, "");
        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    fn expression(
        &mut self,
        frame: &mut Frame<'context, 'module>,
        expression: &Expression,
    ) -> Result<Value<'context, 'module>> {
        let value = match &expression.kind {
            ExpressionKind::Integer(value) => {
                let MachineType::Integer { bits, .. } = expression.machine_type else {
                    return Err("an integer that isn't one".to_string());
                };
                integer_constant(self.context, bits, *value)
            }
            ExpressionKind::Float(value) => self.context.get_type::<F64>().constant_float(*value).to_dynamic(),
            ExpressionKind::Bool(value) => self.integer(1, u64::from(*value)),
            ExpressionKind::Text(text) => {
                let bytes = self.constant_bytes(text);
                let text_type = self.machine_type(&MachineType::Text);
                let length = self.integer(64, text.len() as u64);
                let empty = text_type.undefined().to_dynamic();
                let with_data = frame.builder.insert_value(empty, bytes, 0, "").to_dynamic();
                frame.builder.insert_value(with_data, length, 1, "").to_dynamic()
            }
            ExpressionKind::Local(local) => {
                let loaded_type = self.machine_type(&expression.machine_type);
                frame.builder.load(loaded_type, frame.slots[*local], "").to_dynamic()
            }
            ExpressionKind::Arithmetic { operator, left, right, failure, more } => {
                let left_value = self.expression(frame, left)?;
                let right_value = self.expression(frame, right)?;
                self.arithmetic(
                    frame,
                    *operator,
                    left_value,
                    right_value,
                    &expression.machine_type,
                    failure.as_deref(),
                    more,
                )?
            }
            ExpressionKind::Negate { operand, failure } => {
                let operand = self.expression(frame, operand)?;
                if let MachineType::Integer { bits, signed: true } = expression.machine_type
                    && let Some(failure) = failure
                {
                    let lowest = integer_constant(self.context, bits, -(1i128 << (bits - 1)));
                    let fine =
                        frame.builder.integer_compare(IntegerPredicate::NotEqual, operand, lowest, "").to_dynamic();
                    self.check(frame, fine, failure, &[(operand, expression.machine_type.clone())])?;
                }
                match expression.machine_type {
                    MachineType::Float { .. } => frame.builder.float_negate(operand, "").to_dynamic(),
                    _ => frame.builder.integer_negate(operand, "").to_dynamic(),
                }
            }
            ExpressionKind::Compare { operator, left, right } => {
                let signed = matches!(left.machine_type, MachineType::Integer { signed: true, .. });
                let left = self.expression(frame, left)?;
                let right = self.expression(frame, right)?;
                let predicate = match (operator, signed) {
                    (Comparison::Equal, _) => IntegerPredicate::Equal,
                    (Comparison::NotEqual, _) => IntegerPredicate::NotEqual,
                    (Comparison::Below, true) => IntegerPredicate::SignedLessThan,
                    (Comparison::Below, false) => IntegerPredicate::UnsignedLessThan,
                    (Comparison::Above, true) => IntegerPredicate::SignedGreaterThan,
                    (Comparison::Above, false) => IntegerPredicate::UnsignedGreaterThan,
                    (Comparison::AtLeast, true) => IntegerPredicate::SignedGreaterOrEqual,
                    (Comparison::AtLeast, false) => IntegerPredicate::UnsignedGreaterOrEqual,
                    (Comparison::AtMost, true) => IntegerPredicate::SignedLessOrEqual,
                    (Comparison::AtMost, false) => IntegerPredicate::UnsignedLessOrEqual,
                };
                frame.builder.integer_compare(predicate, left, right, "").to_dynamic()
            }
            ExpressionKind::Not(operand) | ExpressionKind::BitNot(operand) => {
                let operand = self.expression(frame, operand)?;
                frame.builder.not(operand, "").to_dynamic()
            }
            ExpressionKind::And(left, right) | ExpressionKind::Or(left, right) => {
                let is_and = matches!(expression.kind, ExpressionKind::And(..));
                let left = self.expression(frame, left)?;
                let start = frame.builder.insertion_basic_block();
                let second = frame.function.append_basic_block("second");
                let after = frame.function.append_basic_block("after_logic");
                if is_and {
                    frame.builder.conditional_branch(left, second, after);
                } else {
                    frame.builder.conditional_branch(left, after, second);
                }
                frame.builder.position_at_end(second);
                let right = self.expression(frame, right)?;
                let second_end = frame.builder.insertion_basic_block();
                frame.builder.branch(after);
                frame.builder.position_at_end(after);
                let bool_type = self.context.integer_type(1);
                let merged = frame.builder.phi(bool_type, "");
                let short = bool_type.constant_integer(u64::from(!is_and), false).to_dynamic();
                let short = short.checked_cast::<lyftar::Integer>();
                let right = right.checked_cast::<lyftar::Integer>();
                merged.add_incoming(&[(short, start), (right, second_end)]);
                merged.to_dynamic()
            }
            ExpressionKind::Convert(operand) => {
                let MachineType::Integer { bits: from, signed } = operand.machine_type else {
                    return Err("converting something that isn't an integer".to_string());
                };
                let MachineType::Integer { bits: to, .. } = expression.machine_type else {
                    return Err("converting into something that isn't an integer".to_string());
                };
                let operand = self.expression(frame, operand)?;
                let target = self.context.integer_type(to);
                match to.cmp(&from) {
                    std::cmp::Ordering::Less => frame.builder.truncate(operand, target, "").to_dynamic(),
                    std::cmp::Ordering::Greater if signed => {
                        frame.builder.sign_extend(operand, target, "").to_dynamic()
                    }
                    std::cmp::Ordering::Greater => frame.builder.zero_extend(operand, target, "").to_dynamic(),
                    std::cmp::Ordering::Equal => operand,
                }
            }
            ExpressionKind::Call { function, arguments } => {
                let callee = *self.functions.get(*function).ok_or("a call to a missing function")?;
                let mut values = Vec::with_capacity(arguments.len());
                for argument in arguments {
                    values.push(match argument {
                        Argument::Value(value) => self.expression(frame, value)?,
                        Argument::Address(local) => frame.slots[*local],
                    });
                }
                frame.builder.call(callee, &values, "").to_dynamic()
            }
            ExpressionKind::Fit { value, min, max, failure } => {
                let MachineType::Integer { bits, signed } = value.machine_type else {
                    return Err("a bound on something that isn't an integer".to_string());
                };
                let generated = self.expression(frame, value)?;
                let (low, high) = if signed {
                    (IntegerPredicate::SignedGreaterOrEqual, IntegerPredicate::SignedLessOrEqual)
                } else {
                    (IntegerPredicate::UnsignedGreaterOrEqual, IntegerPredicate::UnsignedLessOrEqual)
                };
                let above_min = frame
                    .builder
                    .integer_compare(low, generated, integer_constant(self.context, bits, *min), "")
                    .to_dynamic();
                let below_max = frame
                    .builder
                    .integer_compare(high, generated, integer_constant(self.context, bits, *max), "")
                    .to_dynamic();
                let fits = frame.builder.and(above_min, below_max, "").to_dynamic();
                self.check(frame, fits, failure, &[(generated, value.machine_type.clone())])?;
                generated
            }
            ExpressionKind::Bind { local, value, body } => {
                let generated = self.expression(frame, value)?;
                frame.builder.store(generated, frame.slots[*local]);
                self.expression(frame, body)?
            }
        };
        Ok(value)
    }

    /// Integer arithmetic, with its checks (standard 10.4).
    #[allow(clippy::too_many_arguments, clippy::too_many_lines)]
    fn arithmetic(
        &mut self,
        frame: &mut Frame<'context, 'module>,
        operator: Arithmetic,
        left: Value<'context, 'module>,
        right: Value<'context, 'module>,
        machine_type: &MachineType,
        failure: Option<&CheckFailure>,
        more: &[CheckFailure],
    ) -> Result<Value<'context, 'module>> {
        let (bits, signed) = match *machine_type {
            MachineType::Integer { bits, signed } => (bits, signed),
            MachineType::Bool => (1, false),
            _ => return Err("float arithmetic isn't generated yet".to_string()),
        };
        let operands = [(left, machine_type.clone()), (right, machine_type.clone())];
        let zero = integer_constant(self.context, bits, 0);
        if let (Some(failure), Arithmetic::Add | Arithmetic::Subtract | Arithmetic::Multiply) = (failure, operator) {
            let name = match (operator, signed) {
                (Arithmetic::Add, true) => "llvm.sadd.with.overflow",
                (Arithmetic::Add, false) => "llvm.uadd.with.overflow",
                (Arithmetic::Subtract, true) => "llvm.ssub.with.overflow",
                (Arithmetic::Subtract, false) => "llvm.usub.with.overflow",
                (_, true) => "llvm.smul.with.overflow",
                (_, false) => "llvm.umul.with.overflow",
            };
            let intrinsic = Intrinsic::find(name).ok_or_else(|| format!("LLVM has no `{name}`"))?;
            let declaration = self.module.intrinsic_declaration(intrinsic, &[left.get_type()]);
            let pair = frame.builder.call(declaration, &[left, right], "").to_dynamic();
            let result = frame.builder.extract_value(pair, 0, "").to_dynamic();
            let overflowed = frame.builder.extract_value(pair, 1, "").to_dynamic();
            let fine = frame.builder.not(overflowed, "").to_dynamic();
            self.check(frame, fine, failure, &operands)?;
            return Ok(result);
        }
        match operator {
            Arithmetic::DivideTruncate
            | Arithmetic::DivideFloor
            | Arithmetic::DivideExact
            | Arithmetic::RemainderTruncate
            | Arithmetic::RemainderFloor => {
                if let Some(failure) = failure {
                    let fine = frame.builder.integer_compare(IntegerPredicate::NotEqual, right, zero, "").to_dynamic();
                    self.check(frame, fine, failure, &operands)?;
                }
                // The lowest value divided by -1 doesn't fit, and isn't decided (standard 12.1), so it's never
                // left for LLVM, where it would be undefined.
                if signed && let Some(lowest_failure) = more.last() {
                    let lowest = integer_constant(self.context, bits, -(1i128 << (bits - 1)));
                    let minus_one = integer_constant(self.context, bits, -1);
                    let is_lowest =
                        frame.builder.integer_compare(IntegerPredicate::Equal, left, lowest, "").to_dynamic();
                    let is_minus_one =
                        frame.builder.integer_compare(IntegerPredicate::Equal, right, minus_one, "").to_dynamic();
                    let both = frame.builder.and(is_lowest, is_minus_one, "").to_dynamic();
                    let fine = frame.builder.not(both, "").to_dynamic();
                    self.check(frame, fine, lowest_failure, &operands)?;
                }
                let quotient = if signed {
                    frame.builder.signed_divide(left, right, "").to_dynamic()
                } else {
                    frame.builder.unsigned_divide(left, right, "").to_dynamic()
                };
                let remainder = if signed {
                    frame.builder.signed_remainder(left, right, "").to_dynamic()
                } else {
                    frame.builder.unsigned_remainder(left, right, "").to_dynamic()
                };
                // Flooring differs from truncating when the remainder isn't 0 and its sign differs from the divisor's.
                let needs_floor = || -> Value<'context, 'module> {
                    let nonzero =
                        frame.builder.integer_compare(IntegerPredicate::NotEqual, remainder, zero, "").to_dynamic();
                    let mixed = frame.builder.exclusive_or(remainder, right, "").to_dynamic();
                    let signs_differ =
                        frame.builder.integer_compare(IntegerPredicate::SignedLessThan, mixed, zero, "").to_dynamic();
                    frame.builder.and(nonzero, signs_differ, "").to_dynamic()
                };
                Ok(match operator {
                    Arithmetic::DivideTruncate => quotient,
                    Arithmetic::RemainderTruncate => remainder,
                    Arithmetic::DivideExact => {
                        if let Some(inexact) = more.first() {
                            let exact = frame
                                .builder
                                .integer_compare(IntegerPredicate::Equal, remainder, zero, "")
                                .to_dynamic();
                            self.check(frame, exact, inexact, &operands)?;
                        }
                        quotient
                    }
                    Arithmetic::DivideFloor if signed => {
                        let adjust = needs_floor();
                        let one = integer_constant(self.context, bits, 1);
                        let lower = frame.builder.integer_subtract(quotient, one, "").to_dynamic();
                        frame.builder.select(adjust, lower, quotient, "").to_dynamic()
                    }
                    Arithmetic::RemainderFloor if signed => {
                        let adjust = needs_floor();
                        let moved = frame.builder.integer_add(remainder, right, "").to_dynamic();
                        frame.builder.select(adjust, moved, remainder, "").to_dynamic()
                    }
                    Arithmetic::DivideFloor => quotient,
                    _ => remainder,
                })
            }
            Arithmetic::ShiftLeft | Arithmetic::ShiftRight => {
                let width = integer_constant(self.context, bits, i128::from(bits));
                if let Some(too_far) = more.first() {
                    let within = frame
                        .builder
                        .integer_compare(IntegerPredicate::UnsignedLessThan, right, width, "")
                        .to_dynamic();
                    self.check(frame, within, too_far, &operands)?;
                }
                if operator == Arithmetic::ShiftLeft {
                    let shifted = frame.builder.shift_left(left, right, "").to_dynamic();
                    if let Some(failure) = failure {
                        // It overflowed when shifting back doesn't give the value again.
                        let back = if signed {
                            frame.builder.arithmetic_shift_right(shifted, right, "").to_dynamic()
                        } else {
                            frame.builder.logical_shift_right(shifted, right, "").to_dynamic()
                        };
                        let same = frame.builder.integer_compare(IntegerPredicate::Equal, back, left, "").to_dynamic();
                        let fine = if signed {
                            let before = frame
                                .builder
                                .integer_compare(IntegerPredicate::SignedLessThan, left, zero, "")
                                .to_dynamic();
                            let after = frame
                                .builder
                                .integer_compare(IntegerPredicate::SignedLessThan, shifted, zero, "")
                                .to_dynamic();
                            let kept =
                                frame.builder.integer_compare(IntegerPredicate::Equal, before, after, "").to_dynamic();
                            frame.builder.and(same, kept, "").to_dynamic()
                        } else {
                            same
                        };
                        self.check(frame, fine, failure, &operands)?;
                    }
                    return Ok(shifted);
                }
                if signed && let Some(failure) = failure {
                    let fine = frame
                        .builder
                        .integer_compare(IntegerPredicate::SignedGreaterOrEqual, left, zero, "")
                        .to_dynamic();
                    self.check(frame, fine, failure, &operands)?;
                }
                Ok(frame.builder.logical_shift_right(left, right, "").to_dynamic())
            }
            Arithmetic::Add | Arithmetic::WrappingAdd => Ok(frame.builder.integer_add(left, right, "").to_dynamic()),
            Arithmetic::Subtract => Ok(frame.builder.integer_subtract(left, right, "").to_dynamic()),
            Arithmetic::Multiply => Ok(frame.builder.integer_multiply(left, right, "").to_dynamic()),
            Arithmetic::BitAnd => Ok(frame.builder.and(left, right, "").to_dynamic()),
            Arithmetic::BitOr => Ok(frame.builder.or(left, right, "").to_dynamic()),
            Arithmetic::BitXor => Ok(frame.builder.exclusive_or(left, right, "").to_dynamic()),
        }
    }
}

/// An integer constant of any width up to 128 bits, in two's complement.
fn integer_constant<'context>(
    context: &lyftar::context::Context<'context>,
    bits: u32,
    value: i128,
) -> Value<'context, 'static> {
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let words = [value as u64, (value >> 64) as u64];
    let words = if bits > 64 { &words[..] } else { &words[..1] };
    context.integer_type(bits).constant_integer_from_words(words).to_dynamic()
}
