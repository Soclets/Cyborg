//! Machine code, through Lyftar (rule 1).
//!
//! A program is built in units (D168): the runtime, then its functions. Each unit reads the runtime's small functions
//! first, so the program's code calls straight into them and LLVM's passes can inline them, while their one real copy
//! is in the runtime's own unit. Every local gets a stack slot in the function's entry block, which LLVM turns into registers.

use std::collections::HashMap;

use lyftar::enums::{IntegerPredicate, Linkage, UnnamedAddress};
use lyftar::intrinsics::Intrinsic;
use lyftar::passes::PassOptions;
use lyftar::target::{
    CodeGenerationFileType, CodeGenerationOptimizationLevel, CodeModel, RelocationMode, Target as LlvmTarget,
};
use lyftar::{Builder, with_context};

use crate::checked::{
    Argument, Arithmetic, AtomicOperation, AtomicOrder, Base, Block, CheckFailure, Comparison, Expression,
    ExpressionKind, MachineType, Passing, Piece, Program, Shape, Statement,
};
use crate::target::{Checks, Mode, Overflow, Target};

const OS_RUNTIME: &str = include_str!("runtime/os.ll");
const BARE_RUNTIME: &str = include_str!("runtime/bare.ll");
const FLOAT_RUNTIME: &str = include_str!("runtime/floats.ll");
const SYSTEM_MACOS: &str = include_str!("runtime/system_macos.ll");
const SYSTEM_LINUX: &str = include_str!("runtime/system_linux.ll");
const PROBE_LINUX_AARCH64: &str = include_str!("runtime/probe_linux_aarch64.ll");
const PROBE_LINUX_X86_64: &str = include_str!("runtime/probe_linux_x86_64.ll");
const BOARD_STARTUP: &str = include_str!("runtime/arduino_uno_r4_wifi/startup.ll");

/// The linker script for the Arduino Uno R4 `WiFi` (D261, D281).
pub const BOARD_MEMORY: &str = include_str!("runtime/arduino_uno_r4_wifi/memory.ld");

/// A unit of a build, compiled on its own, in parallel with the others, and cached by its IR (D168).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unit {
    /// The runtime, which is the same for every program on a target and mode, so it's built once and cached.
    Runtime,
    /// Some of the program's functions, by index.
    Functions(Vec<usize>),
}

/// The most units a program's functions are split into, so linking stays quick for big programs.
const MOST_UNITS: usize = 64;

/// How a program is split. `Working` and `Literal`, which don't inline across functions, give each function its own
/// unit, so changing one rebuilds only that one. The `Final` modes keep the program's functions together, so LLVM can
/// inline across all of them.
#[must_use]
pub fn plan(program: &Program, mode: Mode) -> Vec<Unit> {
    // A generic's own body is never built, only its copies (D235).
    let built: Vec<usize> = program
        .functions
        .iter()
        .enumerate()
        .filter(|(_, function)| !function.template)
        .map(|(index, _)| index)
        .collect();
    let count = built.len();
    let mut units = vec![Unit::Runtime];
    if !matches!(mode, Mode::Working(_) | Mode::Literal) || count <= 1 {
        units.push(Unit::Functions(built));
        return units;
    }
    if count <= MOST_UNITS {
        units.extend(built.into_iter().map(|index| Unit::Functions(vec![index])));
        return units;
    }
    // A function's unit comes from its name, not its position, so adding one elsewhere doesn't move it.
    let names = symbols(program);
    let mut buckets = vec![Vec::new(); MOST_UNITS];
    for index in built {
        let Some(name) = names.get(index) else { continue };
        let hashed = name.bytes().fold(0usize, |sum, byte| sum.wrapping_mul(31).wrapping_add(usize::from(byte)));
        buckets[hashed % MOST_UNITS].push(index);
    }
    units.extend(buckets.into_iter().filter(|bucket| !bucket.is_empty()).map(Unit::Functions));
    units
}

/// Each function's symbol, made unique by its position where two would share one.
fn symbols(program: &Program) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    program
        .functions
        .iter()
        .enumerate()
        .map(|(index, function)| {
            if seen.insert(function.symbol.clone()) {
                function.symbol.clone()
            } else {
                format!("{}.{index}", function.symbol)
            }
        })
        .collect()
}

/// The runtime's IR, as its own unit: the operating system's, with float printing, or the board's, with its
/// startup code.
fn runtime_text(target: Target) -> String {
    if target.has_os() {
        format!("{OS_RUNTIME}\n{}\n{FLOAT_RUNTIME}", system_runtime(target))
    } else {
        format!("{BARE_RUNTIME}\n{BOARD_STARTUP}")
    }
}

/// The part of the runtime each operating system does its own way: how a lock waits (D348), and how a tuned program
/// asks for its instructions (D365).
fn system_runtime(target: Target) -> String {
    match target {
        Target::MacOsAarch64 => SYSTEM_MACOS.to_string(),
        // The futex system call's number on each processor.
        Target::LinuxAarch64 => format!("{}\n{PROBE_LINUX_AARCH64}", SYSTEM_LINUX.replace("FUTEX", "98")),
        Target::LinuxX86_64 => format!("{}\n{PROBE_LINUX_X86_64}", SYSTEM_LINUX.replace("FUTEX", "202")),
        Target::ArduinoUnoR4Wifi => String::new(),
    }
}

/// The runtime as each unit of the program sees it: its small functions' bodies, so LLVM can inline them, marked as
/// defined elsewhere, in the runtime's own unit, with float printing only declared, since it's too big to inline.
fn inline_runtime_text(target: Target) -> String {
    if target.has_os() {
        let shared = OS_RUNTIME
            .replace(WRITERS_DEFINED, WRITERS_DECLARED)
            .replace(HEAP_DEFINED, HEAP_DECLARED)
            .replace(NEXT_DEFINED, NEXT_DECLARED)
            .replace(RUNTIME_LOCK_DEFINED, RUNTIME_LOCK_DECLARED);
        let system = system_runtime(target);
        format!("{shared}\n{system}\ndeclare i64 @__cyborg_format_float(double, i32, i32, i32, i1, ptr)\n")
    } else {
        BARE_RUNTIME.to_string()
    }
}

/// The list of buffered writers, which the runtime's unit defines and every other unit shares.
const WRITERS_DEFINED: &str = "@__cyborg_writers = hidden global ptr null";
const WRITERS_DECLARED: &str = "@__cyborg_writers = external hidden global ptr";
/// The OS heap's record, and the next arena's number, which the runtime's unit defines too (D322, D371).
const HEAP_DEFINED: &str = "@__cyborg_the_heap = hidden global %allocator { i64 0, i64 -1, i64 0, i64 -1, i64 0 }";
const HEAP_DECLARED: &str = "@__cyborg_the_heap = external hidden global %allocator";
const NEXT_DEFINED: &str = "@__cyborg_next_allocator = hidden global i64 1";
const NEXT_DECLARED: &str = "@__cyborg_next_allocator = external hidden global i64";
/// The runtime's own lock, which the runtime's unit defines too (D345).
const RUNTIME_LOCK_DEFINED: &str = "@__cyborg_runtime_lock = hidden global i32 0";
const RUNTIME_LOCK_DECLARED: &str = "@__cyborg_runtime_lock = external hidden global i32";

/// The unoptimized IR for one unit, as text. It's all a unit's object depends on, besides the target and mode, so
/// it's what the cache hashes.
///
/// # Errors
/// What LLVM reported, which is a bug in this compiler, since a checked program is always valid.
pub fn unit_ir(program: &Program, target: Target, mode: Mode, unit: &Unit) -> Result<(String, Vec<Messages>)> {
    let Unit::Functions(chosen) = unit else { return Ok((runtime_text(target), Vec::new())) };
    with_context(|context| {
        let module = context
            .parse_ir_text(&inline_runtime_text(target))
            .map_err(|error| format!("the runtime: {}", error.message()))?;
        let view = context.module(&module);
        let mut function = view.last_function();
        while let Some(each) = function {
            if !each.is_declaration() {
                each.set_linkage(Linkage::AvailableExternally);
            }
            function = each.previous_function();
        }
        let mut defined = vec![false; program.functions.len()];
        for index in chosen {
            if let Some(slot) = defined.get_mut(*index) {
                *slot = true;
            }
        }
        let mut generator = Generator {
            context,
            module: view,
            mode,
            bare_metal: !target.has_os(),
            target,
            // `sysconf`'s own number for asking how many cores are online, `_SC_NPROCESSORS_ONLN`.
            cores_name: if target == Target::MacOsAarch64 { 58 } else { 84 },
            program,
            symbols: symbols(program),
            current: Messages::default(),
            messages: Vec::new(),
            whole: chosen.len() == program.functions.iter().filter(|function| !function.template).count(),
            functions: vec![None; program.functions.len()],
            structs: Vec::new(),
            struct_fields: Vec::new(),
            union_fields: Vec::new(),
            unions: Vec::new(),
        };
        generator.aggregates(program);
        for index in chosen {
            generator.declare(*index)?;
        }
        for index in chosen {
            let function = program.functions.get(*index).ok_or("a unit names a missing function")?;
            generator.function(*index, function, *index == program.entry)?;
        }
        for name in mode.size_attributes() {
            let kind = lyftar::attributes::AttributeKind::for_name(name)
                .ok_or_else(|| format!("LLVM has no `{name}` attribute"))?;
            let attribute = context.enum_attribute(kind, 0);
            let mut function = view.last_function();
            while let Some(each) = function {
                if !each.is_declaration() {
                    each.add_attribute(lyftar::attributes::AttributeLocation::Function, attribute);
                }
                function = each.previous_function();
            }
        }
        view.verify().map_err(|error| format!("the generated code: {}", error.message()))?;
        let mut messages = std::mem::take(&mut generator.messages);
        messages.push(std::mem::take(&mut generator.current));
        messages.retain(|each| !each.texts.is_empty() || !each.sites.is_empty());
        Ok((view.print_to_string(), messages))
    })
}

/// One function's crash message texts and sites, in the order its code reads them.
#[derive(Clone, Debug, Default)]
pub struct Messages {
    pub symbol: String,
    pub texts: Vec<String>,
    pub sites: Vec<String>,
}

/// The messages unit's object: each function's tables of crash message texts and sites, which show lines and
/// columns, so it's the one unit that changes when lines move (D168). It's only data, so it's made straight through
/// Lyftar, with no IR text to read and nothing to optimize.
///
/// # Errors
/// What LLVM reported, which is a bug in this compiler.
pub fn messages_object(messages: &[Messages], target: Target, mode: Mode) -> Result<Vec<u8>> {
    let machine = machine(target, mode)?;
    with_context(|context| {
        let mut module = context.create_module("messages");
        {
            let view = context.module(&module);
            view.set_target_machine(&machine);
            let pointer = context.pointer_type(0);
            let word = context.integer_type(64);
            let text_type = context.struct_type(&[pointer, word.to_dynamic()], false);
            let table = |name: String, texts: &[String]| {
                if texts.is_empty() {
                    return;
                }
                let joined: String = texts.concat();
                let bytes = context.constant_string(joined.as_bytes(), false);
                let blob = view.add_global("", bytes.get_type());
                blob.set_initializer(bytes);
                blob.set_global_constant(true);
                blob.set_linkage(Linkage::Private);
                let mut entries = Vec::new();
                let mut offset = 0u64;
                for text in texts {
                    let place = context.constant_get_element_pointer(
                        blob.to_dynamic(),
                        bytes.get_type(),
                        &[word.constant_integer(0, false), word.constant_integer(offset, false)],
                        lyftar::instructions::GetElementPointerFlags::IN_BOUNDS,
                    );
                    let length = word.constant_integer(text.len() as u64, false);
                    entries.push(context.constant_struct(&[place.to_dynamic(), length.to_dynamic()], false));
                    offset += text.len() as u64;
                }
                let array = context.constant_array(text_type, &entries);
                let global = view.add_global(&name, array.get_type());
                global.set_initializer(array);
                global.set_global_constant(true);
                global.set_visibility(lyftar::enums::Visibility::Hidden);
            };
            for each in messages {
                table(format!("cyborg.messages.{}", each.symbol), &each.texts);
                table(format!("cyborg.sites.{}", each.symbol), &each.sites);
            }
            view.verify().map_err(|error| format!("the messages: {}", error.message()))?;
        }
        let buffer = machine
            .emit_to_memory_buffer(context.exclusive_module(&mut module), CodeGenerationFileType::Object)
            .map_err(|error| error.message().to_string())?;
        Ok(buffer.as_bytes().to_vec())
    })
}

