//! Units made of others, prefixes, and points and spans. (D242, D249, D250, D251)
//!
//! A unit is tracked as powers of the units it's made of, so dividing metres by seconds gives
//! metres to the 1 and seconds to the -1, exactly. Maths can make any combination, which
//! shows as `length.meters per time.seconds`, and storing one needs a declared unit whose
//! powers match.

use std::collections::{BTreeMap, HashMap};

use super::{Checker, UnitInfo};
use crate::ast::{Item, Name, Path};
use crate::report::{Outcome, Report, ReportExtra, Span};

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
    pub(super) fn read_prefix_sets(
        items: &[&Item],
    ) -> Outcome<HashMap<String, Vec<(String, Scale)>>> {
        let mut sets: HashMap<String, Vec<(String, Scale)>> = HashMap::new();
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
            let mut words: Vec<(String, Scale)> = Vec::new();
            for (member, scale, per) in &set.members {
                let word = member
                    .text
                    .rsplit_once('.')
                    .map_or(member.text.as_str(), |(_, word)| word)
                    .to_string();
                if words.iter().any(|(found, _)| *found == word) {
                    return Err(Report::refused(
                        member.span,
                        format!("`{}` is declared twice", member.text),
                        "every prefix says its scale, once (D250, D109)",
                    ));
                }
                let scale = match per {
                    Some(per) => scale_of(scale)?.over(&scale_of(per)?, member.span)?,
                    None => scale_of(scale)?,
                };
                words.push((word, scale));
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
        sets: &HashMap<String, Vec<(String, Scale)>>,
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
        for (prefix, scale) in prefixes {
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
                    link: Some(Link::Scale(scale.clone())),
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

/// How a unit relates to the unit it's declared from. (D109, D249, D251, D255)
#[derive(Clone, Debug)]
pub(super) enum Link {
    /// One of this unit is this many of the other.
    Scale(Scale),
    /// A point here is a point there, less this offset; a span only scales. (D249)
    Offset(f64),
    /// Decibels of this amount of the other, for a point. (D251)
    Log(f64),
}

/// A unit's scale: an exact fraction, as `100` or `1 per 1000`, or a float written with `~`.
#[derive(Clone, Debug)]
pub(super) enum Scale {
    Exact(i128, i128),
    Inexact(f64),
}

impl Scale {
    /// This scale divided by another, as `1 per 1000`.
    pub(super) fn over(&self, other: &Scale, span: Span) -> Outcome<Scale> {
        match (self, other) {
            (Scale::Exact(top, bottom), Scale::Exact(other_top, other_bottom)) => fraction(
                top.checked_mul(*other_bottom),
                bottom.checked_mul(*other_top),
                span,
            ),
            _ => Ok(Scale::Inexact(self.float() / other.float())),
        }
    }

    fn float(&self) -> f64 {
        match self {
            #[allow(clippy::cast_precision_loss)] // A float rounds to the nearest. (D255)
            Scale::Exact(top, bottom) => *top as f64 / *bottom as f64,
            Scale::Inexact(value) => *value,
        }
    }
}

/// A fraction in lowest terms, or the oracle saying it's too big to hold.
fn fraction(top: Option<i128>, bottom: Option<i128>, span: Span) -> Outcome<Scale> {
    let (Some(top), Some(bottom)) = (top, bottom) else {
        return Err(Report::not_built(
            span,
            "a unit scale too big for the oracle to hold exactly",
        ));
    };
    if bottom == 0 || top == 0 {
        return Err(Report::refused(
            span,
            "a unit's scale is never zero",
            "every unit says what it is, with a conversion to another (D109)",
        ));
    }
    let divisor = greatest_divisor(top.abs(), bottom.abs());
    let sign = if bottom < 0 { -1 } else { 1 };
    Ok(Scale::Exact(sign * top / divisor, sign * bottom / divisor))
}

fn greatest_divisor(mut left: i128, mut right: i128) -> i128 {
    while right != 0 {
        (left, right) = (right, left % right);
    }
    left.max(1)
}

/// A conversion's amount as a scale: exact for an integer or a decimal written without `~`,
/// and a float for one written with it. (D59)
pub(super) fn scale_of(amount: &crate::ast::Expression) -> Outcome<Scale> {
    use crate::ast::ExpressionKind;
    match &amount.kind {
        ExpressionKind::Integer { digits, negative } => {
            let value: Option<i128> = digits.parse().ok();
            fraction(
                value.map(|value| if *negative { -value } else { value }),
                Some(1),
                amount.span,
            )
        }
        ExpressionKind::Float {
            digits,
            negative,
            inexact: false,
            exponent: None,
        } => {
            let (whole, part) = digits.split_once('.').unwrap_or((digits, ""));
            let top: Option<i128> = format!("{whole}{part}").parse().ok();
            let bottom = u32::try_from(part.len())
                .ok()
                .and_then(|places| 10_i128.checked_pow(places));
            fraction(
                top.map(|top| if *negative { -top } else { top }),
                bottom,
                amount.span,
            )
        }
        _ => Ok(Scale::Inexact(float_of(amount))),
    }
}

/// A number as a float, for an offset or a reference.
pub(super) fn float_of(amount: &crate::ast::Expression) -> f64 {
    use crate::ast::ExpressionKind;
    match &amount.kind {
        ExpressionKind::Integer { digits, negative }
        | ExpressionKind::Float {
            digits, negative, ..
        } => {
            let value: f64 = digits.parse().unwrap_or(0.0);
            if *negative {
                -value
            } else {
                value
            }
        }
        ExpressionKind::Negate(inner) => -float_of(inner),
        _ => 0.0,
    }
}

impl Checker<'_> {
    /// The links from a unit up to its base, and the base. `std`'s units are known here too.
    /// Nothing for a derived unit, or one made of a combination. (D255, D245)
    fn chain(&self, unit: &str) -> Option<(Vec<Link>, String)> {
        match unit {
            "std::units::time.seconds" | "std::units::angle.radians" => {
                return Some((Vec::new(), unit.to_string()));
            }
            "std::units::angle.degrees" => {
                return Some((
                    vec![Link::Scale(Scale::Inexact(0.017_453_292_519_943_295))],
                    "std::units::angle.radians".to_string(),
                ));
            }
            "std::units::angle.turns" => {
                return Some((
                    vec![Link::Scale(Scale::Inexact(std::f64::consts::TAU))],
                    "std::units::angle.radians".to_string(),
                ));
            }
            _ => {}
        }
        let mut links = Vec::new();
        let mut current = unit.to_string();
        for _ in 0..DEPTH_LIMIT {
            let info = self.units.get(&current)?;
            if info.made_of.is_some() {
                return None;
            }
            match (&info.link, &info.target) {
                (Some(link), Some((target, _))) => {
                    links.push(link.clone());
                    current = self.canonical(target);
                }
                _ => return Some((links, current)),
            }
        }
        None
    }
}

const WHY_CONVERT: &str = "a value changes unit with `as`: a float rounds to the nearest, an integer going to a smaller unit is exact, and one going to a bigger unit says how it rounds with `as.trunc`, `as.floor` or `as.exact` (D255)";

impl Checker<'_> {
    /// Whether an `as` changes a number's unit to another of the same kind of number.
    pub(super) fn is_unit_conversion(
        &self,
        found: &crate::types::Type,
        target: &crate::types::Type,
    ) -> bool {
        use crate::types::Type;
        match (found, target) {
            (
                Type::Integer {
                    unit: Some(from), ..
                },
                Type::Integer { unit: Some(to), .. },
            )
            | (
                Type::Float {
                    unit: Some(from), ..
                },
                Type::Float { unit: Some(to), .. },
            ) => !self.same_unit(Some(from), Some(to)),
            _ => false,
        }
    }

    /// `$km as f64.length.meters`: up the chain from one unit to their shared base, then down
    /// to the other. A point applies offsets and logs, and a span only scales. A float rounds
    /// at each step. An integer only takes exact scales: a whole ratio multiplies, and one over
    /// a whole number divides, rounding as its narrowing says. (D255, D249, D251)
    /// trace: D255, D249, D251
    pub(super) fn convert_unit(
        &self,
        node: crate::ir::Node,
        found: &crate::types::Type,
        target: &crate::types::Type,
        narrowing: Option<&Name>,
        span: Span,
    ) -> Outcome<(crate::ir::Node, crate::types::Type)> {
        use crate::types::Type;
        let (Type::Integer {
            unit: Some(from), ..
        }
        | Type::Float {
            unit: Some(from), ..
        }) = found
        else {
            return Err(Report::refused(span, "this has no unit", WHY_CONVERT));
        };
        let (Type::Integer { unit: Some(to), .. } | Type::Float { unit: Some(to), .. }) = target
        else {
            return Err(Report::refused(span, "this has no unit", WHY_CONVERT));
        };
        let (from_bare, from_split) = split_of(from);
        let (to_bare, to_split) = split_of(to);
        if from_split != to_split {
            return Err(Report::refused(
                span,
                format!("`{from}` and `{to}` aren't both points or both spans"),
                "a point converts to a point and a span to a span, since a point applies offsets and a span only scales (D249, D255)",
            ));
        }
        let point = from_split == Some("point");
        let (Some((up, start_base)), Some((down, end_base))) =
            (self.chain(from_bare), self.chain(to_bare))
        else {
            return Err(Report::not_designed(
                span,
                format!(
                    "converting `{from}` to `{to}`, which a derived unit's conversion would do"
                ),
                "12.2",
            ));
        };
        if start_base != end_base {
            return Err(Report::refused(
                span,
                format!("`{from}` and `{to}` don't trace back to the same base"),
                "two units convert if they trace back to the same base, and otherwise they don't (D109)",
            ));
        }
        let converted = match (found, target) {
            (Type::Integer { kind, .. }, Type::Integer { kind: to_kind, .. }) => {
                if kind != to_kind {
                    return Err(Report::not_designed(
                        span,
                        "changing a number's width and its unit in one `as`",
                        "",
                    ));
                }
                Self::convert_integer(&node, *kind, &up, &down, narrowing, span)?
            }
            (
                Type::Float { kind, rules, .. },
                Type::Float {
                    kind: to_kind,
                    rules: to_rules,
                    ..
                },
            ) => {
                if kind != to_kind || rules != to_rules {
                    return Err(Report::not_designed(
                        span,
                        "changing a float's kind and its unit in one `as`",
                        "",
                    ));
                }
                if let Some(narrowing) = narrowing {
                    return Err(Report::refused(
                        narrowing.span,
                        "a float rounds to the nearest, so its `as` says no narrowing",
                        WHY_CONVERT,
                    ));
                }
                let number = crate::ir::Number::Float(*kind, *rules);
                Self::convert_float(node, number, &up, &down, point, span)?
            }
            _ => {
                return Err(Report::refused(
                    span,
                    "this changes more than a unit",
                    WHY_CONVERT,
                ))
            }
        };
        Ok((converted, target.clone()))
    }

    fn convert_float(
        node: crate::ir::Node,
        number: crate::ir::Number,
        up: &[Link],
        down: &[Link],
        point: bool,
        span: Span,
    ) -> Outcome<crate::ir::Node> {
        use crate::ir::{Arithmetic, Node, NodeKind};
        let constant = |value: f64| {
            let value = match number {
                #[allow(clippy::cast_possible_truncation)] // An `f32` rounds to the nearest.
                crate::ir::Number::Float(crate::types::FloatKind::F32, _) => {
                    crate::types::Value::F32(value as f32)
                }
                _ => crate::types::Value::F64(value),
            };
            Box::new(Node {
                span,
                kind: NodeKind::Constant(value),
            })
        };
        let step = |node: Node, operator: Arithmetic, value: f64| Node {
            span,
            kind: NodeKind::Arithmetic {
                operator,
                number,
                left: Box::new(node),
                right: constant(value),
            },
        };
        let divide = Arithmetic::Divide(crate::ast::Narrowing::Bare(span));
        let mut node = node;
        for link in up {
            node = match link {
                Link::Scale(scale) => step(node, Arithmetic::Multiply, scale.float()),
                Link::Offset(offset) if point => step(node, Arithmetic::Subtract, *offset),
                Link::Offset(_) => node,
                Link::Log(reference) => {
                    Self::decibels(node, number, *reference, true, point, span)?
                }
            };
        }
        for link in down.iter().rev() {
            node = match link {
                Link::Scale(scale) => step(node, divide, scale.float()),
                Link::Offset(offset) if point => step(node, Arithmetic::Add, *offset),
                Link::Offset(_) => node,
                Link::Log(reference) => {
                    Self::decibels(node, number, *reference, false, point, span)?
                }
            };
        }
        Ok(node)
    }

    fn decibels(
        node: crate::ir::Node,
        number: crate::ir::Number,
        reference: f64,
        to_amount: bool,
        point: bool,
        span: Span,
    ) -> Outcome<crate::ir::Node> {
        if !point {
            return Err(Report::not_designed(
                span,
                "converting a gain in decibels, a span, to another unit",
                "12.2",
            ));
        }
        Ok(crate::ir::Node {
            span,
            kind: crate::ir::NodeKind::Decibels {
                value: Box::new(node),
                number,
                reference,
                to_amount,
            },
        })
    }

    fn convert_integer(
        node: &crate::ir::Node,
        kind: crate::types::IntegerKind,
        up: &[Link],
        down: &[Link],
        narrowing: Option<&Name>,
        span: Span,
    ) -> Outcome<crate::ir::Node> {
        use crate::ir::{Arithmetic, Node, NodeKind, Number};
        let (top, bottom) = exact_ratio(up, down, span)?;
        let number = Number::Integer(kind);
        let step = |operator: Arithmetic, value: i128| Node {
            span,
            kind: NodeKind::Arithmetic {
                operator,
                number,
                left: Box::new(node.clone()),
                right: Box::new(Node {
                    span,
                    kind: NodeKind::Constant(crate::types::Value::Integer(value)),
                }),
            },
        };
        if bottom == 1 {
            if let Some(narrowing) = narrowing {
                return Err(Report::refused(
                    narrowing.span,
                    "this goes to a smaller unit, which is exact, so its `as` says no narrowing",
                    WHY_CONVERT,
                ));
            }
            return Ok(step(Arithmetic::Multiply, top));
        }
        if top != 1 {
            return Err(Report::not_designed(
                span,
                format!("an integer conversion whose ratio, {top} per {bottom}, isn't whole"),
                "12.2",
            ));
        }
        let Some(narrowing) = narrowing else {
            return Err(Report::refused(
                span,
                format!("this goes to a unit {bottom} times bigger, which can lose something, so `as` says how it rounds"),
                WHY_CONVERT,
            ));
        };
        let rounding = match narrowing.text.as_str() {
            "trunc" => crate::ast::Narrowing::Trunc,
            "floor" => crate::ast::Narrowing::Floor,
            "exact" => crate::ast::Narrowing::Exact,
            other => {
                return Err(Report::not_designed(
                    narrowing.span,
                    format!("`as.{other}`, since the full list of `as` narrowings isn't chosen"),
                    "12.2",
                ));
            }
        };
        Ok(step(Arithmetic::Divide(rounding), bottom))
    }
}

/// How many of the end unit one of the start unit is, as a fraction, for an integer, which
/// only takes exact scales. (D255)
fn exact_ratio(up: &[Link], down: &[Link], span: Span) -> Outcome<(i128, i128)> {
    let mut ratio = Scale::Exact(1, 1);
    for (link, upward) in up
        .iter()
        .map(|link| (link, true))
        .chain(down.iter().map(|link| (link, false)))
    {
        let Link::Scale(scale @ Scale::Exact(..)) = link else {
            return Err(Report::not_designed(
                span,
                "an integer conversion with an offset, a log or a scale written with `~`",
                "12.2",
            ));
        };
        ratio = if upward {
            ratio.over(&Scale::Exact(1, 1).over(scale, span)?, span)?
        } else {
            ratio.over(scale, span)?
        };
    }
    match ratio {
        Scale::Exact(top, bottom) => Ok((top, bottom)),
        Scale::Inexact(_) => Err(Report::not_designed(
            span,
            "an integer conversion that isn't exact",
            "12.2",
        )),
    }
}

const WHY_BOUND: &str = "a bound writes its unit like any value given to its type, in any unit with the same base, converted when compiling, and exactly for an integer type (D260)";

impl Checker<'_> {
    /// The links from a bound's unit to its type's, which have to share a base. (D260)
    fn bound_links(&self, from: &str, to: &str, span: Span) -> Outcome<(Vec<Link>, Vec<Link>)> {
        let (Some((up, start_base)), Some((down, end_base))) = (self.chain(from), self.chain(to))
        else {
            return Err(Report::not_designed(
                span,
                format!("a bound in `{from}` on a type in `{to}`, which a derived unit's conversion would do"),
                "12.2",
            ));
        };
        if start_base != end_base {
            return Err(Report::refused(
                span,
                format!("`{from}` and `{to}` don't trace back to the same base"),
                WHY_BOUND,
            ));
        }
        Ok((up, down))
    }

    /// An integer bound in another unit, converted exactly to its type's, or refused rather
    /// than rounded. (D260)
    /// trace: D260
    pub(super) fn integer_bound_in_unit(
        &self,
        value: i128,
        from: &str,
        to: &str,
        span: Span,
    ) -> Outcome<i128> {
        let (up, down) = self.bound_links(from, to, span)?;
        let (top, bottom) = exact_ratio(&up, &down, span)?;
        let Some(scaled) = value.checked_mul(top) else {
            return Err(Report::not_built(
                span,
                "a bound too big for the oracle to convert",
            ));
        };
        if scaled % bottom != 0 {
            return Err(Report::refused(
                span,
                format!("`{value} {from}` isn't a whole number of `{to}`, so this integer bound can't convert exactly"),
                WHY_BOUND,
            ));
        }
        Ok(scaled / bottom)
    }

    /// A float bound in another unit, converted to its type's the way `as` does, rounding at
    /// each step. (D260, D255)
    /// trace: D260, D255
    pub(super) fn float_bound_in_unit(
        &self,
        value: f64,
        kind: crate::types::FloatKind,
        from: &str,
        to: &str,
        span: Span,
    ) -> Outcome<f64> {
        let (up, down) = self.bound_links(from, to, span)?;
        #[allow(clippy::cast_possible_truncation)] // An `f32` rounds to the nearest.
        let round = |value: f64| match kind {
            crate::types::FloatKind::F32 => f64::from(value as f32),
            crate::types::FloatKind::F64 => value,
        };
        let mut value = value;
        for (link, upward) in up
            .iter()
            .map(|link| (link, true))
            .chain(down.iter().rev().map(|link| (link, false)))
        {
            let Link::Scale(scale) = link else {
                return Err(Report::not_designed(
                    span,
                    "a bound in a unit with an offset or a log",
                    "",
                ));
            };
            let scale = round(scale.float());
            value = round(if upward { value * scale } else { value / scale });
        }
        Ok(value)
    }
}

