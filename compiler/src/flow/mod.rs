//! The control-flow IR, and the ownership checks that run over it (D2, D129, D169, D185, D285).
//!
//! While checking, the checker records what each statement does to a variable's ownership as events, kept in the
//! checked program as `Statement::Flow`. Here each function is lowered to a graph of blocks of those events, and one
//! forward pass follows every path: what's been given away or finished, which `must_Give` duties are still open, and
//! which stored borrows were spoiled by a later use of what they borrow. The pass is small and plain on purpose, since
//! its lattice and transfer functions are what the Verus proofs cover (rule 13).

use std::collections::{BTreeMap, BTreeSet};

use crate::checked::{self, Block, ExpressionKind, LocalId, LoopId, OnFailure, Statement};
use crate::diagnostic::Diagnostic;
use crate::source::Span;

pub mod checks;
use checks::Kind;

const GIVEN_WHY: &str = "a given value is dead afterwards (D2)";
const DUTY_WHY: &str = "a `must_Give` value is handed off with `Give` exactly once, or `finish` ends it (D2, D129)";
const BORROW_WHY: &str =
    "a stored `Lend.mut` means nothing else uses it meanwhile, and a `Lend` means it can't change (D2)";
const HELD_WHY: &str = "`.held` means nobody touches what it borrows for the whole block (D169)";
const PARTIAL_WHY: &str = "after a field is moved out, the value is incomplete (D185)";

/// A variable, or a field inside one.
#[derive(Clone, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct Place {
    pub local: LocalId,
    pub path: Vec<usize>,
    /// As written, `$l.file`, for messages.
    pub shown: String,
}

/// How a variable came to hold its value, which decides whether it carries a duty and how errors name it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Origin {
    Local,
    Given,
    Lent,
    LentMut,
}

/// Where an event happened: the variable as written, the whole place, and the call or hand-off around it.
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct Spot {
    /// `$b`.
    pub variable: Span,
    /// `$b.n`.
    pub place: Span,
    /// The call around it, or the variable when there's none: where a use is said to be.
    pub site: Span,
    /// The call around it, or the whole hand-off, such as `Give $s`: where a hand-off is said to be.
    pub marker: Span,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum Event {
    /// A variable gets a value.
    Bind {
        local: LocalId,
        name: String,
        declared: Span,
        must_give: bool,
        origin: Origin,
        /// Why its duty matters, said when it's left open, for a value whose kind has its own reason.
        why: Option<String>,
    },
    /// Reading it, or a `Copy` or `Lend` of it.
    Read {
        place: Place,
        spot: Spot,
    },
    Give {
        place: Place,
        spot: Spot,
    },
    /// `Lend.mut`, or a `%%` receiver.
    LendMut {
        place: Place,
        spot: Spot,
    },
    Change {
        place: Place,
        spot: Spot,
    },
    /// `finish`, with the `must_Give` fields inside that have to be moved out first (D177, D185).
    Finish {
        local: LocalId,
        spot: Spot,
        duties: Vec<Place>,
    },
    /// A stored borrow made in `borrower`, from `sources`.
    Borrow {
        borrower: LocalId,
        /// Each source, with the field it borrows, or none for all of it (D173).
        sources: Vec<(LocalId, Vec<usize>)>,
        mutable: bool,
        held: bool,
    },
    /// A variable's round ends with it: a duty still open is never done (D180).
    Expire {
        local: LocalId,
    },
    /// A test on a union's case starts being trusted (D9, D104).
    Trust {
        local: LocalId,
        what: String,
    },
    /// A read that relies on that trust.
    TrustedRead {
        local: LocalId,
        spot: Spot,
        what: String,
    },
    /// What only happens on some paths, like the right side of `and`.
    Maybe(Vec<Event>),
    /// The block that declared these ends.
    ScopeEnd {
        locals: Vec<LocalId>,
    },
    /// Leaving the function, by `return` or `fail`, or leaving a loop's variables behind by `break`.
    Leave {
        span: Span,
        locals: Option<Vec<LocalId>>,
    },
}

/// A block of the graph: its events, and the blocks it can go to next.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Node {
    pub events: Vec<Event>,
    pub next: Vec<usize>,
}

