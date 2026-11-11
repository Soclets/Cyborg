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
pub fn object_file(program: &Program, target: Target, mode: Mode) -> Result<Vec<u8>, String> {
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

impl<'context, 'module> Generator<'context, 'module> {
    fn runtime(&self, name: &str) -> Result<Function<'context, 'module>, String> {
        self.module.get_function(name).ok_or_else(|| format!("the runtime has no `{name}`"))
    }

    fn machine_type(&self, machine_type: &MachineType) -> lyftar::types::Type<'context> {
        match machine_type {
            MachineType::Integer { bits, .. } => self.context.integer_type(*bits).to_dynamic(),
            MachineType::Float { bits: 32 } => self.context.get_type::<F32>().to_dynamic(),
            MachineType::Float { .. } => self.context.get_type::<F64>().to_dynamic(),
            MachineType::Bool => self.context.integer_type(1).to_dynamic(),
            MachineType::Void => self.context.void_type(),
            MachineType::Writer => {
                self.context.get_named_struct("writer").unwrap_or_else(|| self.context.integer_type(8).to_dynamic())
            }
        }
    }

    fn program(&mut self, program: &Program) -> Result<(), String> {
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
            self.function(function, index == program.entry)?;
        }
        Ok(())
    }

    fn function(&mut self, function: &crate::checked::Function, is_main: bool) -> Result<(), String> {
        let declared = self
            .module
            .get_function(&function.symbol)
            .ok_or_else(|| format!("`{}` wasn't declared", function.symbol))?;
        let entry = declared.append_basic_block("entry");
        let builder = Builder::at_end_of(entry);
        let mut slots = Vec::with_capacity(function.locals.len());
        for local in &function.locals {
            let slot = builder.stack_allocate(self.machine_type(local), "").to_dynamic();
            slots.push(slot);
        }
        for (index, parameter) in function.parameters.iter().enumerate() {
            let incoming = declared.parameter(u32::try_from(index).unwrap_or(u32::MAX));
            match parameter.passing {
                Passing::Value => {
                    builder.store(incoming, slots[parameter.local]);
                }
                Passing::Address => slots[parameter.local] = incoming,
            }
        }
        let mut frame = Frame { function: declared, slots, loops: HashMap::new(), builder, is_main };
        self.block(&mut frame, &function.body)?;
        // A function that runs off its end, which the checker allows only for `void`, returns here.
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
    fn ensure_open(&self, frame: &mut Frame<'context, 'module>) {
        if frame.builder.insertion_basic_block().terminator().is_some() {
            let dead = frame.function.append_basic_block("after");
            frame.builder.position_at_end(dead);
        }
    }

    fn block(&mut self, frame: &mut Frame<'context, 'module>, block: &Block) -> Result<(), String> {
        for statement in block {
            self.ensure_open(frame);
            self.statement(frame, statement)?;
        }
        Ok(())
    }

    fn statement(&mut self, frame: &mut Frame<'context, 'module>, statement: &Statement) -> Result<(), String> {
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
                self.branch_if_open(frame, after);
                frame.builder.position_at_end(otherwise_block);
                self.block(frame, otherwise)?;
                self.branch_if_open(frame, after);
                frame.builder.position_at_end(after);
            }
            Statement::Loop { id, body } => {
                let top = frame.function.append_basic_block("loop");
                let after = frame.function.append_basic_block("after_loop");
                frame.loops.insert(*id, after);
                frame.builder.branch(top);
                frame.builder.position_at_end(top);
                self.block(frame, body)?;
                self.branch_if_open(frame, top);
                frame.builder.position_at_end(after);
            }
            Statement::Break(id) => {
                let after = *frame.loops.get(id).ok_or("a break outside its loop")?;
                frame.builder.branch(after);
            }
            Statement::Return(value) => {
                let is_main = frame.is_main;
                match value {
                    Some(value) => {
                        let value = self.expression(frame, value)?;
                        frame.builder.return_value(value);
                    }
                    None if is_main => {
                        frame.builder.return_value(self.context.integer_type(32).constant_integer(0, false));
                    }
                    None => {
                        frame.builder.return_void();
                    }
                }
            }
            Statement::OpenWriter { local, stream, capacity } => {
                let open = self.runtime("__cyborg_writer_open")?;
                let descriptor = self
                    .context
                    .integer_type(32)
                    .constant_integer(u64::from(stream.descriptor().unsigned_abs()), false)
                    .to_dynamic();
                let capacity = match capacity {
                    Some(capacity) => self.expression(frame, capacity)?,
                    None => self.context.integer_type(64).constant_integer(0, false).to_dynamic(),
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
            Statement::Check { condition, failure } => {
                if self.mode.keeps_checks() {
                    let condition = self.expression(frame, condition)?;
                    self.check(frame, condition, failure)?;
                }
            }
        }
        Ok(())
    }

    fn branch_if_open(&self, frame: &mut Frame<'context, 'module>, to: BasicBlock<'context, 'module>) {
        if frame.builder.insertion_basic_block().terminator().is_none() {
            frame.builder.branch(to);
        }
    }

    /// Crashes with `failure`'s message unless `holds` is true.
    fn check(
        &mut self,
        frame: &mut Frame<'context, 'module>,
        holds: Value<'context, 'module>,
        failure: &CheckFailure,
    ) -> Result<(), String> {
        let fine = frame.function.append_basic_block("fine");
        let failed = frame.function.append_basic_block("failed");
        frame.builder.conditional_branch(holds, fine, failed);
        frame.builder.position_at_end(failed);
        if self.bare_metal {
            frame.builder.call(self.runtime("__cyborg_trap")?, &[] as &[Value<'context, 'module>], "");
            frame.builder.unreachable();
            frame.builder.position_at_end(fine);
            return Ok(());
        }
        let writer_type = self.machine_type(&MachineType::Writer);
        let writer = frame.builder.stack_allocate(writer_type, "").to_dynamic();
        frame.builder.call(self.runtime("__cyborg_crash_begin")?, &[writer], "");
        self.text(frame, writer, &failure.before)?;
        for (text, value) in &failure.when {
            self.text(frame, writer, text)?;
            self.value(frame, writer, value, &Shape::default())?;
        }
        self.text(frame, writer, &failure.after)?;
        let exit = self.runtime("__cyborg_exit")?;
        let one = self.context.integer_type(32).constant_integer(1, false).to_dynamic();
        frame.builder.call(exit, &[one], "");
        frame.builder.unreachable();
        frame.builder.position_at_end(fine);
        Ok(())
    }

    fn pieces(
        &mut self,
        frame: &mut Frame<'context, 'module>,
        writer: Value<'context, 'module>,
        pieces: &[Piece],
    ) -> Result<(), String> {
        for piece in pieces {
            match piece {
                Piece::Text(text) => self.text(frame, writer, text)?,
                Piece::Value { value, shape } => self.value(frame, writer, value, shape)?,
            }
        }
        Ok(())
    }

    /// Writes constant text.
    fn text(
        &mut self,
        frame: &mut Frame<'context, 'module>,
        writer: Value<'context, 'module>,
        text: &str,
    ) -> Result<(), String> {
        if text.is_empty() {
            return Ok(());
        }
        let constant = self.context.constant_string(text.as_bytes(), false);
        let global = self.module.add_global("", constant.get_type());
        global.set_initializer(constant);
        global.set_global_constant(true);
        global.set_linkage(Linkage::Private);
        global.set_unnamed_address(UnnamedAddress::Global);
        let length = self.context.integer_type(64).constant_integer(text.len() as u64, false).to_dynamic();
        frame.builder.call(self.runtime("__cyborg_writer_write")?, &[writer, global.to_dynamic(), length], "");
        Ok(())
    }

    /// Prints a value with its shape.
    fn value(
        &mut self,
        frame: &mut Frame<'context, 'module>,
        writer: Value<'context, 'module>,
        value: &Expression,
        shape: &Shape,
    ) -> Result<(), String> {
        match value.machine_type {
            MachineType::Integer { bits, signed } => {
                let generated = self.expression(frame, value)?;
                let i64_type = self.context.integer_type(64);
                let wide = if bits == 64 {
                    generated
                } else if signed {
                    frame.builder.sign_extend(generated, i64_type, "").to_dynamic()
                } else {
                    frame.builder.zero_extend(generated, i64_type, "").to_dynamic()
                };
                let zero = i64_type.constant_integer(0, false).to_dynamic();
                let (magnitude, negative) = if signed {
                    let negative =
                        frame.builder.integer_compare(IntegerPredicate::SignedLessThan, wide, zero, "").to_dynamic();
                    let flipped = frame.builder.integer_subtract(zero, wide, "").to_dynamic();
                    (frame.builder.select(negative, flipped, wide, "").to_dynamic(), negative)
                } else {
                    (wide, self.context.integer_type(1).constant_integer(0, false).to_dynamic())
                };
                let (base, upper) = match shape.base {
                    Base::Decimal => (10, false),
                    Base::Binary => (2, false),
                    Base::Octal => (8, false),
                    Base::Hex => (16, false),
                    Base::HexUpper => (16, true),
                };
                let bit = |on: bool| self.context.integer_type(1).constant_integer(u64::from(on), false).to_dynamic();
                let align = match shape.align {
                    Some(crate::checked::Align::Left) => 0,
                    Some(crate::checked::Align::Center) => 2,
                    _ => 1,
                };
                let arguments = [
                    writer,
                    magnitude,
                    negative,
                    i64_type.constant_integer(base, false).to_dynamic(),
                    bit(upper),
                    i64_type.constant_integer(shape.width.unwrap_or(0), false).to_dynamic(),
                    self.context.integer_type(32).constant_integer(align, false).to_dynamic(),
                    bit(shape.zeros),
                    bit(shape.sign),
                ];
                frame.builder.call(self.runtime("__cyborg_print_integer")?, &arguments, "");
                Ok(())
            }
            _ => Err("printing this type isn't generated yet".to_string()),
        }
    }

    fn expression(
        &mut self,
        frame: &mut Frame<'context, 'module>,
        expression: &Expression,
    ) -> Result<Value<'context, 'module>, String> {
        let builder = &frame.builder;
        let value = match &expression.kind {
            ExpressionKind::Integer(value) => {
                let MachineType::Integer { bits, .. } = expression.machine_type else {
                    return Err("an integer that isn't one".to_string());
                };
                // Two's complement in 64 bits, then cut to the width, which keeps negative numbers right.
                #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                let raw = *value as i64 as u64;
                self.context.integer_type(bits).constant_integer(raw, true).to_dynamic()
            }
            ExpressionKind::Float(value) => self.context.get_type::<F64>().constant_float(*value).to_dynamic(),
            ExpressionKind::Bool(value) => {
                self.context.integer_type(1).constant_integer(u64::from(*value), false).to_dynamic()
            }
            ExpressionKind::Local(local) => {
                let loaded_type = self.machine_type(&expression.machine_type);
                builder.load(loaded_type, frame.slots[*local], "").to_dynamic()
            }
            ExpressionKind::Arithmetic { operator, left, right, failure } => {
                let left = self.expression(frame, left)?;
                let right = self.expression(frame, right)?;
                self.arithmetic(frame, *operator, left, right, &expression.machine_type, failure.as_deref())?
            }
            ExpressionKind::Negate { operand, failure } => {
                let operand = self.expression(frame, operand)?;
                let zero = self.zero(&expression.machine_type);
                self.arithmetic(
                    frame,
                    Arithmetic::Subtract,
                    zero,
                    operand,
                    &expression.machine_type,
                    failure.as_deref(),
                )?
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
                if to < from {
                    frame.builder.truncate(operand, target, "").to_dynamic()
                } else if to > from && signed {
                    frame.builder.sign_extend(operand, target, "").to_dynamic()
                } else if to > from {
                    frame.builder.zero_extend(operand, target, "").to_dynamic()
                } else {
                    operand
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
        };
        Ok(value)
    }

    fn zero(&self, machine_type: &MachineType) -> Value<'context, 'module> {
        match machine_type {
            MachineType::Integer { bits, .. } => {
                self.context.integer_type(*bits).constant_integer(0, false).to_dynamic()
            }
            _ => self.context.get_type::<F64>().constant_float(0.0).to_dynamic(),
        }
    }

    /// Integer arithmetic, trapping on overflow and division by zero where the mode keeps checks (standard 10.4).
    fn arithmetic(
        &mut self,
        frame: &mut Frame<'context, 'module>,
        operator: Arithmetic,
        left: Value<'context, 'module>,
        right: Value<'context, 'module>,
        machine_type: &MachineType,
        failure: Option<&CheckFailure>,
    ) -> Result<Value<'context, 'module>, String> {
        let MachineType::Integer { signed, .. } = *machine_type else {
            return Err("float arithmetic isn't generated yet".to_string());
        };
        let builder = &frame.builder;
        let checked = failure.filter(|_| self.mode.keeps_checks());
        if let (Some(failure), Arithmetic::Add | Arithmetic::Subtract | Arithmetic::Multiply) = (checked, operator) {
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
            let pair = builder.call(declaration, &[left, right], "").to_dynamic();
            let result = builder.extract_value(pair, 0, "").to_dynamic();
            let overflowed = builder.extract_value(pair, 1, "").to_dynamic();
            let fine = builder.not(overflowed, "").to_dynamic();
            self.check(frame, fine, failure)?;
            return Ok(result);
        }
        let builder = &frame.builder;
        Ok(match operator {
            Arithmetic::Add | Arithmetic::WrappingAdd => builder.integer_add(left, right, "").to_dynamic(),
            Arithmetic::Subtract => builder.integer_subtract(left, right, "").to_dynamic(),
            Arithmetic::Multiply => builder.integer_multiply(left, right, "").to_dynamic(),
            Arithmetic::BitAnd => builder.and(left, right, "").to_dynamic(),
            Arithmetic::BitOr => builder.or(left, right, "").to_dynamic(),
            Arithmetic::BitXor => builder.exclusive_or(left, right, "").to_dynamic(),
            Arithmetic::ShiftLeft => builder.shift_left(left, right, "").to_dynamic(),
            Arithmetic::ShiftRight if signed => builder.arithmetic_shift_right(left, right, "").to_dynamic(),
            Arithmetic::ShiftRight => builder.logical_shift_right(left, right, "").to_dynamic(),
            Arithmetic::DivideTruncate
            | Arithmetic::DivideFloor
            | Arithmetic::RemainderTruncate
            | Arithmetic::RemainderFloor => {
                return Err("division isn't generated yet".to_string());
            }
        })
    }
}