impl Checker<'_> {
    /// The `unit:` in a number type's brackets, resolved, and the brackets' other labels.
    /// Which order the labels go in isn't decided, so any is read. (D258)
    /// trace: D258
    pub(super) fn unit_in_brackets(
        &self,
        describe: &[crate::ast::Describe],
    ) -> Outcome<(Option<String>, Vec<crate::ast::Describe>)> {
        self.refuse_bracket_order(describe)?;
        let mut unit = None;
        let mut rest = Vec::new();
        for entry in describe {
            if entry.label.text != "unit" {
                rest.push(entry.clone());
                continue;
            }
            let crate::ast::ExpressionKind::Path(path) = &entry.value.kind else {
                return Err(Report::refused(
                    entry.value.span,
                    "`unit:` names a unit",
                    "a type's unit is a `unit:` label in its brackets (D258)",
                ));
            };
            if unit.is_some() {
                return Err(Report::refused(
                    entry.label.span,
                    "this type says `unit:` twice",
                    "a type's unit is a `unit:` label in its brackets (D258)",
                ));
            }
            unit = Some(self.resolve_unit(path)?);
        }
        Ok((unit, rest))
    }

    /// `i32.time.seconds`, the old spelling, refused with the bracket form as its fix when
    /// the brackets hold nothing else. (D258)
    /// trace: D258
    pub(super) fn unit_in_the_chain(
        head: &str,
        unit: &str,
        describe: &[crate::ast::Describe],
        written: &crate::ast::TypeExpression,
    ) -> Box<Report> {
        let refused = Report::refused(
            written.span,
            format!("`{unit}` is a unit, so it goes in the type's brackets as `unit:`"),
            "a type's unit is a `unit:` label in its brackets, next to any bounds (D258)",
        );
        if describe.is_empty() {
            refused.fix(format!("{head}[unit: {unit}]"))
        } else {
            refused
        }
    }
}

