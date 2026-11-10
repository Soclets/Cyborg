//! Units made of others, prefixes, and points and spans. (D242, D249, D250, D251)
//!
//! A unit is tracked as powers of the units it's made of, so dividing metres by seconds gives
//! metres to the 1 and seconds to the -1, exactly. Maths can make any combination, which
//! shows as `length.meters per time.seconds`, and storing one needs a declared unit whose
//! powers match.

use std::collections::{BTreeMap, HashMap};

use super::{Checker, UnitInfo};
use crate::ast::{Item, Name, Path};
use crate::report::{Outcome, Report, Span};

/// Powers of the units a unit is made of, by name.
pub(super) type Powers = BTreeMap<String, i32>;

/// How deep units made of units can go before the oracle says so.
const DEPTH_LIMIT: usize = 64;

/// The written form of a combination with no name, as in `length.meters per time.seconds`.
/// Nothing when it's plain, and the unit itself when it's one unit once.
pub(super) fn unit_from_powers(powers: &Powers) -> Option<String> {
    let mut above = Vec::new();
    let mut below = Vec::new();
    for (unit, power) in powers {
        for _ in 0..power.unsigned_abs() {
            if *power > 0 {
                above.push(unit.clone());
            } else {
                below.push(unit.clone());
            }
        }
    }
    if above.is_empty() && below.is_empty() {
        return None;
    }
    let mut text = if above.is_empty() {
        "1".to_string()
    } else {
        above.join(" x ")
    };
    for unit in below {
        text.push_str(" per ");
        text.push_str(&unit);
    }
    Some(text)
}

/// Reads back what `unit_from_powers` writes.
fn powers_from_text(text: &str) -> Powers {
    let mut powers = Powers::new();
    let mut pieces = text.split(" per ");
    if let Some(above) = pieces.next() {
        for unit in above.split(" x ").filter(|unit| *unit != "1") {
            *powers.entry(unit.to_string()).or_insert(0) += 1;
        }
    }
    for unit in pieces {
        *powers.entry(unit.to_string()).or_insert(0) -= 1;
    }
    powers.retain(|_, power| *power != 0);
    powers
}

/// `.point` or `.span` on a unit, and the unit without it. (D249)
pub(super) fn split_of(unit: &str) -> (&str, Option<&str>) {
    if let Some(bare) = unit.strip_suffix(".point") {
        return (bare, Some("point"));
    }
    if let Some(bare) = unit.strip_suffix(".span") {
        return (bare, Some("span"));
    }
    (unit, None)
}