/// One function as a control-flow graph. Block 0 is the start.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Graph {
    pub nodes: Vec<Node>,
}

/// Lowers a function's checked body to its graph.
#[must_use]
pub fn lower(body: &Block) -> Graph {
    let mut lowering = Lowering { graph: Graph::default(), loops: Vec::new(), instead: Vec::new() };
    let start = lowering.node();
    lowering.block(body, Some(start));
    lowering.graph
}

struct Lowering {
    graph: Graph,
    /// Each open loop, with the block after it.
    loops: Vec<(LoopId, usize)>,
    /// Where `instead` goes, for each open set of arms.
    instead: Vec<usize>,
}

impl Lowering {
    fn node(&mut self) -> usize {
        self.graph.nodes.push(Node::default());
        self.graph.nodes.len() - 1
    }

    fn edge(&mut self, from: usize, to: usize) {
        if let Some(node) = self.graph.nodes.get_mut(from)
            && !node.next.contains(&to)
        {
            node.next.push(to);
        }
    }

    /// Lowers statements from `current`, giving the block where they end, or `None` once every path has left.
    fn block(&mut self, statements: &Block, mut current: Option<usize>) -> Option<usize> {
        for statement in statements {
            // Code after a `return` or `break` still gets a block, unreachable, so nothing in it is lost.
            let here = match current {
                Some(here) => here,
                None => self.node(),
            };
            current = self.statement(statement, here);
        }
        current
    }

    fn statement(&mut self, statement: &Statement, here: usize) -> Option<usize> {
        match statement {
            Statement::Flow(events) => Some(self.events(events, here)),
            Statement::If { condition, then, otherwise } => {
                let here = self.expression(condition, here);
                let then_start = self.node();
                let otherwise_start = self.node();
                self.edge(here, then_start);
                self.edge(here, otherwise_start);
                let then_end = self.block(then, Some(then_start));
                let otherwise_end = self.block(otherwise, Some(otherwise_start));
                self.join(&[then_end, otherwise_end])
            }
            Statement::Loop { id, body, .. } => {
                let head = self.node();
                let after = self.node();
                self.edge(here, head);
                self.loops.push((*id, after));
                if let Some(end) = self.block(body, Some(head)) {
                    self.edge(end, head);
                }
                self.loops.pop();
                Some(after)
            }
            Statement::Break(id) => {
                if let Some(&(_, after)) = self.loops.iter().rev().find(|(each, _)| each == id) {
                    self.edge(here, after);
                }
                None
            }
            Statement::Return(value) => {
                if let Some(value) = value {
                    self.expression(value, here);
                }
                None
            }
            Statement::Instead { value, .. } => {
                let here = self.expression(value, here);
                if let Some(&after) = self.instead.last() {
                    self.edge(here, after);
                }
                None
            }
            Statement::Store { place, value } | Statement::Push { place, value, .. } => {
                let mut here = here;
                for step in &place.steps {
                    if let checked::Step::Item { position, .. } = step {
                        here = self.expression(position, here);
                    }
                }
                Some(self.expression(value, here))
            }
            Statement::Let { value, .. }
            | Statement::AliasValue { value, .. }
            | Statement::Set { value, .. }
            | Statement::SetField { value, .. }
            | Statement::Evaluate(value) => Some(self.expression(value, here)),
            Statement::Check { condition, .. } | Statement::CheckAtSite { condition, .. } => {
                Some(self.expression(condition, here))
            }
            Statement::Assembly { reads, .. } => {
                let mut here = here;
                for (_, value) in reads {
                    here = self.expression(value, here);
                }
                Some(here)
            }
            Statement::ListDone { allocator: value, .. }
            | Statement::ArenaDone { arena: value, .. }
            | Statement::TextDone(value) => Some(self.expression(value, here)),
            Statement::OpenWriter { .. }
            | Statement::Alias { .. }
            | Statement::Print { .. }
            | Statement::Flush { .. }
            | Statement::LockAll { .. }
            | Statement::UnlockAll { .. }
            | Statement::Done { .. } => Some(here),
        }
    }