impl Checker<'_> {
    /// A number type's labels go `unit:`, then `min:`, then `max:`, and another order is
    /// refused with the right one as its fix. (D259)
    /// trace: D259
    fn refuse_bracket_order(&self, describe: &[crate::ast::Describe]) -> Outcome<()> {
        let rank = |label: &str| match label {
            "unit" => Some(0),
            "min" => Some(1),
            "max" => Some(2),
            _ => None,
        };
        let ranks: Vec<Option<u8>> = describe
            .iter()
            .map(|entry| rank(&entry.label.text))
            .collect();
        let known: Vec<u8> = ranks.iter().flatten().copied().collect();
        // A label written twice is refused for that, where it's read.
        let twice = known
            .iter()
            .enumerate()
            .any(|(at, rank)| known.iter().skip(at + 1).any(|later| later == rank));
        if twice || known.windows(2).all(|pair| pair[0] < pair[1]) {
            return Ok(());
        }
        let (Some(first), Some(last)) = (describe.first(), describe.last()) else {
            return Ok(());
        };
        let span = first.label.span.to(last.value.span);
        let refused = Report::refused(
            span,
            "a type's labels go `unit:`, then `min:`, then `max:`",
            "a type reads as what it measures, then its range in that unit, so its labels go `unit:`, `min:`, `max:` (D259)",
        );
        if ranks.iter().any(Option::is_none) {
            return Err(refused);
        }
        let mut ordered: Vec<&crate::ast::Describe> = describe.iter().collect();
        ordered.sort_by_key(|entry| rank(&entry.label.text));
        let written: Vec<String> = ordered
            .iter()
            .map(|entry| format!("{}: {}", entry.label.text, self.text_of(entry.value.span)))
            .collect();
        Err(refused.fix(written.join(", ")))
    }
}