/// The target machine for a target and mode.
fn machine(target: Target, mode: Mode) -> Result<lyftar::target::TargetMachine> {
    use crate::target::Optimization;
    let triple = target.triple();
    let level = match mode {
        Mode::Literal | Mode::Working(Optimization::O0) => CodeGenerationOptimizationLevel::None,
        Mode::Working(Optimization::O1) => CodeGenerationOptimizationLevel::Less,
        Mode::Working(Optimization::O2 | Optimization::Os | Optimization::Oz) | Mode::FinalSmall => {
            CodeGenerationOptimizationLevel::Default
        }
        _ => CodeGenerationOptimizationLevel::Aggressive,
    };
    let relocation = if target.has_os() { RelocationMode::PositionIndependentCode } else { RelocationMode::Static };
    // trace: D365
    let (cpu, features) =
        crate::target::machine_cpu().map_or((target.cpu(), ""), |(cpu, features)| (cpu.as_str(), features.as_str()));
    Ok(LlvmTarget::from_triple(triple).map_err(|error| error.message().to_string())?.create_target_machine(
        triple,
        cpu,
        features,
        level,
        relocation,
        CodeModel::Default,
    ))
}

/// Optimizes one unit's IR and makes its object file, in a context of its own, so units build on separate threads.
///
/// # Errors
/// What LLVM reported, which is a bug in this compiler.
pub fn compile_ir(ir: &str, target: Target, mode: Mode) -> Result<Vec<u8>> {
    optimize(ir, target, mode, false)
}

/// The optimized LLVM IR for a whole program in one unit, as text, so tests can see what each build mode leaves in.
///
/// # Errors
/// What LLVM reported, which is a bug in this compiler.
pub fn optimized_ir(program: &Program, target: Target, mode: Mode) -> Result<String> {
    let unit = Unit::Functions(
        program
            .functions
            .iter()
            .enumerate()
            .filter(|(_, function)| !function.template)
            .map(|(index, _)| index)
            .collect(),
    );
    let (ir, _) = unit_ir(program, target, mode, &unit)?;
    optimize(&ir, target, mode, true).map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
}