    /// Adds events to the block, branching around those that only happen on some paths.
    fn events(&mut self, events: &[Event], mut here: usize) -> usize {
        for event in events {
            if let Event::Maybe(inner) = event {
                let branch = self.node();
                let after = self.node();
                self.edge(here, branch);
                self.edge(here, after);
                let end = self.events(inner, branch);
                self.edge(end, after);
                here = after;
            } else if let Some(node) = self.graph.nodes.get_mut(here) {
                node.events.push(event.clone());
            }
        }
        here
    }

    fn join(&mut self, ends: &[Option<usize>]) -> Option<usize> {
        let live: Vec<usize> = ends.iter().flatten().copied().collect();
        if live.is_empty() {
            return None;
        }
        let after = self.node();
        for end in live {
            self.edge(end, after);
        }
        Some(after)
    }

    /// The `otherwise:` arms inside an expression branch off and come back after it.
    fn expression(&mut self, expression: &checked::Expression, here: usize) -> usize {
        let mut here = here;
        for inner in children(expression) {
            here = self.expression(inner, here);
        }
        if let ExpressionKind::Handle { on_failure, .. } = &expression.kind
            && let OnFailure::Arms { arms, .. } = on_failure.as_ref()
        {
            let after = self.node();
            self.edge(here, after);
            self.instead.push(after);
            for (_, arm) in arms {
                let start = self.node();
                self.edge(here, start);
                if let Some(end) = self.block(arm, Some(start)) {
                    self.edge(end, after);
                }
            }
            self.instead.pop();
            here = after;
        }
        // What `?` runs as it leaves branches off and never comes back (D325).
        if let ExpressionKind::Handle { on_failure, .. } = &expression.kind
            && let OnFailure::PassUp { cleanup, .. } = on_failure.as_ref()
            && !cleanup.is_empty()
        {
            let start = self.node();
            self.edge(here, start);
            self.block(cleanup, Some(start));
        }
        here
    }
}

/// The expressions directly inside one.
fn children(expression: &checked::Expression) -> Vec<&checked::Expression> {
    use ExpressionKind as Kind;
    match &expression.kind {
        Kind::Arithmetic { left, right, .. }
        | Kind::FloatArithmetic { left, right, .. }
        | Kind::Compare { left, right, .. }
        | Kind::And(left, right)
        | Kind::Or(left, right) => vec![left, right],
        Kind::Negate { operand: inner, .. }
        | Kind::Not(inner)
        | Kind::BitNot(inner)
        | Kind::TextOf(inner)
        | Kind::Tag(inner)
        | Kind::Convert(inner)
        | Kind::Field { base: inner, .. }
        | Kind::CaseField { base: inner, .. }
        | Kind::FromIeee { value: inner, .. }
        | Kind::FitFloat { value: inner, .. }
        | Kind::Fit { value: inner, .. }
        | Kind::Clamp { value: inner, .. }
        | Kind::ClampFloat { value: inner, .. }
        | Kind::Maths { value: inner, .. }
        | Kind::TextCopy(inner)
        | Kind::AddressAt(inner)
        | Kind::UnionBits(inner)
        | Kind::Bits(inner)
        | Kind::PointerRead { pointer: inner, .. }
        | Kind::FitText { value: inner, .. } => vec![inner],
        Kind::Handle { call, .. } => vec![call],
        Kind::Call { arguments, .. } => arguments
            .iter()
            .filter_map(|argument| match argument {
                checked::Argument::Value(value) | checked::Argument::Temporary(value) => Some(value),
                _ => None,
            })
            .collect(),
        Kind::Build(fields) | Kind::BuildCase { fields, .. } => fields.iter().collect(),
        Kind::Bind { value, body, .. } => vec![value, body],
        Kind::Select { condition, then, otherwise } => vec![condition, then, otherwise],
        Kind::Atomic { address, operation, .. } => match operation {
            checked::AtomicOperation::Load => vec![address],
            checked::AtomicOperation::Swap(value) | checked::AtomicOperation::Update { value, .. } => {
                vec![address, value]
            }
            checked::AtomicOperation::CompareSwap { expect, value } => vec![address, expect, value],
        },
        _ => Vec::new(),
    }
}

