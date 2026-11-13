//! Units (D99, D100, D101, D109, D112, D242): families of units, each a `base` or a multiple of another, or made of
//! others with `per` and `x`. Every unit expands to an exact scale times powers of bases, which is what decides
//! whether two units are the same and how `as` converts between them.

use std::collections::HashMap;

use crate::diagnostic::Diagnostic;
use crate::source::Span;
use crate::syntax;

use super::types::Unit;
use super::{Check, Checker, NOT_DECIDED, Reported};

const LOOP_WHY: &str = "loops and units with no base are refused (D109)";

/// An exact ratio, kept in lowest terms with a positive bottom.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Ratio {
    pub top: i128,
    pub bottom: i128,
}

impl Ratio {
    pub const ONE: Ratio = Ratio { top: 1, bottom: 1 };

    #[must_use]
    pub fn new(top: i128, bottom: i128) -> Option<Ratio> {
        if bottom == 0 {
            return None;
        }
        let divisor = gcd(top.unsigned_abs(), bottom.unsigned_abs()).max(1);
        let divisor = i128::try_from(divisor).ok()?;
        let sign = if bottom < 0 { -1 } else { 1 };
        Some(Ratio { top: sign * top / divisor, bottom: sign * bottom / divisor })
    }

    #[must_use]
    pub fn times(self, other: Ratio) -> Option<Ratio> {
        Ratio::new(self.top.checked_mul(other.top)?, self.bottom.checked_mul(other.bottom)?)
    }

    #[must_use]
    pub fn power(self, power: i32) -> Option<Ratio> {
        let mut result = Ratio::ONE;
        for _ in 0..power.unsigned_abs() {
            result = result.times(self)?;
        }
        if power < 0 { Ratio::new(result.bottom, result.top) } else { Some(result) }
    }
}

fn gcd(a: u128, b: u128) -> u128 {
    if b == 0 { a } else { gcd(b, a % b) }
}

/// What a unit is, as declared.
#[derive(Clone, Debug)]
enum Definition {
    Base,
    Multiple {
        ratio: Ratio,
        of: String,
        of_span: Span,
    },
    Made(Vec<(String, i32, Span)>),
    /// The same size as another unit, with its zero moved: `celsius = kelvin offset ~-273.15` (D250).
    Offset {
        of: String,
        of_span: Span,
        offset: f64,
    },
    /// Refused or not built, so nothing more is said about it.
    Broken,
}

#[derive(Clone, Debug)]
struct UnitInfo {
    family: String,
    span: Span,
    definition: Definition,
    /// Whether its family says whether each value is a point or a span: one with an offset, or `std`'s `time`
    /// (D249).
    split: bool,
    /// Where its zero is, in its own unit, from its base's zero: -273.15 for celsius.
    offset: f64,
}

/// A unit as an exact scale times powers of bases, and of π for angles: `length.kilometers` is 1000 times
/// `length.meters`, and `angle.degrees` is π/180 of `angle.radians`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expansion {
    pub scale: Ratio,
    pub pi: i32,
    pub powers: Vec<(String, i32)>,
}

/// `.point` or `.span` at the end of a unit's name, split off (D249).
#[must_use]
pub fn mark_of(name: &str) -> (&str, Option<&str>) {
    for mark in ["point", "span"] {
        if let Some(base) = name.strip_suffix(&format!(".{mark}")) {
            return (base, Some(mark));
        }
    }
    (name, None)
}

/// Every declared unit, with `items`, which is base language (D101).
pub struct Units {
    infos: HashMap<String, UnitInfo>,
    order: Vec<String>,
    expansions: HashMap<String, Option<Expansion>>,
}