impl Checker<'_> {
    /// Each `fam.prefix` family and its prefixes' words, as `metric` with `kilo` and `milli`.
    /// (D250)
    pub(super) fn read_prefix_sets(items: &[&Item]) -> Outcome<HashMap<String, Vec<String>>> {
        let mut sets: HashMap<String, Vec<String>> = HashMap::new();
        for item in items {
            let Item::PrefixSet(set) = item else {
                continue;
            };
            if sets.contains_key(&set.family.text) {
                return Err(Report::not_designed(
                    set.family.span,
                    format!("declaring `{}` twice", set.family.text),
                    "",
                ));
            }
            let mut words: Vec<String> = Vec::new();
            for member in &set.members {
                let word = member
                    .text
                    .rsplit_once('.')
                    .map_or(member.text.as_str(), |(_, word)| word)
                    .to_string();
                if words.contains(&word) {
                    return Err(Report::refused(
                        member.span,
                        format!("`{}` is declared twice", member.text),
                        "every prefix says its scale, once (D250, D109)",
                    ));
                }
                words.push(word);
            }
            sets.insert(set.family.text.clone(), words);
        }
        Ok(sets)
    }

    /// `length.meters: base with metric;` makes `length.kilometers` and the rest, by joining
    /// each prefix to the unit's name, each converting to the unit by its scale. Which
    /// prefixes `std`'s `metric` and `binary` list isn't decided. (D250)
    /// trace: D250
    pub(super) fn add_prefixed_units(
        &mut self,
        unit: &Path,
        family: &Name,
        sets: &HashMap<String, Vec<String>>,
    ) -> Outcome<()> {
        let Some(prefixes) = sets.get(&family.text) else {
            if matches!(family.text.as_str(), "metric" | "binary") {
                return Err(Report::not_designed(
                    family.span,
                    format!("which prefixes `std`'s `{}` lists", family.text),
                    "12.2",
                ));
            }
            return Err(Report::refused(
                family.span,
                format!("there's no prefix family `{}`", family.text),
                "a unit opts into a prefix family declared with `fam.prefix` (D250)",
            ));
        };
        let Some((unit_family, word)) = unit.text.split_once('.') else {
            return Ok(());
        };
        for prefix in prefixes {
            let name = format!("{unit_family}.{prefix}{word}");
            if self.units.contains_key(&name) {
                return Err(Report::refused(
                    family.span,
                    format!("`{name}` is declared twice, once by its prefix"),
                    "every unit says what it is, once (D109, D250)",
                ));
            }
            self.units.insert(
                name,
                UnitInfo {
                    base: None,
                    target: Some((unit.text.clone(), unit.span)),
                    span: family.span,
                    made_of: None,
                },
            );
        }
        Ok(())
    }

    /// Each derived unit is made of units that exist, without going round in a loop. (D242)
    /// trace: D242
    pub(super) fn check_derived_units(&self) -> Outcome<()> {
        let mut derived: Vec<(&String, &UnitInfo)> = self
            .units
            .iter()
            .filter(|(_, info)| info.made_of.is_some())
            .collect();
        derived.sort_by_key(|(_, info)| info.span.start);
        for (name, info) in derived {
            for (_, part) in info.made_of.iter().flatten() {
                let part_name = self.resolve_unit_name(part)?;
                if split_of(&part_name).1.is_some()
                    || self.split_families.contains(family_of(&part_name))
                {
                    return Err(Report::not_designed(
                        part.span,
                        "a unit made of one with points and spans",
                        "12.2",
                    ));
                }
            }
            if self.powers_at(name, 0).is_none() {
                return Err(Report::refused(
                    info.span,
                    format!("`{name}` is made of itself"),
                    "loops and units with no base are refused (D109, D242)",
                ));
            }
        }
        Ok(())
    }

    /// The powers of the units a unit is made of. A unit with points and spans, or a loop,
    /// gives nothing.
    pub(super) fn powers(&self, unit: &str) -> Option<Powers> {
        self.powers_at(unit, 0)
    }

    fn powers_at(&self, unit: &str, depth: usize) -> Option<Powers> {
        if depth > DEPTH_LIMIT {
            return None;
        }
        if split_of(unit).1.is_some() {
            return None;
        }
        if unit.contains(" per ") || unit.contains(" x ") {
            return Some(powers_from_text(unit));
        }
        let Some(parts) = self.units.get(unit).and_then(|info| info.made_of.as_ref()) else {
            let mut alone = Powers::new();
            alone.insert(unit.to_string(), 1);
            return Some(alone);
        };
        let mut powers = Powers::new();
        for (divides, part) in parts {
            let name = self.resolve_unit_name(part).ok()?;
            for (inner, power) in self.powers_at(&name, depth + 1)? {
                *powers.entry(inner).or_insert(0) += if *divides { -power } else { power };
            }
        }
        powers.retain(|_, power| *power != 0);
        Some(powers)
    }

    /// The unit a product or quotient of two units has: the declared one is kept when one side
    /// is plain, and otherwise their powers combine. (D242)
    /// trace: D242
    pub(super) fn combined_unit(
        &self,
        left: Option<&String>,
        right: Option<&String>,
        divides: bool,
        span: Span,
    ) -> Outcome<Option<String>> {
        let side = |unit: Option<&String>| -> Outcome<Powers> {
            match unit {
                None => Ok(Powers::new()),
                Some(unit) => self.powers(unit).ok_or_else(|| {
                    Report::not_designed(
                        span,
                        format!("multiplying or dividing `{unit}`, which has points and spans"),
                        "12.2",
                    )
                }),
            }
        };
        let mut powers = side(left)?;
        for (unit, power) in side(right)? {
            *powers.entry(unit).or_insert(0) += if divides { -power } else { power };
        }
        powers.retain(|_, power| *power != 0);
        Ok(unit_from_powers(&powers))
    }

    /// Whether two units are the same, by name or by what they're made of. (D242)
    pub(super) fn same_unit(&self, left: Option<&String>, right: Option<&String>) -> bool {
        match (left, right) {
            (None, None) => true,
            (Some(left), Some(right)) => {
                left == right
                    || self
                        .powers(left)
                        .zip(self.powers(right))
                        .is_some_and(|(left, right)| left == right)
            }
            _ => false,
        }
    }

    /// Whether two units differ only in which units of the same bases they're made of, as
    /// kilometres per second and metres per second do, which a conversion would cross.
    pub(super) fn same_bases(&self, left: &str, right: &str) -> bool {
        let bases = |unit: &str| -> Option<Powers> {
            let mut bases = Powers::new();
            for (atom, power) in self.powers(unit)? {
                let base = self
                    .units
                    .get(&atom)
                    .and_then(|info| info.base.clone())
                    .unwrap_or(atom);
                *bases.entry(base).or_insert(0) += power;
            }
            bases.retain(|_, power| *power != 0);
            Some(bases)
        };
        bases(left)
            .zip(bases(right))
            .is_some_and(|(left, right)| left == right)
    }

    /// Whether a unit is in a family whose values say `.point` or `.span`: one with an offset
    /// or a log unit, or `std`'s `time`. (D249, D251)
    pub(super) fn needs_a_split(&self, unit: &str) -> bool {
        let family = family_of(unit);
        family == "std::units::time" || self.split_families.contains(family)
    }
}