/// What a variable holds on the paths reaching a point.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum Holding {
    Alive,
    /// Given away or finished, on every path, or on some when `maybe`.
    Gone {
        finished: bool,
        at: Span,
        maybe: bool,
    },
    /// Fields were moved out (D185): each one's path, where, and whether on every path.
    Partial {
        moved: Vec<(Vec<usize>, Span, bool)>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct Loan {
    sources: Vec<(LocalId, Vec<usize>)>,
    mutable: bool,
    held: bool,
    /// Where something used what it borrows, after which it can't be used.
    spoiled: Option<Span>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct State {
    holding: BTreeMap<LocalId, Holding>,
    loans: BTreeMap<LocalId, Loan>,
    /// Each variable a test trusts, with where a `change` or `Lend.mut` ended that, on some path (D104).
    trusts: BTreeMap<LocalId, Option<Span>>,
}

/// What's known about each variable that doesn't change along a path.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct Variable {
    name: String,
    declared: Span,
    must_give: bool,
    origin: Origin,
    why: Option<String>,
}

impl Holding {
    /// What the proven checks decide on.
    fn kind(&self) -> Kind {
        match self {
            Holding::Alive => Kind::Alive,
            Holding::Partial { .. } => Kind::Partial,
            Holding::Gone { maybe: true, .. } => Kind::MaybeGone,
            Holding::Gone { maybe: false, .. } => Kind::Gone,
        }
    }
}

/// Where two sets of paths meet. Which kind comes out is `checks::join`'s, which is proven never to lose a path that
/// gave the value away; this only carries the spans that messages point at.
fn join_holding(a: &Holding, b: &Holding) -> Holding {
    let kind = checks::join(a.kind(), b.kind());
    let gone = [a, b].into_iter().find_map(|holding| match holding {
        Holding::Gone { finished, at, .. } => Some((*finished, *at)),
        _ => None,
    });
    match (kind, gone) {
        (Kind::Gone, Some((finished, at))) => Holding::Gone { finished, at, maybe: false },
        (Kind::MaybeGone, Some((finished, at))) => Holding::Gone { finished, at, maybe: true },
        (Kind::Partial, _) => {
            let mine = match a {
                Holding::Partial { moved } => moved.clone(),
                _ => Vec::new(),
            };
            let theirs = match b {
                Holding::Partial { moved } => moved.clone(),
                _ => Vec::new(),
            };
            // A field counts as moved out on every path only if both sides moved it on every path.
            let mut moved: Vec<(Vec<usize>, Span, bool)> = mine
                .iter()
                .map(|(path, at, every)| {
                    let both = theirs.iter().any(|(other, _, also)| other == path && *also);
                    (path.clone(), *at, *every && both)
                })
                .collect();
            for (path, at, _) in theirs {
                if !moved.iter().any(|(other, _, _)| *other == path) {
                    moved.push((path, at, false));
                }
            }
            Holding::Partial { moved }
        }
        _ => Holding::Alive,
    }
}

fn join(a: &State, b: &State) -> State {
    let mut joined = a.clone();
    for (local, holding) in &b.holding {
        let merged = match a.holding.get(local) {
            Some(mine) => join_holding(mine, holding),
            None => holding.clone(),
        };
        joined.holding.insert(*local, merged);
    }
    for (local, loan) in &b.loans {
        match joined.loans.get_mut(local) {
            Some(mine) => {
                if checks::join_spoiled(mine.spoiled.is_some(), loan.spoiled.is_some()) && mine.spoiled.is_none() {
                    mine.spoiled = loan.spoiled;
                }
            }
            None => {
                joined.loans.insert(*local, loan.clone());
            }
        }
    }
    for (local, ended) in &b.trusts {
        match joined.trusts.get_mut(local) {
            Some(mine) => {
                if checks::join_spoiled(mine.is_some(), ended.is_some()) && mine.is_none() {
                    *mine = *ended;
                }
            }
            None => {
                joined.trusts.insert(*local, *ended);
            }
        }
    }
    joined
}

/// Runs the ownership checks over a function's graph.
#[must_use]
pub fn check(graph: &Graph) -> Vec<Diagnostic> {
    let mut variables = BTreeMap::new();
    for node in &graph.nodes {
        for event in &node.events {
            if let Event::Bind { local, name, declared, must_give, origin, why } = event {
                variables.insert(
                    *local,
                    Variable {
                        name: name.clone(),
                        declared: *declared,
                        must_give: *must_give,
                        origin: *origin,
                        why: why.clone(),
                    },
                );
            }
        }
    }
    // Every block's state on entry, found by going round until nothing changes. The lattice is finite, since every
    // span in it comes from the function, so this ends.
    let mut entry: Vec<Option<State>> = vec![None; graph.nodes.len()];
    if !graph.nodes.is_empty() {
        entry[0] = Some(State::default());
    }
    let mut work: BTreeSet<usize> = BTreeSet::from([0]);
    let mut quiet = Checker { variables: &variables, errors: Vec::new(), report: false };
    while let Some(index) = work.pop_first() {
        let Some(state) = entry[index].clone() else { continue };
        let out = quiet.run(&graph.nodes[index].events, state);
        for &next in &graph.nodes[index].next {
            let merged = match &entry[next] {
                Some(old) => join(old, &out),
                None => out.clone(),
            };
            if entry[next].as_ref() != Some(&merged) {
                entry[next] = Some(merged);
                work.insert(next);
            }
        }
    }
    let mut checker = Checker { variables: &variables, errors: Vec::new(), report: true };
    for (index, node) in graph.nodes.iter().enumerate() {
        if let Some(state) = entry[index].clone() {
            checker.run(&node.events, state);
        }
    }
    let mut seen = BTreeSet::new();
    checker.errors.retain(|error| seen.insert((error.span.start, error.span.end, error.what.clone())));
    checker.errors
}

struct Checker<'a> {
    variables: &'a BTreeMap<LocalId, Variable>,
    errors: Vec<Diagnostic>,
    report: bool,
}