impl Default for Units {
    fn default() -> Units {
        let mut units = Units { infos: HashMap::new(), order: Vec::new(), expansions: HashMap::new() };
        // `items` is base language (D101), and `std` has `time`, whose values are points or spans (D249), and
        // `angle`, with radians as its base (standard 2.16).
        let builtin: [(&str, &str, bool, Ratio, i32, &str); 5] = [
            ("items", "", false, Ratio::ONE, 0, "items"),
            ("std::units::time.seconds", "std::units::time", true, Ratio::ONE, 0, "std::units::time.seconds"),
            ("std::units::angle.radians", "std::units::angle", false, Ratio::ONE, 0, "std::units::angle.radians"),
            (
                "std::units::angle.degrees",
                "std::units::angle",
                false,
                Ratio { top: 1, bottom: 180 },
                1,
                "std::units::angle.radians",
            ),
            (
                "std::units::angle.turns",
                "std::units::angle",
                false,
                Ratio { top: 2, bottom: 1 },
                1,
                "std::units::angle.radians",
            ),
        ];
        for (name, family, split, scale, pi, base) in builtin {
            units.infos.insert(
                name.to_string(),
                UnitInfo {
                    family: family.to_string(),
                    span: Span::default(),
                    definition: Definition::Base,
                    split,
                    offset: 0.0,
                },
            );
            units
                .expansions
                .insert(name.to_string(), Some(Expansion { scale, pi, powers: vec![(base.to_string(), 1)] }));
        }
        units
    }
}

impl Units {
    #[must_use]
    pub fn knows(&self, name: &str) -> bool {
        self.infos.contains_key(name)
    }

    /// Whether a unit's family says whether each value is a point or a span (D249).
    #[must_use]
    pub fn split(&self, name: &str) -> bool {
        self.infos.get(mark_of(name).0).is_some_and(|info| info.split)
    }

    /// Where a unit's zero is, from its base's zero, in its own unit (D250).
    #[must_use]
    pub fn offset(&self, name: &str) -> f64 {
        self.infos.get(mark_of(name).0).map_or(0.0, |info| info.offset)
    }

    /// Whether a name is a unit family, like `length`.
    #[must_use]
    pub fn is_family(&self, name: &str) -> bool {
        self.infos.values().any(|info| info.family == name)
    }

    /// The unit's family, `time` for `time.seconds`.
    #[must_use]
    pub fn family(&self, name: &str) -> Option<&str> {
        self.infos.get(name).map(|info| info.family.as_str())
    }

    /// A unit made by maths, expanded: the product of its units' expansions.
    #[must_use]
    pub fn expand(&self, unit: &Unit) -> Option<Expansion> {
        let mut scale = Ratio::ONE;
        let mut powers: Vec<(String, i32)> = Vec::new();
        let mut pi = 0;
        for (name, power) in &unit.factors {
            let expansion = self.expansions.get(mark_of(name).0)?.as_ref()?;
            scale = scale.times(expansion.scale.power(*power)?)?;
            pi += expansion.pi * power;
            for (base, inner) in &expansion.powers {
                match powers.iter_mut().find(|(each, _)| each == base) {
                    Some((_, mine)) => *mine += inner * power,
                    None => powers.push((base.clone(), inner * power)),
                }
            }
        }
        powers.retain(|(_, power)| *power != 0);
        powers.sort();
        Some(Expansion { scale, pi, powers })
    }
}

