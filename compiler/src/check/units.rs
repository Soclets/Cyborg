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
    /// Refused or not built, so nothing more is said about it.
    Broken,
}

#[derive(Clone, Debug)]
struct UnitInfo {
    family: String,
    span: Span,
    definition: Definition,
}

/// A unit as an exact scale times powers of bases: `length.kilometers` is 1000 times `length.meters`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expansion {
    pub scale: Ratio,
    pub powers: Vec<(String, i32)>,
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
        units.infos.insert(
            "items".to_string(),
            UnitInfo { family: String::new(), span: Span::default(), definition: Definition::Base },
        );
        units
            .expansions
            .insert("items".to_string(), Some(Expansion { scale: Ratio::ONE, powers: vec![("items".to_string(), 1)] }));
        units
    }
}

impl Units {
    #[must_use]
    pub fn knows(&self, name: &str) -> bool {
        self.infos.contains_key(name)
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
        for (name, power) in &unit.factors {
            let expansion = self.expansions.get(name)?.as_ref()?;
            scale = scale.times(expansion.scale.power(*power)?)?;
            for (base, inner) in &expansion.powers {
                match powers.iter_mut().find(|(each, _)| each == base) {
                    Some((_, mine)) => *mine += inner * power,
                    None => powers.push((base.clone(), inner * power)),
                }
            }
        }
        powers.retain(|(_, power)| *power != 0);
        powers.sort();
        Some(Expansion { scale, powers })
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
            for entry in &family.entries {
                let name = entry.name.text();
                if self.units.infos.contains_key(&name) {
                    self.error(Diagnostic::refused(
                        entry.name.span,
                        format!("`{name}` is declared twice"),
                        "every unit says what it is, once (D109)",
                    ));
                    continue;
                }
                let definition = self.unit_definition(entry);
                self.units.order.push(name.clone());
                self.units.infos.insert(name, UnitInfo { family: set.clone(), span: entry.name.span, definition });
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
            syntax::FamilyEntryKind::UnitSetTo(syntax::UnitDefinition::Offset(span)) => {
                self.not_yet(*span, "a unit with an offset".to_string());
                Definition::Broken
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
            Definition::Base => Some(Expansion { scale: Ratio::ONE, powers: vec![(name.to_string(), 1)] }),
            Definition::Broken => None,
            Definition::Multiple { ratio, of, of_span } => {
                if of.starts_with("std::") {
                    self.not_yet(*of_span, format!("the unit `{of}`, from `std`"));
                    None
                } else if !self.units.knows(of) {
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
                    self.expand_declared(of, visiting)
                        .and_then(|inner| Some(Expansion { scale: inner.scale.times(*ratio)?, powers: inner.powers }))
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

    /// The unit a type or a number names: `time.seconds`, `items`, or one shortened by `use` (D112).
    pub fn resolve_unit(&mut self, path: &syntax::Path) -> Check<Unit> {
        let text = self.resolve_path(path);
        let text = text.strip_prefix("my::").unwrap_or(&text).to_string();
        if text.starts_with("std::") {
            return Err(self.not_yet(path.span, format!("the unit `{text}`, from `std`")));
        }
        if text.contains("::") {
            return Err(self.error(Diagnostic::not_designed(
                path.span,
                format!("the unit `{text}`, from another kit"),
                "not decided yet (standard 12.9)",
            )));
        }
        if self.units.knows(&text) {
            return Ok(Unit::named(&text));
        }
        for split in [".point", ".span"] {
            if let Some(unit) = text.strip_suffix(split)
                && self.units.knows(unit)
            {
                return Err(self.error(Diagnostic::refused(
                    path.span,
                    format!("`{unit}` has no points and spans, so it takes no `{split}`"),
                    "only a family with an offset or a log unit, and `std`'s `time`, has points and spans (D249)",
                )));
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

    /// The `unit:` label on a number type, which comes first, before `min:` and `max:` (D255).
    pub fn unit_label<'labels>(
        &mut self,
        labels: &'labels [syntax::Label],
    ) -> Check<(Option<Unit>, &'labels [syntax::Label])> {
        let Some((first, rest)) = labels.split_first() else { return Ok((None, labels)) };
        if first.name.text() != "unit" {
            if let Some(later) = rest.iter().find(|label| label.name.text() == "unit") {
                return Err(self.error(Diagnostic::refused(
                    later.name.span,
                    "`unit:` comes first, then `min:`, then `max:`",
                    "a type's labels go `unit:`, then `min:`, then `max:` (D255)",
                )));
            }
            return Ok((None, labels));
        }
        let unit = match &first.value.kind {
            syntax::ExpressionKind::Path(path) => self.resolve_unit(path)?,
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
                a == b || matches!((self.units.expand(a), self.units.expand(b)), (Some(x), Some(y)) if x == y)
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