impl Checker<'_> {
    fn name(&self, local: LocalId) -> String {
        self.variables.get(&local).map_or_else(String::new, |variable| variable.name.clone())
    }

    fn say(&mut self, diagnostic: Diagnostic) {
        if self.report {
            self.errors.push(diagnostic);
        }
    }

    fn run(&mut self, events: &[Event], mut state: State) -> State {
        for event in events {
            self.event(event, &mut state);
        }
        state
    }

    fn event(&mut self, event: &Event, state: &mut State) {
        match event {
            Event::Bind { local, .. } => {
                state.holding.insert(*local, Holding::Alive);
            }
            Event::Read { place, spot } => {
                if self.usable(state, place, spot, false) {
                    self.spoil(state, place, spot, true);
                }
            }
            Event::LendMut { place, spot } | Event::Change { place, spot } => {
                if self.usable(state, place, spot, false) {
                    self.spoil(state, place, spot, false);
                }
                // A change ends a test's trust, said at the whole statement, or at the call that lends it (D104).
                let at = if matches!(event, Event::Change { .. }) { spot.marker } else { spot.site };
                if let Some(trust) = state.trusts.get_mut(&place.local)
                    && trust.is_none()
                {
                    *trust = Some(at);
                }
            }
            Event::Give { place, spot } if !place.path.is_empty() => self.move_out(state, place, spot),
            Event::Give { place, spot } => {
                if self.usable(state, place, spot, false) {
                    self.spoil(state, place, spot, false);
                    state.holding.insert(place.local, Holding::Gone { finished: false, at: spot.marker, maybe: false });
                }
            }
            Event::Finish { local, spot, duties } => {
                let place = Place { local: *local, ..Place::default() };
                if !self.usable(state, &place, spot, true) {
                    return;
                }
                // Every `must_Give` field inside has to be moved out first, on every path (D177, D185).
                let moved = match state.holding.get(local) {
                    Some(Holding::Partial { moved }) => moved.clone(),
                    _ => Vec::new(),
                };
                if let Some(waiting) =
                    duties.iter().find(|duty| !moved.iter().any(|(path, _, every)| *path == duty.path && *every))
                {
                    self.say(Diagnostic::refused(
                        spot.variable,
                        format!("`{}` hasn't been given away, so `finish` waits for it", waiting.shown),
                        "`finish` ends a value's own duty, after every `must_Give` value inside it is given away (D129, D177)",
                    ));
                    return;
                }
                self.spoil(state, &place, spot, false);
                state.holding.insert(*local, Holding::Gone { finished: true, at: spot.site, maybe: false });
            }
            Event::Borrow { borrower, sources, mutable, held } => {
                state.loans.insert(
                    *borrower,
                    Loan { sources: sources.clone(), mutable: *mutable, held: *held, spoiled: None },
                );
            }
            Event::ScopeEnd { locals } => {
                for local in locals {
                    state.loans.remove(local);
                }
            }
            Event::Leave { span, locals } => {
                self.leave(state, *span, locals.as_deref());
                // What a `break` or `instead` leaves behind is gone after it.
                for local in locals.iter().flatten() {
                    state.holding.remove(local);
                }
            }
            Event::Maybe(_) => {}
            Event::Expire { local } => {
                let holding = state.holding.get(local).cloned();
                let variable = self.variables.get(local).cloned();
                if let (Some(holding), Some(variable)) = (holding, variable)
                    && variable.must_give
                    && checks::duty_open(holding.kind())
                {
                    let name = &variable.name;
                    match holding {
                        Holding::Gone { at, .. } => self.say(
                            Diagnostic::refused(
                                variable.declared,
                                format!("`{name}` isn't given away or finished on every path"),
                                variable.why.clone().unwrap_or_else(|| DUTY_WHY.to_string()),
                            )
                            .with_because(at, format!("`{name}` went here, on only one path")),
                        ),
                        _ => self.say(Diagnostic::refused(
                            variable.declared,
                            format!("`{name}` is never given away or finished"),
                            variable.why.clone().unwrap_or_else(|| DUTY_WHY.to_string()),
                        )),
                    }
                }
                state.holding.remove(local);
            }
            Event::Trust { local, .. } => {
                state.trusts.insert(*local, None);
            }
            Event::TrustedRead { local, spot, what } => {
                if let Some(Some(at)) = state.trusts.get(local) {
                    let name = self.name(*local);
                    self.say(
                        Diagnostic::refused(
                            spot.place,
                            format!("`{name}` may not be `{what}` any more"),
                            "a test's trust ends at a `change`, and at a `Lend.mut` too (D104)",
                        )
                        .with_because(*at, format!("`{name}` changed here")),
                    );
                }
            }
        }
    }

    /// Whether the place can be used here, saying why not if it can't.
    fn usable(&mut self, state: &State, place: &Place, spot: &Spot, finishing: bool) -> bool {
        let name = self.name(place.local);
        if let Some(loan) = state.loans.get(&place.local)
            && let Some(at) = loan.spoiled
        {
            self.say(
                Diagnostic::refused(
                    spot.variable,
                    format!("`{name}` can't be used after what it borrows was used"),
                    BORROW_WHY,
                )
                .with_because(at, "what it borrows was used here"),
            );
            return false;
        }
        // Whether it's usable is `checks::whole_usable`'s call, proven to refuse anything given away on any path; this
        // only picks the words.
        let holding = state.holding.get(&place.local).cloned().unwrap_or(Holding::Alive);
        if checks::whole_usable(holding.kind()) || (finishing && holding.kind() == Kind::Partial) {
            return true;
        }
        match Some(&holding) {
            Some(Holding::Gone { finished: false, at, .. }) => {
                let span = spot.site;
                self.say(
                    Diagnostic::refused(span, format!("`{name}` was already given away"), GIVEN_WHY)
                        .with_because(*at, format!("`{name}` was given away here")),
                );
                false
            }
            Some(Holding::Gone { finished: true, at, .. }) => {
                if finishing {
                    self.say(
                        Diagnostic::refused(
                            spot.site,
                            format!("`{name}` was already finished"),
                            "a `must_Give` duty ends once (D2, D129)",
                        )
                        .with_because(*at, format!("`{name}` was finished here")),
                    );
                } else {
                    self.say(Diagnostic::not_designed(
                        spot.site,
                        format!("using `{name}` after `finish`"),
                        "not decided yet, and the standard doesn't say",
                    ));
                }
                false
            }
            Some(Holding::Partial { moved }) if !finishing => {
                let at = moved.first().map_or(spot.site, |(_, at, _)| *at);
                self.say(
                    Diagnostic::refused(
                        spot.variable,
                        format!(
                            "a field of `{name}` was moved out, so all that's left is moving out another, or `finish`"
                        ),
                        PARTIAL_WHY,
                    )
                    .with_because(at, "it was moved out here"),
                );
                false
            }
            _ => true,
        }
    }

    /// `Give $l.file`: moving one field out, which leaves the rest incomplete (D185).
    fn move_out(&mut self, state: &mut State, place: &Place, spot: &Spot) {
        let mut moved = match state.holding.get(&place.local) {
            Some(Holding::Partial { moved }) => moved.clone(),
            Some(Holding::Alive) | None => Vec::new(),
            Some(Holding::Gone { .. }) => {
                self.usable(state, place, spot, false);
                return;
            }
        };
        if let Some((_, at, _)) = moved.iter().find(|(path, _, _)| *path == place.path) {
            self.say(
                Diagnostic::refused(
                    spot.place,
                    format!("`{}` was already moved out", place.shown),
                    "a given value is dead afterwards (D2, D185)",
                )
                .with_because(*at, "it was moved out here"),
            );
            return;
        }
        if state.loans.get(&place.local).is_some_and(|loan| loan.spoiled.is_some()) {
            self.usable(state, place, spot, false);
            return;
        }
        self.spoil(state, place, spot, false);
        moved.push((place.path.clone(), spot.marker, true));
        state.holding.insert(place.local, Holding::Partial { moved });
    }

    /// A use of `local` spoils the stored borrows of it that it conflicts with: any use, for one lent to change,
    /// and anything but a read, for one lent to read.
    fn spoil(&mut self, state: &mut State, place: &Place, spot: &Spot, reading: bool) {
        let local = place.local;
        // Different fields are separate, so only a use that overlaps the borrowed field spoils it (D173).
        let overlaps = |(source, path): &(LocalId, Vec<usize>)| *source == local && checks::overlaps(path, &place.path);
        let borrowers: Vec<LocalId> = state
            .loans
            .iter()
            .filter(|(_, loan)| {
                loan.sources.iter().any(overlaps) && loan.spoiled.is_none() && checks::spoils(loan.mutable, reading)
            })
            .map(|(borrower, _)| *borrower)
            .collect();
        for borrower in borrowers {
            let Some(loan) = state.loans.get_mut(&borrower) else { continue };
            if loan.held {
                let holder = self.name(borrower);
                let source = self.name(local);
                let declared = self.variables.get(&borrower).map_or(spot.place, |variable| variable.declared);
                self.say(
                    Diagnostic::refused(
                        spot.place,
                        format!("`{holder}` holds `{source}` until the end of its block"),
                        HELD_WHY,
                    )
                    .with_because(declared, format!("`{holder}` holds it from here")),
                );
            } else {
                loan.spoiled = Some(spot.place);
            }
        }
    }

    /// Leaving the function: every `must_Give` duty has to be done on every path.
    fn leave(&mut self, state: &State, span: Span, locals: Option<&[LocalId]>) {
        for (local, variable) in self.variables {
            if !variable.must_give || !matches!(variable.origin, Origin::Local | Origin::Given) {
                continue;
            }
            if locals.is_some_and(|locals| !locals.contains(local)) {
                continue;
            }
            let name = &variable.name;
            // One a `break` or `instead` left behind was checked there.
            let Some(holding) = state.holding.get(local).cloned() else { continue };
            // Whether the duty is open is `checks::duty_open`'s call, proven to catch it open on any path.
            if !checks::duty_open(holding.kind()) {
                continue;
            }
            match Some(&holding) {
                Some(Holding::Alive | Holding::Partial { .. }) => {
                    let note = if variable.origin == Origin::Given {
                        format!("`{name}` was given to this function here")
                    } else {
                        format!("`{name}` was made here")
                    };
                    self.say(
                        Diagnostic::refused(
                            span,
                            format!("this leaves with `{name}` never given away or finished"),
                            variable.why.clone().unwrap_or_else(|| DUTY_WHY.to_string()),
                        )
                        .with_because(variable.declared, note),
                    );
                }
                Some(Holding::Gone { at, maybe: true, .. }) => {
                    self.say(
                        Diagnostic::refused(
                            variable.declared,
                            format!("`{name}` isn't given away or finished on every path"),
                            DUTY_WHY,
                        )
                        .with_because(*at, format!("`{name}` went here, on only one path")),
                    );
                }
                _ => {}
            }
        }
    }
}