impl Checker<'_> {
    /// Reads every `fam.unit`, then works out what each unit is, so units can point at each other in any order.
    pub fn declare_units(&mut self, file: &syntax::File) {
        for item in &file.items {
            let syntax::ItemKind::Family(family) = &item.kind else { continue };
            if family.keyword.text() != "fam.unit" {
                continue;
            }
            let set = family.name.text();
            self.check_closer(&family.end, &format!("end fam.unit {set}"), super::CLOSER_WHY);
            let split = family.entries.iter().any(|entry| {
                matches!(
                    entry.kind,
                    syntax::FamilyEntryKind::UnitSetTo(
                        syntax::UnitDefinition::Offset { .. } | syntax::UnitDefinition::Decibels(_)
                    )
                )
            });
            for entry in &family.entries {
                let name = entry.name.text();
                // A family is one level deep (D161).
                if let Some(rest) = name.strip_prefix(&format!("{set}."))
                    && rest.contains('.')
                {
                    self.error(Diagnostic::refused(
                        entry.name.span,
                        format!("`{name}` goes more than one level into `{set}`"),
                        "a family is one level deep, so a family can't hold a family (D161)",
                    ));
                    continue;
                }
                if self.units.infos.contains_key(&name) {
                    self.error(Diagnostic::refused(
                        entry.name.span,
                        format!("`{name}` is declared twice"),
                        "every unit says what it is, once (D109)",
                    ));
                    continue;
                }
                let definition = self.unit_definition(entry);
                let offset = match &definition {
                    Definition::Offset { offset, .. } => *offset,
                    _ => 0.0,
                };
                self.units.order.push(name.clone());
                self.units
                    .infos
                    .insert(name, UnitInfo { family: set.clone(), span: entry.name.span, definition, split, offset });
            }
        }
        for name in self.units.order.clone() {
            let mut visiting = Vec::new();
            let _ = self.expand_declared(&name, &mut visiting);
        }
    }

    fn unit_definition(&mut self, entry: &syntax::FamilyEntry) -> Definition {
        match &entry.kind {
            syntax::FamilyEntryKind::Base { with: None } => Definition::Base,
            syntax::FamilyEntryKind::Base { with: Some(with) } => {
                self.not_yet(with.span, "a prefix with `with`".to_string());
                Definition::Broken
            }
            syntax::FamilyEntryKind::Bare => {
                self.error(Diagnostic::refused(
                    entry.name.span,
                    "this unit says neither `base` nor `=`",
                    "every unit says what it is, `: base` or `=` with a conversion (D109)",
                ));
                Definition::Broken
            }
            syntax::FamilyEntryKind::UnitSetTo(syntax::UnitDefinition::Multiple { amount, unit }) => {
                let syntax::ExpressionKind::Integer(text) = &amount.kind else {
                    self.not_yet(amount.span, "a unit that's a fraction of another".to_string());
                    return Definition::Broken;
                };
                let Some(ratio) = text.replace('_', "").parse::<i128>().ok().and_then(|top| Ratio::new(top, 1)) else {
                    self.not_yet(amount.span, "this conversion".to_string());
                    return Definition::Broken;
                };
                Definition::Multiple { ratio, of: self.resolve_path(unit), of_span: unit.span }
            }
            syntax::FamilyEntryKind::UnitSetTo(syntax::UnitDefinition::Made { factors }) => Definition::Made(
                factors.iter().map(|(path, power)| (self.resolve_path(path), *power, path.span)).collect(),
            ),
            syntax::FamilyEntryKind::UnitSetTo(syntax::UnitDefinition::Offset { unit, amount, span }) => {
                let written = self.slice(amount.span).trim_start_matches('~').replace('_', "");
                let Ok(offset) = written.parse::<f64>() else {
                    self.not_yet(*span, "this offset".to_string());
                    return Definition::Broken;
                };
                Definition::Offset { of: self.resolve_path(unit), of_span: unit.span, offset }
            }
            syntax::FamilyEntryKind::UnitSetTo(syntax::UnitDefinition::Decibels(span)) => {
                self.not_yet(*span, "a log unit, with `decibels of`".to_string());
                Definition::Broken
            }
            syntax::FamilyEntryKind::UnitSetTo(syntax::UnitDefinition::Other(span)) => {
                self.error(Diagnostic::not_designed(
                    *span,
                    "how a conversion other than a multiple is written",
                    "not decided yet (standard 12.2)",
                ));
                Definition::Broken
            }
            _ => {
                self.error(Diagnostic::refused(
                    entry.span,
                    "a unit is `: base`, or `=` with a conversion",
                    "every unit says what it is, `: base` or `=` with a conversion (D109)",
                ));
                Definition::Broken
            }
        }
    }

    /// What a declared unit expands to, saying once where a loop or an unknown unit stops it.
    fn expand_declared(&mut self, name: &str, visiting: &mut Vec<String>) -> Option<Expansion> {
        if let Some(known) = self.units.expansions.get(name) {
            return known.clone();
        }
        let info = self.units.infos.get(name)?.clone();
        if visiting.iter().any(|each| each == name) {
            return None;
        }
        visiting.push(name.to_string());
        let result = match &info.definition {
            Definition::Base => Some(Expansion { scale: Ratio::ONE, pi: 0, powers: vec![(name.to_string(), 1)] }),
            Definition::Offset { of, of_span, .. } => {
                if self.units.knows(of) {
                    self.expand_declared(of, visiting)
                } else {
                    self.error(Diagnostic::refused(
                        *of_span,
                        format!("the unit `{of}` isn't known"),
                        "`=` points at a unit, in any family (D109)",
                    ));
                    None
                }
            }
            Definition::Broken => None,
            Definition::Multiple { ratio, of, of_span } => {
                if !self.units.knows(of) {
                    self.error(Diagnostic::refused(
                        *of_span,
                        format!("the unit `{of}` isn't known"),
                        "`=` points at a unit, in any family (D109)",
                    ));
                    None
                } else if visiting.contains(of) {
                    self.loop_error(visiting, name, info.span, false);
                    None
                } else {
                    self.expand_declared(of, visiting).and_then(|inner| {
                        Some(Expansion { scale: inner.scale.times(*ratio)?, pi: inner.pi, powers: inner.powers })
                    })
                }
            }
            Definition::Made(factors) => {
                if factors.iter().any(|(factor, _, _)| factor == name) {
                    self.error(Diagnostic::refused(
                        info.span,
                        format!("`{name}` is made of itself"),
                        "loops and units with no base are refused (D109, D242)",
                    ));
                    None
                } else {
                    let mut unit_factors = Vec::new();
                    let mut known = true;
                    for (factor, power, span) in factors {
                        if !self.units.knows(factor) {
                            self.error(Diagnostic::refused(
                                *span,
                                format!("the unit `{factor}` isn't known"),
                                "`=` points at a unit, in any family (D109)",
                            ));
                            known = false;
                            break;
                        }
                        if visiting.contains(factor) {
                            self.loop_error(visiting, name, info.span, true);
                            known = false;
                            break;
                        }
                        if self.expand_declared(factor, visiting).is_none() {
                            known = false;
                        }
                        unit_factors.push((factor.clone(), *power));
                    }
                    if known { self.units.expand(&Unit { factors: unit_factors }) } else { None }
                }
            }
        };
        visiting.pop();
        self.units.expansions.insert(name.to_string(), result.clone());
        result
    }

    /// A loop of `=` lines, said once, at the first unit in it.
    fn loop_error(&mut self, visiting: &[String], name: &str, span: Span, made: bool) {
        let first = visiting.first().cloned().unwrap_or_else(|| name.to_string());
        let first_span = self.units.infos.get(&first).map_or(span, |info| info.span);
        let why = if made { "loops and units with no base are refused (D109, D242)" } else { LOOP_WHY };
        self.error(Diagnostic::refused(first_span, format!("`{first}` never reaches a `base`"), why));
        for each in visiting {
            self.units.expansions.insert(each.clone(), None);
        }
    }

    /// The unit a type or a number names: `time.seconds`, `items`, or one shortened by `use` (D112). A number's unit
    /// can be bare in a family with points and spans, taking its mark from where it lands, and a type's can't (D249).
    pub fn resolve_unit(&mut self, path: &syntax::Path, bare_ok: bool) -> Check<Unit> {
        let text = self.resolve_path(path);
        let text = text.strip_prefix("my::").unwrap_or(&text).to_string();
        if let Some(rest) = text.strip_prefix("std::")
            && !rest.starts_with("units::")
        {
            return Err(self.error(Diagnostic::refused_fix(
                path.span,
                format!("std::units::{rest}"),
                "a unit family in `std` sits in the `units` group (D199, D196)",
            )));
        }
        if text.contains("::") && !text.starts_with("std::units::") {
            return Err(self.error(Diagnostic::not_designed(
                path.span,
                format!("the unit `{text}`, from another kit"),
                "not decided yet (standard 12.9)",
            )));
        }
        let (base, mark) = mark_of(&text);
        if self.units.knows(base) {
            let split = self.units.split(base);
            match mark {
                Some(mark) if !split => {
                    return Err(self.error(Diagnostic::refused(
                        path.span,
                        format!("`{base}` has no points and spans, so it takes no `.{mark}`"),
                        "only a family with an offset or a log unit, and `std`'s `time`, has points and spans (D249)",
                    )));
                }
                None if split && !bare_ok => {
                    return Err(self.error(Diagnostic::refused(
                        path.span,
                        format!("`{base}` says whether it's a `.point` or a `.span`"),
                        "in a family with an offset or a log unit, and in `std`'s `time`, every value says whether it's a point or a span (D249, D251)",
                    )));
                }
                _ => return Ok(Unit::named(&text)),
            }
        }
        if !text.contains('.')
            && self.units.order.iter().any(|name| name.rsplit_once('.').is_some_and(|(_, last)| last == text))
        {
            return Err(self.error(Diagnostic::refused(
                path.span,
                format!("the bare unit `{text}` doesn't say its family"),
                "a unit always says its family, or its path from a kit, unless `use` shortens it (D112)",
            )));
        }
        if text.starts_with('?') {
            return Err(self.not_yet(path.span, "a unit parameter".to_string()));
        }
        Err(self.error(Diagnostic::refused(
            path.span,
            format!("the unit `{text}` isn't known"),
            "a unit is declared in a `fam.unit` (D109)",
        )))
    }

    /// A unit as a print shows it: the way the file writes it, with a `use`'s short name, and without `std`'s path
    /// (D246).
    #[must_use]
    pub fn shown(&self, unit: &Unit) -> String {
        let named = |name: &str| -> String {
            let (base, _) = mark_of(name);
            for (short, full) in &self.uses {
                let full = full.strip_prefix("my::").unwrap_or(full);
                if full == base {
                    return short.clone();
                }
            }
            base.strip_prefix("std::units::").unwrap_or(base).to_string()
        };
        let shown = Unit { factors: unit.factors.iter().map(|(name, power)| (named(name), *power)).collect() };
        shown.to_string()
    }

    /// The `unit:` label on a number type, which comes first, before `min:` and `max:` (D255).
    pub fn unit_label<'labels>(
        &mut self,
        labels: &'labels [syntax::Label],
    ) -> Check<(Option<Unit>, &'labels [syntax::Label])> {
        // The labels go `unit:`, then `min:`, then `max:` (D259).
        let rank = |label: &syntax::Label| match label.name.text().as_str() {
            "unit" => 0,
            "min" => 1,
            "max" => 2,
            _ => 3,
        };
        if labels.windows(2).any(|pair| rank(&pair[0]) > rank(&pair[1])) && labels.iter().all(|label| rank(label) < 3) {
            let mut ordered: Vec<&syntax::Label> = labels.iter().collect();
            ordered.sort_by_key(|label| rank(label));
            let fixed: Vec<String> = ordered.iter().map(|label| self.slice(label.span).to_string()).collect();
            let span = labels[0].span.to(labels[labels.len() - 1].span);
            return Err(self.error(Diagnostic::refused_fix(
                span,
                fixed.join(", "),
                "a type reads as what it measures, then its range in that unit, so its labels go `unit:`, `min:`, `max:` (D259)",
            )));
        }
        let Some((first, rest)) = labels.split_first() else { return Ok((None, labels)) };
        if first.name.text() != "unit" {
            return Ok((None, labels));
        }
        if let Some(again) = rest.iter().find(|label| label.name.text() == "unit") {
            return Err(self.error(Diagnostic::refused(
                again.name.span,
                "this type says `unit:` twice",
                "a type's unit is a `unit:` label in its brackets (D258)",
            )));
        }
        let unit = match &first.value.kind {
            syntax::ExpressionKind::Path(path) => self.resolve_unit(path, false)?,
            syntax::ExpressionKind::Variable(name) if name.text.starts_with('?') => {
                return Err(self.not_yet(first.value.span, "a unit parameter".to_string()));
            }
            _ => return Err(self.not_yet(first.value.span, "this unit".to_string())),
        };
        Ok((Some(unit), rest))
    }

    /// Whether two units are the same, as written or once expanded, with nothing to convert.
    #[must_use]
    pub fn same_unit(&self, a: Option<&super::types::Unit>, b: Option<&super::types::Unit>) -> bool {
        match (a, b) {
            (None, None) => true,
            (Some(a), Some(b)) => {
                // A point and a span are never the same, whatever their units (D249).
                unit_mark(a) == unit_mark(b)
                    && (a == b || matches!((self.units.expand(a), self.units.expand(b)), (Some(x), Some(y)) if x == y))
            }
            _ => false,
        }
    }

    /// Whether a unit was declared with `per` or `x`, or made by maths, rather than being one plain unit.
    #[must_use]
    pub fn is_compound(&self, unit: &super::types::Unit) -> bool {
        match unit.single() {
            Some(name) => matches!(self.units.infos.get(name).map(|info| &info.definition), Some(Definition::Made(_))),
            None => true,
        }
    }

    /// Why nothing more can be said about a unit that isn't built yet.
    pub fn unit_not_designed(&mut self, span: Span, what: String) -> Reported {
        self.error(Diagnostic::not_designed(span, what, NOT_DECIDED))
    }
}

/// A unit's `.point` or `.span`, if it's one unit that says it.
#[must_use]
pub fn unit_mark(unit: &Unit) -> Option<&str> {
    unit.single().and_then(|name| mark_of(name).1)
}

/// A unit without its mark: `temperature.celsius` for `temperature.celsius.point`.
#[must_use]
pub fn unmarked(unit: &Unit) -> Option<&str> {
    unit.single().map(|name| mark_of(name).0)
}