fn optimize(ir: &str, target: Target, mode: Mode, as_text: bool) -> Result<Vec<u8>> {
    let machine = machine(target, mode)?;
    with_context(|context| {
        let mut module = context.parse_ir_text(ir).map_err(|error| format!("a unit's IR: {}", error.message()))?;
        context.module(&module).set_target_machine(&machine);
        context
            .exclusive_module(&mut module)
            .run_passes(mode.pipeline(), Some(&machine), &PassOptions::default())
            .map_err(|error| error.message().to_string())?;
        if as_text {
            return Ok(context.module(&module).print_to_string().into_bytes());
        }
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
    target: Target,
    cores_name: u64,
    program: &'module Program,
    symbols: Vec<String>,
    /// Whether this unit holds every function, so all but `main` can be internal to it.
    whole: bool,
    /// The function being made's messages and sites, for the messages unit.
    current: Messages,
    /// Every finished function's messages and sites.
    messages: Vec<Messages>,
    /// Each function, once this unit defines or calls it.
    functions: Vec<Option<Function<'context, 'module>>>,
    /// Each struct's LLVM type, and each union's, with each union's cases as LLVM structs.
    structs: Vec<lyftar::types::Type<'context>>,
    struct_fields: Vec<Vec<MachineType>>,
    /// Each union's cases' fields, by machine type.
    union_fields: Vec<Vec<Vec<MachineType>>>,
    unions: Vec<(lyftar::types::Type<'context>, Vec<lyftar::types::Type<'context>>)>,
}

/// Where each local lives while generating one function.
struct Frame<'context, 'module> {
    function: Function<'context, 'module>,
    slots: Vec<Value<'context, 'module>>,
    loops: HashMap<usize, BasicBlock<'context, 'module>>,
    builder: Builder<'context, 'module>,
    /// Whether this is `fn.main`, which returns 0 to the system.
    is_main: bool,
    /// Each local's machine type.
    local_types: Vec<MachineType>,
    /// Where `instead` goes, for each `otherwise:` around the code being made, innermost last.
    handlers: Vec<BasicBlock<'context, 'module>>,
    /// The function's sites.
    sites: Vec<crate::checked::Site>,
}

/// A value with its machine type, which a failure's `when:` line prints.
type Operand<'context, 'module> = (Value<'context, 'module>, MachineType);

impl<'context, 'module> Generator<'context, 'module> {
    fn runtime(&self, name: &str) -> Result<Function<'context, 'module>> {
        self.module.get_function(name).ok_or_else(|| format!("the runtime has no `{name}`"))
    }

    /// The address of a place and what's there: through fields, and items, each item's position checked against the
    /// count first (D133).
    fn place_pointer(
        &mut self,
        frame: &mut Frame<'context, 'module>,
        place: &crate::checked::Place,
    ) -> Result<(Value<'context, 'module>, MachineType)> {
        use crate::checked::Step;
        let mut pointer = frame.slots[place.local];
        let mut machine_type = frame.local_types.get(place.local).cloned().ok_or("a place in a missing local")?;
        for step in &place.steps {
            match step {
                Step::Field(index) => {
                    let MachineType::Struct(id) = machine_type else {
                        return Err("a field of something that isn't a struct".into());
                    };
                    let struct_type = *self.structs.get(id).ok_or("a struct that wasn't made")?;
                    pointer = frame
                        .builder
                        .struct_get_element_pointer(struct_type, pointer, u32::try_from(*index).unwrap_or(0), "")
                        .to_dynamic();
                    machine_type = self
                        .struct_fields
                        .get(id)
                        .and_then(|fields| fields.get(*index))
                        .cloned()
                        .ok_or("a missing field")?;
                }
                Step::Item { position, from_one, check } => {
                    let written = self.expression(frame, position)?;
                    let MachineType::Integer { bits, signed } = position.machine_type else {
                        return Err("a position that isn't an integer".into());
                    };
                    let word = self.context.integer_type(64);
                    let mut index = match bits.cmp(&64) {
                        std::cmp::Ordering::Less if signed => frame.builder.sign_extend(written, word, "").to_dynamic(),
                        std::cmp::Ordering::Less => frame.builder.zero_extend(written, word, "").to_dynamic(),
                        _ => written,
                    };
                    if *from_one {
                        index = frame.builder.integer_subtract(index, self.integer(64, 1), "").to_dynamic();
                    }
                    let (items, count, item) = match &machine_type {
                        MachineType::Array { item, len } => (pointer, self.integer(64, *len), (**item).clone()),
                        MachineType::List(item) => {
                            let list_type = self.list_type();
                            let storage =
                                frame.builder.struct_get_element_pointer(list_type, pointer, 0, "").to_dynamic();
                            let storage = frame.builder.load(self.context.pointer_type(0), storage, "").to_dynamic();
                            let count =
                                frame.builder.struct_get_element_pointer(list_type, pointer, 1, "").to_dynamic();
                            let count = frame.builder.load(word, count, "").to_dynamic();
                            (storage, count, (**item).clone())
                        }
                        _ => return Err("an item of something that isn't a list or array".into()),
                    };
                    if let Some(failure) = check {
                        let inside = frame
                            .builder
                            .integer_compare(IntegerPredicate::UnsignedLessThan, index, count, "")
                            .to_dynamic();
                        let operands = [
                            (written, position.machine_type.clone()),
                            (count, MachineType::Integer { bits: 64, signed: false }),
                        ];
                        self.check(frame, inside, failure, &operands)?;
                    }
                    let item_type = self.machine_type(&item);
                    pointer = frame.builder.get_element_pointer(item_type, items, &[index], "").to_dynamic();
                    machine_type = item;
                }
            }
        }
        Ok((pointer, machine_type))
    }

    /// A list's record: where its items are, how many there are, and how many fit before it grows (D11).
    /// A list with nothing in it, no allocator yet and no block (D322).
    fn empty_list(&self) -> Value<'context, 'module> {
        let none = integer_constant(self.context, 64, -1);
        let zero = self.integer(64, 0);
        let null = self.context.pointer_type(0).constant_null().to_dynamic();
        self.context.constant_struct(&[null, zero, zero, none, none], false)
    }

    fn list_type(&self) -> lyftar::types::Type<'context> {
        let word = self.context.integer_type(64).to_dynamic();
        self.context.struct_type(&[self.context.pointer_type(0), word, word, word, word], false)
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
            MachineType::Address => self.context.pointer_type(0),
            MachineType::Array { item, len } => self.machine_type(item).array_type(*len),
            MachineType::List(_) => self.list_type(),
            MachineType::Struct(id) => {
                self.structs.get(*id).copied().unwrap_or_else(|| self.context.integer_type(8).to_dynamic())
            }
            MachineType::Union(id) => {
                self.unions.get(*id).map_or_else(|| self.context.integer_type(8).to_dynamic(), |(union, _)| *union)
            }
        }
    }

    fn integer(&self, bits: u32, value: u64) -> Value<'context, 'module> {
        self.context.integer_type(bits).constant_integer(value, false).to_dynamic()
    }

    /// Makes the program's struct and union types, before anything uses them.
    fn aggregates(&mut self, program: &Program) {
        self.struct_fields.clone_from(&program.structs);
        self.union_fields.clone_from(&program.unions);
        // Every named type exists before any body uses it: structs and unions can hold each other.
        for id in 0..program.structs.len() {
            self.structs.push(self.context.create_named_struct(&format!("struct.{id}")));
        }
        for id in 0..program.unions.len() {
            let union = self.context.create_named_struct(&format!("union.{id}"));
            self.unions.push((union, Vec::new()));
        }
        for (id, cases) in program.unions.iter().enumerate() {
            let words = cases
                .iter()
                .map(|fields| fields.iter().map(|field| size_in_words(field, program)).sum::<u64>())
                .max()
                .unwrap_or(0);
            let payload = self.context.integer_type(64).to_dynamic().array_type(words.max(1));
            self.unions[id].0.set_struct_body(&[self.context.integer_type(32).to_dynamic(), payload], false);
        }
        for (id, fields) in program.structs.iter().enumerate() {
            let fields: Vec<_> = fields.iter().map(|field| self.machine_type(field)).collect();
            // trace: D319
            let packed = program.packed.get(id).copied().unwrap_or(false);
            self.structs[id].set_struct_body(&fields, packed);
        }
        for (id, cases) in program.unions.iter().enumerate() {
            let case_types: Vec<_> = cases
                .iter()
                .map(|fields| {
                    let fields: Vec<_> = fields.iter().map(|field| self.machine_type(field)).collect();
                    self.context.struct_type(&fields, false)
                })
                .collect();
            self.unions[id].1 = case_types;
        }
    }

    /// A site of the function being made: its place in the function's table of sites in the messages unit, which is
    /// what travels (D29, D96).
    fn site(&mut self, frame: &Frame<'context, 'module>, index: usize) -> Result<Value<'context, 'module>> {
        if frame.sites.get(index).is_none() {
            return Err("a site the function doesn't have".to_string());
        }
        self.current.sites = frame.sites.iter().map(|site| site.text.clone()).collect();
        let table = self.table(&format!("cyborg.sites.{}", self.current.symbol));
        let text_type = self.machine_type(&MachineType::Text);
        let index = self.integer(64, index as u64);
        Ok(self.context.constant_get_element_pointer(
            table,
            text_type,
            &[index],
            lyftar::instructions::GetElementPointerFlags::IN_BOUNDS,
        ))
    }

    /// A function of the program, declared the first time this unit needs it, so a unit's IR only mentions what it
    /// uses and stays the same when other functions change.
    fn declare(&mut self, index: usize) -> Result<Function<'context, 'module>> {
        if let Some(Some(declared)) = self.functions.get(index) {
            return Ok(*declared);
        }
        let program = self.program;
        let function = program.functions.get(index).ok_or("a call to a missing function")?;
        let is_main = index == program.entry;
        let pointer = self.context.pointer_type(0);
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
        let symbol = if is_main { "main" } else { self.symbols.get(index).map_or("cyborg.function", String::as_str) };
        let declared = self.module.add_function(symbol, self.context.function_type(returns, &parameters, false));
        if !is_main {
            if self.whole {
                declared.set_linkage(Linkage::Internal);
            } else {
                declared.set_visibility(lyftar::enums::Visibility::Hidden);
            }
        }
        if let Some(slot) = self.functions.get_mut(index) {
            *slot = Some(declared);
        }
        Ok(declared)
    }

    fn function(&mut self, index: usize, function: &crate::checked::Function, is_main: bool) -> Result<()> {
        let declared = self.declare(index)?;
        let symbol = self.symbols.get(index).cloned().unwrap_or_default();
        let finished = std::mem::replace(&mut self.current, Messages { symbol, ..Messages::default() });
        if !finished.symbol.is_empty() {
            self.messages.push(finished);
        }
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
        let mut frame = Frame {
            function: declared,
            slots,
            loops: HashMap::new(),
            builder,
            is_main,
            local_types: function.locals.clone(),
            handlers: Vec::new(),
            sites: function.sites.clone(),
        };
        if is_main {
            self.machine_check(&mut frame)?;
        }
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

    /// Makes room in a list for `want` items through its allocator (D321, D322, D385): a wrong allocator is a failed
    /// check, and running out of memory crashes, is ignored by going to `after`, or sets a flag and goes there.
    fn grow_list(
        &mut self,
        frame: &mut Frame<'context, 'module>,
        record: Value<'context, 'module>,
        growth: &crate::checked::Growth,
        want: Value<'context, 'module>,
        size: Value<'context, 'module>,
        after: lyftar::basic_block::BasicBlock<'context, 'module>,
    ) -> Result<()> {
        use crate::checked::OutOfMemory;
        let allocator = self.expression(frame, &growth.allocator)?;
        let report = Self::temporary(frame, self.context.integer_type(64).array_type(4));
        let arguments = [
            record,
            allocator,
            want,
            size,
            self.integer(64, growth.standard),
            self.integer(64, growth.align.max(1)),
            self.integer(64, u64::from(growth.grow)),
            self.integer(64, growth.step.max(1)),
            report,
        ];
        let status = frame.builder.call(self.runtime("__cyborg_list_reserve")?, &arguments, "").to_dynamic();
        let not_wrong =
            frame.builder.integer_compare(IntegerPredicate::NotEqual, status, self.integer(64, 2), "").to_dynamic();
        let wrong_operands = self.report(frame, report, 2);
        self.check(frame, not_wrong, &growth.wrong, &wrong_operands)?;
        let room = frame.builder.integer_compare(IntegerPredicate::Equal, status, self.integer(64, 0), "").to_dynamic();
        match &growth.out_of_memory {
            OutOfMemory::Crash(failure) => {
                let operands = self.report(frame, report, 4);
                self.crash_unless(frame, room, failure, &operands)?;
            }
            OutOfMemory::Ignore => {
                let push = frame.function.append_basic_block("grown");
                frame.builder.conditional_branch(room, push, after);
                frame.builder.position_at_end(push);
            }
            OutOfMemory::Flag(local) => {
                let out = frame.builder.not(room, "").to_dynamic();
                frame.builder.store(out, frame.slots[*local]);
                let push = frame.function.append_basic_block("grown");
                frame.builder.conditional_branch(room, push, after);
                frame.builder.position_at_end(push);
            }
        }
        Ok(())
    }

    /// The first `count` numbers a runtime call wrote in its report, as a failure's operands.
    fn report(
        &mut self,
        frame: &mut Frame<'context, 'module>,
        report: Value<'context, 'module>,
        count: u64,
    ) -> Vec<Operand<'context, 'module>> {
        let word = self.context.integer_type(64).to_dynamic();
        (0..count)
            .map(|index| {
                let place =
                    frame.builder.get_element_pointer(word, report, &[self.integer(64, index)], "").to_dynamic();
                (frame.builder.load(word, place, "").to_dynamic(), MachineType::Integer { bits: 64, signed: false })
            })
            .collect()
    }

    /// A stack slot for a temporary, made in the entry block, so a loop doesn't make a new one each round.
    fn temporary(
        frame: &Frame<'context, 'module>,
        slot_type: lyftar::types::Type<'context>,
    ) -> Value<'context, 'module> {
        let entry = frame.function.entry_basic_block();
        match entry {
            Some(entry) => Builder::at_first_insertion_point(entry).stack_allocate(slot_type, "").to_dynamic(),
            None => frame.builder.stack_allocate(slot_type, "").to_dynamic(),
        }
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
            Statement::Flow(_) => {}
            Statement::Store { place, value } => {
                // The place first, so a position runs before the value (D133).
                let (pointer, _) = self.place_pointer(frame, place)?;
                let value = self.expression(frame, value)?;
                frame.builder.store(value, pointer);
            }
            Statement::Push { place, value, room, growth } => {
                let (record, machine_type) = self.place_pointer(frame, place)?;
                let MachineType::List(item) = machine_type else {
                    return Err("adding to something that isn't a list".into());
                };
                let list_type = self.list_type();
                let word = MachineType::Integer { bits: 64, signed: false };
                let after = frame.function.append_basic_block("after_push");
                let count_place = frame.builder.struct_get_element_pointer(list_type, record, 1, "").to_dynamic();
                let count = frame.builder.load(self.context.integer_type(64), count_place, "").to_dynamic();
                if let Some((max, failure)) = room {
                    let fits = frame
                        .builder
                        .integer_compare(IntegerPredicate::UnsignedLessThan, count, self.integer(64, *max), "")
                        .to_dynamic();
                    match failure {
                        Some(failure) => self.crash_unless(frame, fits, failure, &[(count, word.clone())])?,
                        // `?? void`: a full list is left as it is.
                        None => {
                            let push = frame.function.append_basic_block("push");
                            frame.builder.conditional_branch(fits, push, after);
                            frame.builder.position_at_end(push);
                        }
                    }
                }
                let item_type = self.machine_type(&item);
                let size = self.context.size_of(item_type);
                if let Some(growth) = growth {
                    let one = self.integer(64, 1);
                    let want = frame.builder.integer_add(count, one, "").to_dynamic();
                    self.grow_list(frame, record, growth, want, size, after)?;
                }
                let slot = frame.builder.call(self.runtime("__cyborg_list_push")?, &[record, size], "").to_dynamic();
                let value = self.expression(frame, value)?;
                frame.builder.store(value, slot);
                frame.builder.branch(after);
                frame.builder.position_at_end(after);
            }
            Statement::ListDone { place, allocator, wrong } => {
                // trace: D322, D323
                let (record, _) = self.place_pointer(frame, place)?;
                let allocator = self.expression(frame, allocator)?;
                let report = Self::temporary(frame, self.context.integer_type(64).array_type(4));
                let status = frame
                    .builder
                    .call(self.runtime("__cyborg_list_same_allocator")?, &[record, allocator, report], "")
                    .to_dynamic();
                let fine = frame
                    .builder
                    .integer_compare(IntegerPredicate::Equal, status, self.integer(64, 0), "")
                    .to_dynamic();
                let operands = self.report(frame, report, 2);
                self.check(frame, fine, wrong, &operands)?;
                frame.builder.call(self.runtime("__cyborg_list_done")?, &[record, allocator], "");
            }
            Statement::TextDone(value) => {
                let text = self.expression(frame, value)?;
                frame.builder.call(self.runtime("__cyborg_text_free")?, &[text], "");
            }
            Statement::ArenaDone { arena, alive } => {
                // trace: D330
                let arena = self.expression(frame, arena)?;
                let count = frame.builder.call(self.runtime("__cyborg_alive")?, &[arena], "").to_dynamic();
                let none =
                    frame.builder.integer_compare(IntegerPredicate::Equal, count, self.integer(64, 0), "").to_dynamic();
                self.check(frame, none, alive, &[(count, MachineType::Integer { bits: 64, signed: false })])?;
                frame.builder.call(self.runtime("__cyborg_arena_free")?, &[arena], "");
            }
            Statement::AliasValue { local, value } => {
                let address = self.expression(frame, value)?;
                if let Some(slot) = frame.slots.get_mut(*local) {
                    *slot = address;
                }
            }
            Statement::Alias { local, target, path } => {
                let place = self.field_place(frame, *target, path)?;
                if let Some(slot) = frame.slots.get_mut(*local) {
                    *slot = place;
                }
            }
            Statement::Let { local, value } | Statement::Set { local, value } => {
                let value = self.expression(frame, value)?;
                frame.builder.store(value, frame.slots[*local]);
            }
            Statement::Evaluate(value) => {
                self.expression(frame, value)?;
            }
            Statement::Instead { local, value } => {
                let value = self.expression(frame, value)?;
                frame.builder.store(value, frame.slots[*local]);
                let after = *frame.handlers.last().ok_or("`instead` outside its arms")?;
                frame.builder.branch(after);
            }
            Statement::SetField { local, path, value } => {
                let value = self.expression(frame, value)?;
                let place = self.field_place(frame, *local, path)?;
                frame.builder.store(value, place);
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
            Statement::Loop { id, body, .. } => {
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
            Statement::TaskStart { task, pool, call } => self.task_start(frame, *task, pool, call)?,
            Statement::TaskJoin { task, result } => {
                // trace: D343, D357
                let pointer = self.context.pointer_type(0);
                let record = frame.builder.load(pointer, frame.slots[*task], "").to_dynamic();
                frame.builder.call(self.runtime("__cyborg_task_join")?, &[record], "");
                let result_type = frame.local_types.get(*result).cloned().unwrap_or(MachineType::Void);
                if result_type != MachineType::Void {
                    let stored = self.machine_type(&result_type);
                    let head = self.context.struct_type(&[pointer, pointer, stored], false);
                    let place = frame.builder.struct_get_element_pointer(head, record, 2, "").to_dynamic();
                    let value = frame.builder.load(stored, place, "").to_dynamic();
                    frame.builder.store(value, frame.slots[*result]);
                }
                frame.builder.call(self.runtime("__cyborg_task_free")?, &[record], "");
            }
            Statement::PoolDone(pool) => {
                let pool = self.expression(frame, pool)?;
                frame.builder.call(self.runtime("__cyborg_pool_done")?, &[pool], "");
            }
            Statement::LockAll { addresses, count } | Statement::UnlockAll { addresses, count } => {
                // trace: D348, D349
                let name = if matches!(statement, Statement::LockAll { .. }) {
                    "__cyborg_lock_all"
                } else {
                    "__cyborg_unlock_all"
                };
                let function = self.runtime(name)?;
                frame.builder.call(function, &[frame.slots[*addresses], self.integer(64, *count)], "");
            }
            Statement::Flush { writer } => {
                let flush = self.runtime("__cyborg_writer_flush")?;
                frame.builder.call(flush, &[frame.slots[*writer]], "");
            }
            Statement::Done { writer } => {
                let done = self.runtime("__cyborg_writer_done")?;
                frame.builder.call(done, &[frame.slots[*writer]], "");
            }
            Statement::CheckAtSite { condition, failure, operands, site } => {
                let mut values = Vec::new();
                for operand in operands {
                    values.push((self.expression(frame, operand)?, operand.machine_type.clone()));
                }
                let site = self.expression(frame, site)?;
                let holds = self.expression(frame, condition)?;
                let fine = frame.function.append_basic_block("fine");
                let failed = frame.function.append_basic_block("failed");
                frame.builder.conditional_branch(holds, fine, failed);
                frame.builder.position_at_end(failed);
                self.crash_with_site(frame, failure, &values, site)?;
                frame.builder.unreachable();
                frame.builder.position_at_end(fine);
            }
            Statement::Assembly { lines, reads, writes, clobbers, intel } => {
                self.assembly(frame, lines, reads, writes, clobbers, *intel)?;
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

    /// An assembly block, as LLVM inline assembly: each `writes:` register is an output, each `reads:` one an input,
    /// and each `clobbers:` one is changed (D233). `$` in the text is LLVM's operand mark, so it's doubled.
    fn assembly(
        &mut self,
        frame: &mut Frame<'context, 'module>,
        lines: &[String],
        reads: &[(String, crate::checked::Expression)],
        writes: &[(String, usize, MachineType)],
        clobbers: &[String],
        intel: bool,
    ) -> Result<()> {
        let mut constraints: Vec<String> = writes.iter().map(|(register, _, _)| format!("={{{register}}}")).collect();
        constraints.extend(reads.iter().map(|(register, _)| format!("{{{register}}}")));
        constraints.extend(clobbers.iter().map(|register| format!("~{{{register}}}")));
        let mut inputs = Vec::new();
        let mut input_types = Vec::new();
        for (_, value) in reads {
            input_types.push(self.machine_type(&value.machine_type));
            inputs.push(self.expression(frame, value)?);
        }
        let output_types: Vec<_> = writes.iter().map(|(_, _, machine_type)| self.machine_type(machine_type)).collect();
        let returns = match output_types.as_slice() {
            [] => self.context.void_type().to_dynamic(),
            [one] => *one,
            many => self.context.struct_type(many, false).to_dynamic(),
        };
        let function_type = self.context.function_type(returns, &input_types, false);
        let text = lines.iter().map(|line| line.replace('$', "$$")).collect::<Vec<_>>().join("\n");
        let dialect =
            if intel { lyftar::enums::InlineAssemblyDialect::Intel } else { lyftar::enums::InlineAssemblyDialect::Att };
        let assembly = function_type.inline_assembly(&text, &constraints.join(","), true, false, dialect, false);
        let result = frame.builder.call(assembly, inputs.as_slice(), "").to_dynamic();
        match writes {
            [] => {}
            [(_, local, _)] => {
                frame.builder.store(result, frame.slots[*local]);
            }
            many => {
                for (index, (_, local, _)) in many.iter().enumerate() {
                    let value = frame.builder.extract_value(result, u32::try_from(index).unwrap_or(0), "").to_dynamic();
                    frame.builder.store(value, frame.slots[*local]);
                }
            }
        }
        Ok(())
    }

    /// Whether this mode leaves out a check: only a decided one, since a stop for something not decided yet stays in
    /// every mode (rule 4).
    fn drops(&self, failure: &CheckFailure) -> bool {
        self.mode.checks() != Checks::Kept && Self::decided(failure)
    }

    fn decided(failure: &CheckFailure) -> bool {
        failure.source.as_ref().is_some_and(|source| source.kind == crate::diagnostic::Kind::CheckFailed)
    }

    /// Crashes with `failure`'s message unless `holds` is true. Where the mode drops decided checks (standard 10.3),
    /// `FinalPerf` and `FinalShit` hand `holds` to the optimizer as a fact instead (10.6), and `FinalSmall` leaves it.
    fn check(
        &mut self,
        frame: &mut Frame<'context, 'module>,
        holds: Value<'context, 'module>,
        failure: &CheckFailure,
        operands: &[Operand<'context, 'module>],
    ) -> Result<()> {
        // trace: D14
        if self.drops(failure) {
            if self.mode.checks() == Checks::Assumed {
                let intrinsic = Intrinsic::find("llvm.assume").ok_or("LLVM has no `llvm.assume`")?;
                let declaration = self.module.intrinsic_declaration(intrinsic, &[]);
                frame.builder.call(declaration, &[holds], "");
            }
            return Ok(());
        }
        self.crash_unless(frame, holds, failure, operands)
    }

    /// Crashes with `failure`'s message unless `holds` is true, in every mode: a failure nothing handles, like
    /// running out of memory, isn't a check a mode can drop (D96).
    fn crash_unless(
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
            let writer = Self::temporary(frame, writer_type);
            frame.builder.call(self.runtime("__cyborg_crash_begin")?, &[writer], "");
            self.message(frame, writer, &failure.before)?;
            for (text, operand) in &failure.when {
                self.message(frame, writer, text)?;
                let (value, machine_type) =
                    operands.get(*operand).cloned().ok_or("a failure names a missing operand")?;
                self.print_number(frame, writer, value, &machine_type, &Shape::default())?;
            }
            self.message(frame, writer, &failure.after)?;
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
                Piece::Value { value, shape, failures } => {
                    let generated = self.expression(frame, value)?;
                    match &value.machine_type {
                        MachineType::Float { bits } => {
                            self.print_float(frame, writer, generated, *bits, shape, failures.as_deref())?;
                        }
                        MachineType::Integer { signed: true, bits } if failures.is_some() => {
                            if let Some(failures) = failures {
                                let zero = integer_constant(self.context, *bits, 0);
                                let fine = frame
                                    .builder
                                    .integer_compare(IntegerPredicate::SignedGreaterOrEqual, generated, zero, "")
                                    .to_dynamic();
                                self.check(frame, fine, &failures.halfway, &[])?;
                            }
                            self.print_number(frame, writer, generated, &value.machine_type, shape)?;
                        }
                        MachineType::Text => {
                            let columns = match &value.kind {
                                ExpressionKind::Text(text) => shape.columns_of(text),
                                // With no width, columns aren't needed.
                                _ => 0,
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

    /// Writes part of a crash message. Its text shows lines and columns, which move whenever lines above change, so it
    /// lives in the messages unit, in its function's table, read by its place there; the function's own code, and so
    /// its cached object, doesn't change (D168).
    fn message(
        &mut self,
        frame: &mut Frame<'context, 'module>,
        writer: Value<'context, 'module>,
        text: &str,
    ) -> Result<()> {
        if text.is_empty() {
            return Ok(());
        }
        let index = self.current.texts.len();
        self.current.texts.push(text.to_string());
        let table = self.table(&format!("cyborg.messages.{}", self.current.symbol));
        let text_type = self.machine_type(&MachineType::Text);
        let place =
            frame.builder.get_element_pointer(text_type, table, &[self.integer(64, index as u64)], "").to_dynamic();
        let held = frame.builder.load(text_type, place, "").to_dynamic();
        let data = frame.builder.extract_value(held, 0, "").to_dynamic();
        let length = frame.builder.extract_value(held, 1, "").to_dynamic();
        frame.builder.call(self.runtime("__cyborg_writer_write")?, &[writer, data, length], "");
        Ok(())
    }

    /// A function's table of texts in the messages unit, declared here.
    fn table(&mut self, name: &str) -> Value<'context, 'module> {
        if let Some(found) = self.module.get_global(name) {
            return found.to_dynamic();
        }
        let text_type = self.machine_type(&MachineType::Text);
        let global = self.module.add_global(name, text_type);
        global.set_visibility(lyftar::enums::Visibility::Hidden);
        global.set_global_constant(true);
        global.to_dynamic()
    }

    /// Prints a number or a `bool` with its shape.
    /// A failing call, and what happens when it fails (standard 8).
    #[allow(clippy::too_many_lines)]
    fn handle(
        &mut self,
        frame: &mut Frame<'context, 'module>,
        call: &Expression,
        union: usize,
        on_failure: &crate::checked::OnFailure,
        value_type: &MachineType,
    ) -> Result<Value<'context, 'module>> {
        use crate::checked::OnFailure;
        let gives_value = !matches!(value_type, MachineType::Void);
        let result = self.expression(frame, call)?;
        let failure = frame.builder.extract_value(result, 1, "").to_dynamic();
        let tag = frame.builder.extract_value(failure, 0, "").to_dynamic();
        let site = frame.builder.extract_value(result, 2, "").to_dynamic();
        // The value goes in one slot on every path: the arms' local, which `instead` writes too, or a temporary.
        let value_slot = match on_failure {
            OnFailure::Arms { result, .. } if gives_value => Some(frame.slots[*result]),
            _ if gives_value => Some(Self::temporary(frame, self.machine_type(value_type))),
            _ => None,
        };
        let ok = frame.function.append_basic_block("call_fine");
        let failed = frame.function.append_basic_block("call_failed");
        let after = frame.function.append_basic_block("after_call");
        let fine = frame.builder.integer_compare(IntegerPredicate::Equal, tag, self.integer(32, 0), "").to_dynamic();
        frame.builder.conditional_branch(fine, ok, failed);
        frame.builder.position_at_end(ok);
        if let Some(slot) = value_slot {
            let value = frame.builder.extract_value(result, 0, "").to_dynamic();
            frame.builder.store(value, slot);
        }
        frame.builder.branch(after);
        frame.builder.position_at_end(failed);
        let case_value = |generator: &mut Self,
                          frame: &mut Frame<'context, 'module>,
                          case: usize,
                          index: usize|
         -> Result<Value<'context, 'module>> {
            let (union_type, case_types) = generator.unions.get(union).cloned().ok_or("a union that wasn't made")?;
            let case_type = *case_types.get(case).ok_or("a case that wasn't made")?;
            let slot = Self::temporary(frame, union_type);
            frame.builder.store(failure, slot);
            let payload = frame.builder.struct_get_element_pointer(union_type, slot, 1, "").to_dynamic();
            let whole = frame.builder.load(case_type, payload, "").to_dynamic();
            Ok(frame.builder.extract_value(whole, u32::try_from(index).unwrap_or(0), "").to_dynamic())
        };
        match on_failure {
            OnFailure::Crash(messages) => {
                for message in messages {
                    let this_case = frame.function.append_basic_block("this_failure");
                    let next = frame.function.append_basic_block("next_failure");
                    let is_case = frame
                        .builder
                        .integer_compare(IntegerPredicate::Equal, tag, self.integer(32, message.case as u64), "")
                        .to_dynamic();
                    frame.builder.conditional_branch(is_case, this_case, next);
                    frame.builder.position_at_end(this_case);
                    let mut operands = Vec::new();
                    if message.carried {
                        let field_type = self
                            .union_fields
                            .get(union)
                            .and_then(|cases| cases.get(message.case))
                            .and_then(|fields| fields.first())
                            .cloned()
                            .ok_or("a carried value that isn't there")?;
                        operands.push((case_value(self, frame, message.case, 0)?, field_type));
                    }
                    self.crash_with_site(frame, &message.failure, &operands, site)?;
                    frame.builder.unreachable();
                    frame.builder.position_at_end(next);
                }
                frame.builder.unreachable();
            }
            OnFailure::Ignore => {
                frame.builder.branch(after);
            }
            OnFailure::Fallback(fallback) => {
                let value = self.expression(frame, fallback)?;
                if let Some(slot) = value_slot {
                    frame.builder.store(value, slot);
                }
                frame.builder.branch(after);
            }
            OnFailure::PassUp { mapping, structure, union: caller_union, value_type: caller_value, cleanup } => {
                let (caller_union_type, caller_cases) =
                    self.unions.get(*caller_union).cloned().ok_or("a union that wasn't made")?;
                let caller_struct = *self.structs.get(*structure).ok_or("a struct that wasn't made")?;
                for (from, to) in mapping {
                    let this_case = frame.function.append_basic_block("pass_up");
                    let next = frame.function.append_basic_block("next_failure");
                    let is_case = frame
                        .builder
                        .integer_compare(IntegerPredicate::Equal, tag, self.integer(32, *from as u64), "")
                        .to_dynamic();
                    frame.builder.conditional_branch(is_case, this_case, next);
                    frame.builder.position_at_end(this_case);
                    let fields =
                        self.union_fields.get(union).and_then(|cases| cases.get(*from)).cloned().unwrap_or_default();
                    let case_type = *caller_cases.get(*to).ok_or("a case that wasn't made")?;
                    let mut data = case_type.undefined().to_dynamic();
                    for index in 0..fields.len() {
                        let value = case_value(self, frame, *from, index)?;
                        data =
                            frame.builder.insert_value(data, value, u32::try_from(index).unwrap_or(0), "").to_dynamic();
                    }
                    let slot = Self::temporary(frame, caller_union_type);
                    let tag_place =
                        frame.builder.struct_get_element_pointer(caller_union_type, slot, 0, "").to_dynamic();
                    frame.builder.store(self.integer(32, *to as u64), tag_place);
                    let payload = frame.builder.struct_get_element_pointer(caller_union_type, slot, 1, "").to_dynamic();
                    frame.builder.store(data, payload);
                    let passed_failure = frame.builder.load(caller_union_type, slot, "").to_dynamic();
                    let mut passed = caller_struct.undefined().to_dynamic();
                    let zero_value = self.machine_type(caller_value).constant_null().to_dynamic();
                    passed = frame.builder.insert_value(passed, zero_value, 0, "").to_dynamic();
                    passed = frame.builder.insert_value(passed, passed_failure, 1, "").to_dynamic();
                    passed = frame.builder.insert_value(passed, site, 2, "").to_dynamic();
                    // trace: D325
                    self.block(frame, cleanup)?;
                    frame.builder.return_value(passed);
                    frame.builder.position_at_end(next);
                }
                frame.builder.unreachable();
            }
            OnFailure::Arms { arms, result, union_local } => {
                frame.builder.store(failure, frame.slots[*union_local]);
                frame.handlers.push(after);
                for (case, statements) in arms {
                    let this_case = frame.function.append_basic_block("arm");
                    let next = frame.function.append_basic_block("next_arm");
                    let is_case = frame
                        .builder
                        .integer_compare(IntegerPredicate::Equal, tag, self.integer(32, *case as u64), "")
                        .to_dynamic();
                    frame.builder.conditional_branch(is_case, this_case, next);
                    frame.builder.position_at_end(this_case);
                    self.block(frame, statements)?;
                    Self::branch_if_open(frame, after);
                    frame.builder.position_at_end(next);
                }
                frame.handlers.pop();
                let _ = result;
                frame.builder.unreachable();
            }
        }
        frame.builder.position_at_end(after);
        match value_slot {
            Some(slot) => Ok(frame.builder.load(self.machine_type(value_type), slot, "").to_dynamic()),
            None => Ok(self.integer(8, 0)),
        }
    }

    /// Crashes with a failure's message, then where it was raised, from the site's text (D96).
    fn crash_with_site(
        &mut self,
        frame: &mut Frame<'context, 'module>,
        failure: &CheckFailure,
        operands: &[Operand<'context, 'module>],
        site: Value<'context, 'module>,
    ) -> Result<()> {
        let writer_type = self.machine_type(&MachineType::Writer);
        let writer = Self::temporary(frame, writer_type);
        frame.builder.call(self.runtime("__cyborg_crash_begin")?, &[writer], "");
        self.message(frame, writer, &failure.before)?;
        for (text, operand) in &failure.when {
            self.message(frame, writer, text)?;
            let (value, machine_type) = operands.get(*operand).cloned().ok_or("a failure names a missing operand")?;
            self.print_number(frame, writer, value, &machine_type, &Shape::default())?;
        }
        self.message(frame, writer, &failure.after)?;
        let entry_type = self.machine_type(&MachineType::Text);
        let text = frame.builder.load(entry_type, site, "").to_dynamic();
        let data = frame.builder.extract_value(text, 0, "").to_dynamic();
        let length = frame.builder.extract_value(text, 1, "").to_dynamic();
        frame.builder.call(self.runtime("__cyborg_writer_write")?, &[writer, data, length], "");
        frame.builder.call(self.runtime("__cyborg_exit")?, &[self.integer(32, 1)], "");
        Ok(())
    }

    /// The address of a field, maybe deep, inside a local.
    fn field_place(
        &mut self,
        frame: &mut Frame<'context, 'module>,
        local: usize,
        path: &[usize],
    ) -> Result<Value<'context, 'module>> {
        let mut place = frame.slots[local];
        let mut machine_type = frame.local_types.get(local).cloned().ok_or("a field of a missing local")?;
        for index in path {
            let MachineType::Struct(id) = machine_type else {
                return Err("a field of something that isn't a struct".into());
            };
            let struct_type = *self.structs.get(id).ok_or("a struct that wasn't made")?;
            place = frame
                .builder
                .struct_get_element_pointer(struct_type, place, u32::try_from(*index).unwrap_or(0), "")
                .to_dynamic();
            machine_type =
                self.struct_fields.get(id).and_then(|fields| fields.get(*index)).cloned().ok_or("a missing field")?;
        }
        Ok(place)
    }

    /// Formats a float with the runtime and prints it, crashing on what isn't decided (D161, D162, D194).
    fn print_float(
        &mut self,
        frame: &mut Frame<'context, 'module>,
        writer: Value<'context, 'module>,
        value: Value<'context, 'module>,
        bits: u32,
        shape: &Shape,
        failures: Option<&crate::checked::FloatPrintFailures>,
    ) -> Result<()> {
        let wide = if bits == 32 {
            frame.builder.float_extend(value, self.context.get_type::<F64>(), "").to_dynamic()
        } else {
            value
        };
        let shaped = shape.places.is_some() || shape.science || shape.width.is_some() || shape.sign || shape.zeros;
        if shaped && let Some(failures) = failures {
            // NaN with shapes on it isn't decided.
            let is_number =
                frame.builder.float_compare(lyftar::enums::FloatPredicate::Ordered, wide, wide, "").to_dynamic();
            self.check(frame, is_number, &failures.nan, &[])?;
        }
        let form = match (shape.stored, shape.places.is_some(), shape.science) {
            (true, _, _) => 1,
            (false, true, false) => 2,
            (false, false, true) => 3,
            (false, true, true) => 4,
            (false, false, false) => 0,
        };
        let places = shape.places.unwrap_or(0);
        let buffer_type = self.context.integer_type(8).to_dynamic().array_type(1400 + u64::from(places));
        let buffer = Self::temporary(frame, buffer_type);
        let arguments = [
            wide,
            self.integer(32, u64::from(bits)),
            self.integer(32, form),
            self.integer(32, u64::from(places)),
            self.integer(1, u64::from(shape.sign)),
            buffer,
        ];
        let length = frame.builder.call(self.runtime("__cyborg_format_float")?, &arguments, "").to_dynamic();
        if let Some(failures) = failures {
            let zero = self.integer(64, 0);
            let not_halfway = frame
                .builder
                .integer_compare(IntegerPredicate::NotEqual, length, integer_constant(self.context, 64, -1), "")
                .to_dynamic();
            self.check(frame, not_halfway, &failures.halfway, &[(value, MachineType::Float { bits })])?;
            let fine =
                frame.builder.integer_compare(IntegerPredicate::SignedGreaterOrEqual, length, zero, "").to_dynamic();
            self.check(frame, fine, &failures.science, &[])?;
        }
        let align = match shape.align {
            Some(crate::checked::Align::Left) => 0,
            Some(crate::checked::Align::Center) => 2,
            _ => 1,
        };
        let arguments = [
            writer,
            buffer,
            length,
            self.integer(64, shape.width.unwrap_or(0)),
            self.integer(32, align),
            self.integer(1, u64::from(shape.zeros)),
        ];
        frame.builder.call(self.runtime("__cyborg_print_padded")?, &arguments, "");
        Ok(())
    }

    fn print_number(
        &mut self,
        frame: &mut Frame<'context, 'module>,
        writer: Value<'context, 'module>,
        value: Value<'context, 'module>,
        machine_type: &MachineType,
        shape: &Shape,
    ) -> Result<()> {
        if let MachineType::Float { bits } = *machine_type {
            return self.print_float(frame, writer, value, bits, shape, None);
        }
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
            ExpressionKind::Site(index) => self.site(frame, *index)?,
            ExpressionKind::Integer(value) => {
                let MachineType::Integer { bits, .. } = expression.machine_type else {
                    return Err("an integer that isn't one".to_string());
                };
                integer_constant(self.context, bits, *value)
            }
            ExpressionKind::Float(value) => match expression.machine_type {
                MachineType::Float { bits: 32 } => self.context.get_type::<F32>().constant_float(*value).to_dynamic(),
                _ => self.context.get_type::<F64>().constant_float(*value).to_dynamic(),
            },
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
            ExpressionKind::Compare { operator, left, right }
                if matches!(left.machine_type, MachineType::Float { .. }) =>
            {
                use lyftar::enums::FloatPredicate;
                let left = self.expression(frame, left)?;
                let right = self.expression(frame, right)?;
                // IEEE 754 decides comparing (D167): anything with NaN is false, except `!=`.
                let predicate = match operator {
                    Comparison::Equal => FloatPredicate::OrderedEqual,
                    Comparison::NotEqual => FloatPredicate::UnorderedNotEqual,
                    Comparison::Below => FloatPredicate::OrderedLessThan,
                    Comparison::Above => FloatPredicate::OrderedGreaterThan,
                    Comparison::AtLeast => FloatPredicate::OrderedGreaterOrEqual,
                    Comparison::AtMost => FloatPredicate::OrderedLessOrEqual,
                };
                frame.builder.float_compare(predicate, left, right, "").to_dynamic()
            }
            ExpressionKind::FloatArithmetic { operator, left, right, infinity, nan } => {
                use crate::checked::FloatArithmetic;
                let left_value = self.expression(frame, left)?;
                let right_value = self.expression(frame, right)?;
                let result = match operator {
                    FloatArithmetic::Add => frame.builder.float_add(left_value, right_value, "").to_dynamic(),
                    FloatArithmetic::Subtract => frame.builder.float_subtract(left_value, right_value, "").to_dynamic(),
                    FloatArithmetic::Multiply => frame.builder.float_multiply(left_value, right_value, "").to_dynamic(),
                    FloatArithmetic::Divide => frame.builder.float_divide(left_value, right_value, "").to_dynamic(),
                };
                let operands = [(left_value, left.machine_type.clone()), (right_value, right.machine_type.clone())];
                if let Some(infinity) = infinity {
                    let MachineType::Float { bits } = expression.machine_type else {
                        return Err("float maths on a non-float".into());
                    };
                    let magnitude = self.float_magnitude(frame, result, bits)?;
                    let infinite = if bits == 32 {
                        self.context.get_type::<F32>().constant_float(f64::INFINITY).to_dynamic()
                    } else {
                        self.context.get_type::<F64>().constant_float(f64::INFINITY).to_dynamic()
                    };
                    let finite = frame
                        .builder
                        .float_compare(lyftar::enums::FloatPredicate::UnorderedNotEqual, magnitude, infinite, "")
                        .to_dynamic();
                    self.check(frame, finite, infinity, &operands)?;
                }
                if let Some(nan) = nan {
                    let ordered = frame
                        .builder
                        .float_compare(lyftar::enums::FloatPredicate::Ordered, result, result, "")
                        .to_dynamic();
                    self.check(frame, ordered, nan, &operands)?;
                }
                result
            }
            ExpressionKind::TextOf(value) => {
                let generated = self.expression(frame, value)?;
                match value.machine_type {
                    MachineType::Float { bits } => {
                        let wide = if bits == 32 {
                            frame.builder.float_extend(generated, self.context.get_type::<F64>(), "").to_dynamic()
                        } else {
                            generated
                        };
                        let arguments = [wide, self.integer(32, u64::from(bits))];
                        frame.builder.call(self.runtime("__cyborg_text_of_float")?, &arguments, "").to_dynamic()
                    }
                    MachineType::Integer { bits, signed } => {
                        let i64_type = self.context.integer_type(64);
                        let wide = match bits.cmp(&64) {
                            std::cmp::Ordering::Equal => generated,
                            std::cmp::Ordering::Greater => frame.builder.truncate(generated, i64_type, "").to_dynamic(),
                            std::cmp::Ordering::Less if signed => {
                                frame.builder.sign_extend(generated, i64_type, "").to_dynamic()
                            }
                            std::cmp::Ordering::Less => frame.builder.zero_extend(generated, i64_type, "").to_dynamic(),
                        };
                        let zero = self.integer(64, 0);
                        let (magnitude, negative) = if signed {
                            let negative = frame
                                .builder
                                .integer_compare(IntegerPredicate::SignedLessThan, wide, zero, "")
                                .to_dynamic();
                            let flipped = frame.builder.integer_subtract(zero, wide, "").to_dynamic();
                            (frame.builder.select(negative, flipped, wide, "").to_dynamic(), negative)
                        } else {
                            (wide, self.integer(1, 0))
                        };
                        frame
                            .builder
                            .call(self.runtime("__cyborg_text_of_integer")?, &[magnitude, negative], "")
                            .to_dynamic()
                    }
                    _ => return Err("text of something that isn't a number".to_string()),
                }
            }
            ExpressionKind::FromIeee { value, nan, infinity } => {
                use lyftar::enums::FloatPredicate;
                let MachineType::Float { bits } = value.machine_type else {
                    return Err("`as` from a non-float".into());
                };
                let generated = self.expression(frame, value)?;
                let ordered =
                    frame.builder.float_compare(FloatPredicate::Ordered, generated, generated, "").to_dynamic();
                self.check(frame, ordered, nan, &[])?;
                let magnitude = self.float_magnitude(frame, generated, bits)?;
                let infinite = if bits == 32 {
                    self.context.get_type::<F32>().constant_float(f64::INFINITY).to_dynamic()
                } else {
                    self.context.get_type::<F64>().constant_float(f64::INFINITY).to_dynamic()
                };
                let finite = frame
                    .builder
                    .float_compare(FloatPredicate::UnorderedNotEqual, magnitude, infinite, "")
                    .to_dynamic();
                self.check(frame, finite, infinity, &[])?;
                generated
            }
            ExpressionKind::FitFloat { value, min, max, failure } => {
                use lyftar::enums::FloatPredicate;
                let MachineType::Float { bits } = value.machine_type else {
                    return Err("a float bound on a non-float".into());
                };
                let generated = self.expression(frame, value)?;
                let constant = |number: f64| {
                    if bits == 32 {
                        self.context.get_type::<F32>().constant_float(number).to_dynamic()
                    } else {
                        self.context.get_type::<F64>().constant_float(number).to_dynamic()
                    }
                };
                let (low, high) = (constant(*min), constant(*max));
                let above_min =
                    frame.builder.float_compare(FloatPredicate::OrderedGreaterOrEqual, generated, low, "").to_dynamic();
                let below_max =
                    frame.builder.float_compare(FloatPredicate::OrderedLessOrEqual, generated, high, "").to_dynamic();
                let fits = frame.builder.and(above_min, below_max, "").to_dynamic();
                self.check(frame, fits, failure, &[(generated, value.machine_type.clone())])?;
                generated
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
            ExpressionKind::Call { function, arguments, .. } => {
                let callee = self.declare(*function)?;
                let mut values = Vec::with_capacity(arguments.len());
                for argument in arguments {
                    values.push(match argument {
                        Argument::Value(value) => self.expression(frame, value)?,
                        Argument::Address(local) => frame.slots[*local],
                        Argument::FieldAddress { local, path } => self.field_place(frame, *local, path)?,
                        Argument::Place(place) => self.place_pointer(frame, place)?.0,
                        Argument::Temporary(value) => {
                            let generated = self.expression(frame, value)?;
                            let slot_type = self.machine_type(&value.machine_type);
                            let slot = Self::temporary(frame, slot_type);
                            frame.builder.store(generated, slot);
                            slot
                        }
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
            ExpressionKind::Select { condition, then, otherwise } => {
                let condition = self.expression(frame, condition)?;
                let then = self.expression(frame, then)?;
                let otherwise = self.expression(frame, otherwise)?;
                frame.builder.select(condition, then, otherwise, "").to_dynamic()
            }
            ExpressionKind::UnionBits(value) => {
                // trace: D398
                let generated = self.expression(frame, value)?;
                let target = self.machine_type(&expression.machine_type);
                let as_integer = match value.machine_type {
                    MachineType::Float { bits } => frame
                        .builder
                        .cast(lyftar::enums::Opcode::BitCast, generated, self.context.integer_type(bits), "")
                        .to_dynamic(),
                    _ => generated,
                };
                frame.builder.zero_extend_or_bit_cast(as_integer, target, "").to_dynamic()
            }
            ExpressionKind::Bits(value) => {
                // trace: D398
                let generated = self.expression(frame, value)?;
                match expression.machine_type {
                    MachineType::Float { bits } => {
                        let narrow = frame
                            .builder
                            .truncate_or_bit_cast(generated, self.context.integer_type(bits), "")
                            .to_dynamic();
                        let float = self.machine_type(&expression.machine_type);
                        frame.builder.cast(lyftar::enums::Opcode::BitCast, narrow, float, "").to_dynamic()
                    }
                    MachineType::Bool => {
                        frame.builder.truncate_or_bit_cast(generated, self.context.integer_type(1), "").to_dynamic()
                    }
                    _ => {
                        let target = self.machine_type(&expression.machine_type);
                        frame.builder.truncate_or_bit_cast(generated, target, "").to_dynamic()
                    }
                }
            }
            ExpressionKind::AddressAt(address) => {
                let number = self.expression(frame, address)?;
                frame.builder.integer_to_pointer(number, self.context.pointer_type(0), "").to_dynamic()
            }
            ExpressionKind::PointerRead { pointer, volatile } => {
                let pointer = self.expression(frame, pointer)?;
                let loaded_type = self.machine_type(&expression.machine_type);
                let loaded = frame.builder.load(loaded_type, pointer, "");
                if *volatile && let Some(load) = loaded.as_kind::<lyftar::values::LoadInstruction>() {
                    load.set_volatile(true);
                }
                loaded.to_dynamic()
            }
            ExpressionKind::PointerWrite { pointer, value, volatile } => {
                let pointer = self.expression(frame, pointer)?;
                let value = self.expression(frame, value)?;
                let stored = frame.builder.store(value, pointer);
                if *volatile {
                    stored.set_volatile(true);
                }
                self.context.pointer_type(0).constant_null().to_dynamic()
            }
            ExpressionKind::Atomic { address, operation, ordering, failure } => {
                self.atomic(frame, address, operation, *ordering, failure.as_deref(), &expression.machine_type)?
            }
            ExpressionKind::PointerMoved { pointer, bytes } => {
                let pointer = self.expression(frame, pointer)?;
                let bytes = self.expression(frame, bytes)?;
                let byte = self.context.integer_type(8).to_dynamic();
                frame.builder.get_element_pointer(byte, pointer, &[bytes], "").to_dynamic()
            }
            ExpressionKind::Split { list, at, failure } => {
                // trace: D382
                let (pointer, machine_type) = self.place_pointer(frame, list)?;
                let item_type = match &machine_type {
                    MachineType::List(item) => self.machine_type(item),
                    _ => return Err("a split of something that isn't a list".into()),
                };
                let list_type = self.list_type();
                let record = frame.builder.load(list_type, pointer, "").to_dynamic();
                let data = frame.builder.extract_value(record, 0, "").to_dynamic();
                let count = frame.builder.extract_value(record, 1, "").to_dynamic();
                let place = self.expression(frame, at)?;
                let place = frame.builder.integer_cast(place, self.context.integer_type(64), true, "").to_dynamic();
                let inside =
                    frame.builder.integer_compare(IntegerPredicate::UnsignedLessOrEqual, place, count, "").to_dynamic();
                self.check(frame, inside, failure, &[(place, MachineType::I64), (count, MachineType::I64)])?;
                let rest = frame.builder.integer_subtract(count, place, "").to_dynamic();
                let later = frame.builder.get_element_pointer(item_type, data, &[place], "").to_dynamic();
                let none = integer_constant(self.context, 64, -1);
                let view = |frame: &mut Frame<'context, 'module>, start, length| {
                    let mut record = list_type.undefined().to_dynamic();
                    record = frame.builder.insert_value(record, start, 0, "").to_dynamic();
                    record = frame.builder.insert_value(record, length, 1, "").to_dynamic();
                    record = frame.builder.insert_value(record, length, 2, "").to_dynamic();
                    record = frame.builder.insert_value(record, none, 3, "").to_dynamic();
                    frame.builder.insert_value(record, none, 4, "").to_dynamic()
                };
                let first = view(frame, data, place);
                let second = view(frame, later, rest);
                let mut halves = self.machine_type(&expression.machine_type).undefined().to_dynamic();
                halves = frame.builder.insert_value(halves, first, 0, "").to_dynamic();
                frame.builder.insert_value(halves, second, 1, "").to_dynamic()
            }
            ExpressionKind::ArrayView(place) => {
                // trace: D379
                let (pointer, machine_type) = self.place_pointer(frame, place)?;
                let MachineType::Array { len, .. } = machine_type else {
                    return Err("a view of something that isn't an array".into());
                };
                let count = self.integer(64, len);
                let none = integer_constant(self.context, 64, -1);
                let mut record = self.list_type().undefined().to_dynamic();
                record = frame.builder.insert_value(record, pointer, 0, "").to_dynamic();
                record = frame.builder.insert_value(record, count, 1, "").to_dynamic();
                record = frame.builder.insert_value(record, count, 2, "").to_dynamic();
                record = frame.builder.insert_value(record, none, 3, "").to_dynamic();
                frame.builder.insert_value(record, none, 4, "").to_dynamic()
            }
            ExpressionKind::TextCopy(value) => {
                let text = self.expression(frame, value)?;
                let data = frame.builder.extract_value(text, 0, "").to_dynamic();
                let length = frame.builder.extract_value(text, 1, "").to_dynamic();
                frame.builder.call(self.runtime("__cyborg_text_copy")?, &[data, length], "").to_dynamic()
            }
            ExpressionKind::FitText { value, most, failure } => {
                // trace: D328
                let text = self.expression(frame, value)?;
                let length = frame.builder.extract_value(text, 1, "").to_dynamic();
                let fits = frame
                    .builder
                    .integer_compare(IntegerPredicate::UnsignedLessOrEqual, length, self.integer(64, *most), "")
                    .to_dynamic();
                self.crash_unless(frame, fits, failure, &[(length, MachineType::Integer { bits: 64, signed: false })])?;
                text
            }
            ExpressionKind::Pool { count } => {
                // trace: D345, D392
                let arguments = [self.integer(64, *count), self.integer(32, self.cores_name)];
                frame.builder.call(self.runtime("__cyborg_pool_new")?, &arguments, "").to_dynamic()
            }
            ExpressionKind::Heap => {
                frame.builder.call(self.runtime("__cyborg_heap")?, &[] as &[Value<'context, 'module>], "").to_dynamic()
            }
            ExpressionKind::ArenaNew { room } => {
                // trace: D330, D386
                let room = self.integer(64, *room);
                frame.builder.call(self.runtime("__cyborg_arena_new")?, &[room], "").to_dynamic()
            }
            ExpressionKind::ListFrom { items, growth } => {
                // trace: D369
                let MachineType::List(item) = &expression.machine_type else {
                    return Err("a list from something that isn't a list".into());
                };
                let list_type = self.list_type();
                let record = Self::temporary(frame, list_type);
                let empty = self.empty_list();
                frame.builder.store(empty, record);
                let item_type = self.machine_type(item);
                let size = self.context.size_of(item_type);
                let after = frame.function.append_basic_block("after_from");
                let want = self.integer(64, items.len() as u64);
                if !items.is_empty() {
                    self.grow_list(frame, record, growth, want, size, after)?;
                }
                for each in items {
                    let slot =
                        frame.builder.call(self.runtime("__cyborg_list_push")?, &[record, size], "").to_dynamic();
                    let value = self.expression(frame, each)?;
                    frame.builder.store(value, slot);
                }
                frame.builder.branch(after);
                frame.builder.position_at_end(after);
                frame.builder.load(list_type, record, "").to_dynamic()
            }
            ExpressionKind::Maths { function: crate::checked::MathsFunction::Sqrt, value } => {
                // trace: D405
                let argument = self.expression(frame, value)?;
                let intrinsic = Intrinsic::find("llvm.sqrt").ok_or("LLVM has no `llvm.sqrt`")?;
                let declaration = self.module.intrinsic_declaration(intrinsic, &[argument.get_type()]);
                frame.builder.call(declaration, &[argument], "").to_dynamic()
            }
            ExpressionKind::Maths { function, value } => {
                // trace: D256
                let argument = self.expression(frame, value)?;
                let double = self.context.get_type::<F64>().to_dynamic();
                let symbol = function.symbol();
                let callee = match self.module.get_function(symbol) {
                    Some(callee) => callee,
                    None => self.module.add_function(symbol, self.context.function_type(double, &[double], false)),
                };
                frame.builder.call(callee, &[argument], "").to_dynamic()
            }
            ExpressionKind::Clamp { value, min, max } => {
                let MachineType::Integer { bits, signed } = value.machine_type else {
                    return Err("a clamp on something that isn't an integer".to_string());
                };
                let generated = self.expression(frame, value)?;
                let (below, above) = if signed {
                    (IntegerPredicate::SignedLessThan, IntegerPredicate::SignedGreaterThan)
                } else {
                    (IntegerPredicate::UnsignedLessThan, IntegerPredicate::UnsignedGreaterThan)
                };
                let low = integer_constant(self.context, bits, *min);
                let high = integer_constant(self.context, bits, *max);
                let too_low = frame.builder.integer_compare(below, generated, low, "").to_dynamic();
                let raised = frame.builder.select(too_low, low, generated, "").to_dynamic();
                let too_high = frame.builder.integer_compare(above, raised, high, "").to_dynamic();
                frame.builder.select(too_high, high, raised, "").to_dynamic()
            }
            ExpressionKind::ClampFloat { value, min, max } => {
                use lyftar::enums::FloatPredicate;
                let MachineType::Float { bits } = value.machine_type else {
                    return Err("a float clamp on a non-float".into());
                };
                let generated = self.expression(frame, value)?;
                let constant = |number: f64| {
                    if bits == 32 {
                        self.context.get_type::<F32>().constant_float(number).to_dynamic()
                    } else {
                        self.context.get_type::<F64>().constant_float(number).to_dynamic()
                    }
                };
                let (low, high) = (constant(*min), constant(*max));
                let too_low =
                    frame.builder.float_compare(FloatPredicate::OrderedLessThan, generated, low, "").to_dynamic();
                let raised = frame.builder.select(too_low, low, generated, "").to_dynamic();
                let too_high =
                    frame.builder.float_compare(FloatPredicate::OrderedGreaterThan, raised, high, "").to_dynamic();
                frame.builder.select(too_high, high, raised, "").to_dynamic()
            }
            ExpressionKind::Zero => self.machine_type(&expression.machine_type).constant_null().to_dynamic(),
            ExpressionKind::AddressOf { local, path } => self.field_place(frame, *local, path)?,
            ExpressionKind::Load(place) => {
                let (pointer, machine_type) = self.place_pointer(frame, place)?;
                frame.builder.load(self.machine_type(&machine_type), pointer, "").to_dynamic()
            }
            ExpressionKind::PlaceAddress(place) => self.place_pointer(frame, place)?.0,
            ExpressionKind::ListLen(list) => {
                let list = self.expression(frame, list)?;
                frame.builder.extract_value(list, 1, "").to_dynamic()
            }
            ExpressionKind::ListOf(items) => {
                let MachineType::List(item) = &expression.machine_type else {
                    return Err("a list of something that isn't a list".into());
                };
                let item_type = self.machine_type(item);
                let count = self.integer(64, items.len() as u64);
                let storage = if items.is_empty() {
                    self.context.pointer_type(0).constant_null().to_dynamic()
                } else {
                    let size = self.context.size_of(item_type);
                    let bytes = frame.builder.integer_multiply(size, count, "").to_dynamic();
                    frame.builder.call(self.runtime("__cyborg_allocate")?, &[bytes], "").to_dynamic()
                };
                for (index, item) in items.iter().enumerate() {
                    let value = self.expression(frame, item)?;
                    let slot = frame
                        .builder
                        .get_element_pointer(item_type, storage, &[self.integer(64, index as u64)], "")
                        .to_dynamic();
                    frame.builder.store(value, slot);
                }
                let mut record = self.list_type().undefined().to_dynamic();
                record = frame.builder.insert_value(record, storage, 0, "").to_dynamic();
                record = frame.builder.insert_value(record, count, 1, "").to_dynamic();
                record = frame.builder.insert_value(record, count, 2, "").to_dynamic();
                // No allocator yet, and no block in one (D322).
                let none = integer_constant(self.context, 64, -1);
                record = frame.builder.insert_value(record, none, 3, "").to_dynamic();
                frame.builder.insert_value(record, none, 4, "").to_dynamic()
            }
            ExpressionKind::Fill { value, len } => {
                let value = self.expression(frame, value)?;
                let array_type = self.machine_type(&expression.machine_type);
                let mut built = array_type.undefined().to_dynamic();
                for index in 0..*len {
                    built =
                        frame.builder.insert_value(built, value, u32::try_from(index).unwrap_or(0), "").to_dynamic();
                }
                built
            }
            ExpressionKind::Handle { call, union, on_failure } => {
                self.handle(frame, call, *union, on_failure, &expression.machine_type)?
            }
            ExpressionKind::Build(fields) => {
                let mut built = self.machine_type(&expression.machine_type).undefined().to_dynamic();
                for (index, field) in fields.iter().enumerate() {
                    let value = self.expression(frame, field)?;
                    built =
                        frame.builder.insert_value(built, value, u32::try_from(index).unwrap_or(0), "").to_dynamic();
                }
                built
            }
            ExpressionKind::Field { base, index } => {
                let base = self.expression(frame, base)?;
                frame.builder.extract_value(base, u32::try_from(*index).unwrap_or(0), "").to_dynamic()
            }
            ExpressionKind::BuildCase { case, fields } => {
                let MachineType::Union(id) = expression.machine_type else {
                    return Err("a case of something that isn't a union".into());
                };
                let (union_type, case_types) = self.unions.get(id).cloned().ok_or("a union that wasn't made")?;
                let case_type = *case_types.get(*case).ok_or("a case that wasn't made")?;
                let mut built = case_type.undefined().to_dynamic();
                for (index, field) in fields.iter().enumerate() {
                    let value = self.expression(frame, field)?;
                    built =
                        frame.builder.insert_value(built, value, u32::try_from(index).unwrap_or(0), "").to_dynamic();
                }
                let slot = Self::temporary(frame, union_type);
                let tag_place = frame.builder.struct_get_element_pointer(union_type, slot, 0, "").to_dynamic();
                frame.builder.store(self.integer(32, *case as u64), tag_place);
                let payload = frame.builder.struct_get_element_pointer(union_type, slot, 1, "").to_dynamic();
                frame.builder.store(built, payload);
                frame.builder.load(union_type, slot, "").to_dynamic()
            }
            ExpressionKind::Tag(base) => {
                let base = self.expression(frame, base)?;
                frame.builder.extract_value(base, 0, "").to_dynamic()
            }
            ExpressionKind::CaseField { base, case, index } => {
                let MachineType::Union(id) = base.machine_type else {
                    return Err("a case field of something that isn't a union".into());
                };
                let (union_type, case_types) = self.unions.get(id).cloned().ok_or("a union that wasn't made")?;
                let case_type = *case_types.get(*case).ok_or("a case that wasn't made")?;
                let value = self.expression(frame, base)?;
                let slot = Self::temporary(frame, union_type);
                frame.builder.store(value, slot);
                let payload = frame.builder.struct_get_element_pointer(union_type, slot, 1, "").to_dynamic();
                let case_value = frame.builder.load(case_type, payload, "").to_dynamic();
                frame.builder.extract_value(case_value, u32::try_from(*index).unwrap_or(0), "").to_dynamic()
            }
            ExpressionKind::Bind { local, value, body } => {
                let generated = self.expression(frame, value)?;
                frame.builder.store(generated, frame.slots[*local]);
                self.expression(frame, body)?
            }
        };
        Ok(value)
    }

    /// A program tuned with `--machine.cpu=` asks for each instruction family it was built to use before anything
    /// else, and stops naming the first one missing (D365).
    fn machine_check(&mut self, frame: &mut Frame<'context, 'module>) -> Result<()> {
        // trace: D365
        let Some((cpu, features)) = crate::target::machine_cpu() else { return Ok(()) };
        let probes = crate::machine::probes(self.target, features);
        if probes.is_empty() {
            return Ok(());
        }
        // Cyborg spells a processor `apple_m5`, where LLVM says `apple-m5` (D420).
        let shown = cpu.replace('-', "_");
        for (feature, probe) in probes {
            let has = match probe {
                crate::machine::Probe::Sysctl(name) => {
                    let name = self.constant_bytes(&format!("hw.optional.arm.{name}\0"));
                    frame.builder.call(self.runtime("__cyborg_has_sysctl")?, &[name], "").to_dynamic()
                }
                crate::machine::Probe::Hwcap { second, bit } => {
                    let which = self.integer(64, if second { 26 } else { 16 });
                    let arguments = [which, self.integer(64, u64::from(bit))];
                    frame.builder.call(self.runtime("__cyborg_has_hwcap")?, &arguments, "").to_dynamic()
                }
                crate::machine::Probe::Cpuid { leaf, register, bit } => {
                    let arguments = [
                        self.integer(32, u64::from(leaf)),
                        self.integer(32, u64::from(register)),
                        self.integer(32, u64::from(bit)),
                    ];
                    frame.builder.call(self.runtime("__cyborg_has_cpuid")?, &arguments, "").to_dynamic()
                }
            };
            let fine = frame.function.append_basic_block("has_instructions");
            let missing = frame.function.append_basic_block("missing_instructions");
            frame.builder.conditional_branch(has, fine, missing);
            frame.builder.position_at_end(missing);
            // trace: D420
            let message = format!(
                "check failed: this processor is missing instructions the program was built for\n what: built with --machine.cpu={shown}, and this processor has no `{feature}` instructions\n why:  a program tuned for one processor needs its instructions (D365)\n fix:  rebuild for this machine with --machine.cpu=native, or without --machine.cpu=\n"
            );
            let arguments = [self.constant_bytes(&message), self.integer(64, message.len() as u64)];
            frame.builder.call(self.runtime("__cyborg_machine_missing")?, &arguments, "");
            frame.builder.unreachable();
            frame.builder.position_at_end(fine);
        }
        Ok(())
    }

    /// Starts a call on one of an OS pool's threads (D343, D345, D356). Its arguments are worked out here, where the
    /// spawn is written, into a record on the heap: the thread, the pool, room for what the call gives back, then
    /// each argument, with a temporary's value held in the record itself, so it lives as long as the task. A small
    /// function made for this spawn is what the thread runs: it unpacks the record, makes the call, stores what it
    /// gave back, and frees its thread for the next spawn.
    fn task_start(
        &mut self,
        frame: &mut Frame<'context, 'module>,
        task: usize,
        pool: &Expression,
        call: &Expression,
    ) -> Result<()> {
        // trace: D343, D345, D356
        let ExpressionKind::Call { function, arguments, .. } = &call.kind else {
            return Err("a task that isn't a call".into());
        };
        let callee = self.declare(*function)?;
        let pointer = self.context.pointer_type(0);
        let returns = call.machine_type != MachineType::Void;
        let result_type =
            if returns { self.machine_type(&call.machine_type) } else { self.context.integer_type(8).to_dynamic() };
        let pool = self.expression(frame, pool)?;
        // Each argument's value and type in the record, and whether the call takes the record's own place for it.
        let mut fields = Vec::with_capacity(arguments.len());
        for argument in arguments {
            fields.push(match argument {
                Argument::Value(value) => {
                    (self.expression(frame, value)?, self.machine_type(&value.machine_type), false)
                }
                Argument::Temporary(value) => {
                    (self.expression(frame, value)?, self.machine_type(&value.machine_type), true)
                }
                Argument::Address(local) => (frame.slots[*local], pointer, false),
                Argument::FieldAddress { local, path } => (self.field_place(frame, *local, path)?, pointer, false),
                Argument::Place(place) => (self.place_pointer(frame, place)?.0, pointer, false),
            });
        }
        let mut layout = vec![pointer, pointer, result_type];
        layout.extend(fields.iter().map(|(_, field_type, _)| *field_type));
        let record_type = self.context.struct_type(&layout, false);
        let size = self.context.size_of(record_type);
        let record = frame.builder.call(self.runtime("__cyborg_allocate")?, &[size], "").to_dynamic();
        let pool_place = frame.builder.struct_get_element_pointer(record_type, record, 1, "").to_dynamic();
        frame.builder.store(pool, pool_place);
        for (index, (value, _, _)) in fields.iter().enumerate() {
            let at = u32::try_from(index + 3).unwrap_or(u32::MAX);
            let place = frame.builder.struct_get_element_pointer(record_type, record, at, "").to_dynamic();
            frame.builder.store(*value, place);
        }
        // What the thread runs.
        let entry_type = self.context.function_type(pointer, &[pointer], false);
        let entry = self.module.add_function("cyborg.task", entry_type);
        entry.set_linkage(Linkage::Internal);
        let builder = Builder::at_end_of(entry.append_basic_block("entry"));
        let own = entry.parameter(0);
        let mut values = Vec::with_capacity(fields.len());
        for (index, (_, field_type, by_place)) in fields.iter().enumerate() {
            let at = u32::try_from(index + 3).unwrap_or(u32::MAX);
            let place = builder.struct_get_element_pointer(record_type, own, at, "").to_dynamic();
            values.push(if *by_place { place } else { builder.load(*field_type, place, "").to_dynamic() });
        }
        let made = builder.call(callee, &values, "").to_dynamic();
        if returns {
            let place = builder.struct_get_element_pointer(record_type, own, 2, "").to_dynamic();
            builder.store(made, place);
        }
        builder.call(self.runtime("__cyborg_task_end")?, &[own], "");
        builder.return_value(pointer.constant_null());
        frame.builder.call(self.runtime("__cyborg_task_start")?, &[record, entry.as_pointer()], "");
        frame.builder.store(record, frame.slots[task]);
        Ok(())
    }

    /// An atomic's operation, through the CPU's own indivisible instructions (D354, D360): `%load` is an atomic load,
    /// `%swap`, the `.wrap` and bit forms are one `atomicrmw`, `%compare_swap` is one `cmpxchg`, and a checked or
    /// saturating `%add` or `%sub` works out the new value and retries its `cmpxchg` until nothing changed it first.
    /// A `bool` is worked on as its byte, since atomics need whole bytes.
    #[allow(clippy::too_many_lines)]
    fn atomic(
        &mut self,
        frame: &mut Frame<'context, 'module>,
        address: &Expression,
        operation: &AtomicOperation,
        ordering: AtomicOrder,
        failure: Option<&CheckFailure>,
        result_type: &MachineType,
    ) -> Result<Value<'context, 'module>> {
        use lyftar::enums::{AtomicOrdering as Order, AtomicReadModifyWriteOperation as Rmw};
        // trace: D354, D360
        let order = match ordering {
            AtomicOrder::Relaxed => Order::Monotonic,
            AtomicOrder::Acquire => Order::Acquire,
            AtomicOrder::Release => Order::Release,
            AtomicOrder::AcquireRelease => Order::AcquireRelease,
            AtomicOrder::SequentiallyConsistent => Order::SequentiallyConsistent,
        };
        // A failed `cmpxchg` only reads, so it keeps the reading half of the ordering.
        let failed_order = match ordering {
            AtomicOrder::Relaxed | AtomicOrder::Release => Order::Monotonic,
            AtomicOrder::Acquire | AtomicOrder::AcquireRelease => Order::Acquire,
            AtomicOrder::SequentiallyConsistent => Order::SequentiallyConsistent,
        };
        let pointer = self.expression(frame, address)?;
        let item = match operation {
            AtomicOperation::Load => result_type.clone(),
            AtomicOperation::Swap(value)
            | AtomicOperation::Update { value, .. }
            | AtomicOperation::CompareSwap { value, .. } => value.machine_type.clone(),
        };
        let is_bool = item == MachineType::Bool;
        let stored = if is_bool { self.context.integer_type(8).to_dynamic() } else { self.machine_type(&item) };
        let natural = self.machine_type(&item);
        let widen = |frame: &Frame<'context, 'module>, value: Value<'context, 'module>| {
            if is_bool { frame.builder.zero_extend_or_bit_cast(value, stored, "").to_dynamic() } else { value }
        };
        let narrow = |frame: &Frame<'context, 'module>, value: Value<'context, 'module>| {
            if is_bool { frame.builder.truncate_or_bit_cast(value, natural, "").to_dynamic() } else { value }
        };
        let load = |frame: &Frame<'context, 'module>, order: Order| {
            let loaded = frame.builder.load(stored, pointer, "");
            if let Some(instruction) = loaded.as_kind::<lyftar::values::LoadInstruction>() {
                instruction.set_ordering(order);
            }
            loaded.to_dynamic()
        };
        let old = match operation {
            AtomicOperation::Load => load(frame, order),
            AtomicOperation::Swap(value) => {
                let value = self.expression(frame, value)?;
                let value = widen(frame, value);
                frame.builder.atomic_read_modify_write(Rmw::Exchange, pointer, value, order, false).to_dynamic()
            }
            AtomicOperation::CompareSwap { expect, value } => {
                let expect = self.expression(frame, expect)?;
                let expect = widen(frame, expect);
                let value = self.expression(frame, value)?;
                let value = widen(frame, value);
                let pair = frame
                    .builder
                    .atomic_compare_exchange(pointer, expect, value, order, failed_order, false)
                    .to_dynamic();
                frame.builder.extract_value(pair, 0, "").to_dynamic()
            }
            AtomicOperation::Update { operator, value } => {
                let value = self.expression(frame, value)?;
                let direct = match operator {
                    Arithmetic::WrappingAdd => Some(Rmw::Add),
                    Arithmetic::WrappingSubtract => Some(Rmw::Subtract),
                    Arithmetic::BitAnd => Some(Rmw::And),
                    Arithmetic::BitOr => Some(Rmw::Or),
                    Arithmetic::BitXor => Some(Rmw::ExclusiveOr),
                    _ => None,
                };
                if let Some(rmw) = direct {
                    let value = widen(frame, value);
                    frame.builder.atomic_read_modify_write(rmw, pointer, value, order, false).to_dynamic()
                } else {
                    let retry = frame.function.append_basic_block("atomic_retry");
                    let done = frame.function.append_basic_block("atomic_done");
                    frame.builder.branch(retry);
                    frame.builder.position_at_end(retry);
                    let seen = load(frame, Order::Monotonic);
                    let current = narrow(frame, seen);
                    let more: &[CheckFailure] = &[];
                    let changed = self.arithmetic(frame, *operator, current, value, &item, failure, more)?;
                    let changed = widen(frame, changed);
                    let pair = frame
                        .builder
                        .atomic_compare_exchange(pointer, seen, changed, order, failed_order, false)
                        .to_dynamic();
                    let swapped = frame.builder.extract_value(pair, 1, "").to_dynamic();
                    frame.builder.conditional_branch(swapped, done, retry);
                    frame.builder.position_at_end(done);
                    seen
                }
            }
        };
        if *result_type == MachineType::Void {
            return Ok(self.context.pointer_type(0).constant_null().to_dynamic());
        }
        Ok(narrow(frame, old))
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
        // A plain `+` that overflows wraps in `FinalPerf` and `FinalSmall`, and is undefined in `FinalShit` (D42).
        if let (Some(failure), Arithmetic::Add | Arithmetic::Subtract | Arithmetic::Multiply) = (failure, operator)
            && Self::decided(failure)
            && self.mode.overflow() != Overflow::Checked
        {
            let promise = self.mode.overflow() == Overflow::Undefined;
            let builder = &frame.builder;
            return Ok(match (operator, signed, promise) {
                (Arithmetic::Add, _, false) => builder.integer_add(left, right, "").to_dynamic(),
                (Arithmetic::Add, true, true) => builder.integer_add_no_signed_wrap(left, right, "").to_dynamic(),
                (Arithmetic::Add, false, true) => builder.integer_add_no_unsigned_wrap(left, right, "").to_dynamic(),
                (Arithmetic::Subtract, _, false) => builder.integer_subtract(left, right, "").to_dynamic(),
                (Arithmetic::Subtract, true, true) => {
                    builder.integer_subtract_no_signed_wrap(left, right, "").to_dynamic()
                }
                (Arithmetic::Subtract, false, true) => {
                    builder.integer_subtract_no_unsigned_wrap(left, right, "").to_dynamic()
                }
                (_, _, false) => builder.integer_multiply(left, right, "").to_dynamic(),
                (_, true, true) => builder.integer_multiply_no_signed_wrap(left, right, "").to_dynamic(),
                (_, false, true) => builder.integer_multiply_no_unsigned_wrap(left, right, "").to_dynamic(),
            });
        }
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
            Arithmetic::Subtract | Arithmetic::WrappingSubtract => {
                Ok(frame.builder.integer_subtract(left, right, "").to_dynamic())
            }
            Arithmetic::Multiply | Arithmetic::WrappingMultiply => {
                Ok(frame.builder.integer_multiply(left, right, "").to_dynamic())
            }
            Arithmetic::SaturatingAdd | Arithmetic::SaturatingSubtract | Arithmetic::SaturatingMultiply => {
                // trace: D294
                let name = match (operator, signed) {
                    (Arithmetic::SaturatingAdd, true) => "llvm.sadd.sat",
                    (Arithmetic::SaturatingAdd, false) => "llvm.uadd.sat",
                    (Arithmetic::SaturatingSubtract, true) => "llvm.ssub.sat",
                    (Arithmetic::SaturatingSubtract, false) => "llvm.usub.sat",
                    (_, true) => "llvm.smul.fix.sat",
                    (_, false) => "llvm.umul.fix.sat",
                };
                let intrinsic = Intrinsic::find(name).ok_or_else(|| format!("LLVM has no `{name}`"))?;
                let declaration = self.module.intrinsic_declaration(intrinsic, &[left.get_type()]);
                if operator == Arithmetic::SaturatingMultiply {
                    // A fixed-point multiply with no fraction bits is a whole-number one.
                    let scale = self.integer(32, 0);
                    return Ok(frame.builder.call(declaration, &[left, right, scale], "").to_dynamic());
                }
                Ok(frame.builder.call(declaration, &[left, right], "").to_dynamic())
            }
            Arithmetic::RotateLeft | Arithmetic::RotateRight => {
                // trace: D404
                let name = if operator == Arithmetic::RotateLeft { "llvm.fshl" } else { "llvm.fshr" };
                let intrinsic = Intrinsic::find(name).ok_or_else(|| format!("LLVM has no `{name}`"))?;
                let declaration = self.module.intrinsic_declaration(intrinsic, &[left.get_type()]);
                Ok(frame.builder.call(declaration, &[left, left, right], "").to_dynamic())
            }
            Arithmetic::BitAnd => Ok(frame.builder.and(left, right, "").to_dynamic()),
            Arithmetic::BitOr => Ok(frame.builder.or(left, right, "").to_dynamic()),
            Arithmetic::BitXor => Ok(frame.builder.exclusive_or(left, right, "").to_dynamic()),
        }
    }
}

impl<'context, 'module> Generator<'context, 'module> {
    /// A float's magnitude, through LLVM's `fabs`.
    fn float_magnitude(
        &mut self,
        frame: &mut Frame<'context, 'module>,
        value: Value<'context, 'module>,
        _bits: u32,
    ) -> Result<Value<'context, 'module>> {
        let intrinsic = Intrinsic::find("llvm.fabs").ok_or("LLVM has no `llvm.fabs`")?;
        let declaration = self.module.intrinsic_declaration(intrinsic, &[value.get_type()]);
        Ok(frame.builder.call(declaration, &[value], "").to_dynamic())
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

/// A conservative size, in 8-byte words, for a value of a machine type, so a union's room fits its largest case.
fn size_in_words(machine_type: &MachineType, program: &Program) -> u64 {
    match machine_type {
        MachineType::Integer { bits, .. } => u64::from(bits.div_ceil(64)).max(1),
        MachineType::Float { .. } | MachineType::Bool | MachineType::Void => 1,
        MachineType::Text => 2,
        MachineType::Writer => 6,
        MachineType::Address => 1,
        MachineType::Array { item, len } => size_in_words(item, program).saturating_mul(*len).max(1),
        MachineType::List(_) => 5,
        MachineType::Struct(id) => program
            .structs
            .get(*id)
            .map_or(1, |fields| fields.iter().map(|field| size_in_words(field, program)).sum::<u64>().max(1)),
        MachineType::Union(id) => {
            1 + program.unions.get(*id).map_or(1, |cases| {
                cases
                    .iter()
                    .map(|fields| fields.iter().map(|field| size_in_words(field, program)).sum::<u64>())
                    .max()
                    .unwrap_or(1)
            })
        }
    }
}