/// A unit's family, as `temperature` for `temperature.celsius`.
pub(super) fn family_of(unit: &str) -> &str {
    unit.rsplit_once('.').map_or(unit, |(family, _)| family)
}

impl Checker<'_> {
    /// `+` and `-` on points and spans: a point plus a span is a point, a point minus a point
    /// is a span, and spans add, while a point plus a point is refused. Nothing when neither
    /// side has points and spans. (D249, D251)
    /// trace: D249, D251
    pub(super) fn point_and_span(
        operator: crate::ir::Arithmetic,
        left_unit: Option<&String>,
        right_unit: Option<&String>,
        left: &crate::types::Type,
        right: &crate::types::Type,
        span: Span,
    ) -> Outcome<Option<String>> {
        use crate::ir::Arithmetic;
        const WHY: &str = "a point plus a span is a point, a point minus a point is a span, and spans add, while a point plus a point is refused (D249, D251)";
        let (Some(left_unit), Some(right_unit)) = (left_unit, right_unit) else {
            return Ok(None);
        };
        let (left_bare, left_split) = split_of(left_unit);
        let (right_bare, right_split) = split_of(right_unit);
        let (Some(left_split), Some(right_split)) = (left_split, right_split) else {
            return Ok(None);
        };
        if left_bare != right_bare {
            return Err(Report::refused(
                span,
                format!("this mixes `{left}` and `{right}`"),
                "the same units can add and compare, and mixing them is refused (D100)",
            ));
        }
        let adds = operator != Arithmetic::Subtract;
        let result = match (adds, left_split, right_split) {
            (_, "point", "span") | (true, "span", "point") => "point",
            (_, "span", "span") | (false, "point", "point") => "span",
            (true, "point", "point") => {
                return Err(Report::refused(span, "this adds a point to a point", WHY));
            }
            _ => {
                return Err(Report::refused(span, "this takes a point from a span", WHY));
            }
        };
        Ok(Some(format!("{left_bare}.{result}")))
    }
}

impl Checker<'_> {
    /// Storing a value whose unit isn't the one declared: refused when it's made of other
    /// bases, and not designed when it would take a conversion, as kilometres per second into
    /// metres per second would, or when it's a product of a unit parameter. (D242, D243)
    /// trace: D242, D243
    pub(super) fn refuse_unstorable_unit(
        &self,
        found: &crate::types::Type,
        target: &crate::types::Type,
        expression: &crate::ast::Expression,
    ) -> Outcome<()> {
        use crate::types::Type;
        let unit = |found: &Type| match found {
            Type::Integer { unit, .. } | Type::Float { unit, .. } => unit.clone(),
            _ => None,
        };
        let combination = |unit: &str| unit.contains(" per ") || unit.contains(" x ");
        let (Some(found_unit), Some(target_unit)) = (unit(found), unit(target)) else {
            if unit(found).is_some_and(|unit| combination(&unit)) {
                return Err(Report::refused(
                    expression.span,
                    format!("this is `{found}`, where `{target}` is needed"),
                    "storing a value needs a declared unit whose powers match (D242)",
                ));
            }
            return Ok(());
        };
        if self.same_unit(Some(&found_unit), Some(&target_unit)) {
            return Ok(());
        }
        let span = expression.span;
        if found_unit.contains('?') && (found_unit.contains(" per ") || found_unit.contains(" x "))
        {
            return Err(Report::not_designed(
                span,
                "storing a product or quotient of a unit parameter",
                "12.2",
            ));
        }
        if self.same_bases(&found_unit, &target_unit) {
            return Err(Report::not_designed(
                span,
                format!("converting `{found_unit}` to `{target_unit}`, which a derived unit's conversion would do"),
                "12.2",
            ));
        }
        if found_unit.contains(" per ") || found_unit.contains(" x ") {
            return Err(Report::refused(
                span,
                format!("this is `{found}`, where `{target}` is needed"),
                "storing a value needs a declared unit whose powers match (D242)",
            ));
        }
        Ok(())
    }
}

impl Checker<'_> {
    /// A literal like `10.0 celsius` in a family with points and spans, which can't say which
    /// it is.
    pub(super) fn literal_with_a_split(&self, expression: &crate::ast::Expression) -> bool {
        let crate::ast::ExpressionKind::WithUnit { unit, .. } = &expression.kind else {
            return false;
        };
        self.resolve_unit_name(unit)
            .is_ok_and(|unit| self.needs_a_split(&unit))
    }
}
