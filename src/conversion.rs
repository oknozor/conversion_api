use crate::{ConvertError, Unit};
use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

const KNOWN_CONVERSIONS: [[&str; 3]; 4] = [
    ["lb", "kg", "0.45359237"],
    ["lb", "g", "453.59237"],
    ["kg", "lb", "2.20462262"],
    ["kg", "metric ton", "0.001"],
];

pub static CONVERSION_TABLE: Lazy<HashSet<ConversionRule>> = Lazy::new(|| {
    // given k=4 (the number of unit) and n=2 (a conversion pair) we have a total of k^n permutations
    let permutations = 4_i32.pow(2) as usize;

    // We will rely on ConversionRule Hash implementation to generate all possible rules.
    // Since we only have a total of 16 permutations we will use a simple greedy algorithm
    // to find all permutations.
    let mut rules = HashSet::with_capacity(permutations);

    // Insert known rules and their counter part in the conversion table
    KNOWN_CONVERSIONS
        .iter()
        .map(ConversionRule::try_from)
        .filter_map(Result::ok)
        .for_each(|rule| {
            let invert_rule = rule.invert();
            rules.insert(rule);
            rules.insert(invert_rule);
        });

    loop {
        // Fill the conversion table until we have all the possible permutations
        if rules.len() == permutations {
            break;
        }

        let current_rules: HashSet<ConversionRule> = rules.clone();

        // Find possible rule combination and generate a new one plus its inversion
        for rule in &current_rules {
            for other in &current_rules {
                if other.from == rule.to {
                    let rule = rule.combine(other);
                    if !rules.contains(&rule) {
                        let inverted = rule.invert();

                        rules.insert(rule);
                        if !rules.contains(&inverted) {
                            rules.insert(inverted);
                        }
                    }
                }
            }
        }
    }

    rules
});

/// A conversion  from a given unit to the target unit.
#[derive(Copy, Clone, Debug)]
pub struct ConversionRule {
    /// The unit to convert from.
    pub from: Unit,
    /// Target unit of the conversion rule.
    pub to: Unit,
    factor: f64,
}

impl PartialEq for ConversionRule {
    fn eq(&self, other: &Self) -> bool {
        self.from.eq(&other.from) && self.to.eq(&other.to)
    }
}

impl Eq for ConversionRule {}

impl Hash for ConversionRule {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.from.hash(state);
        self.to.hash(state);
        state.finish();
    }
}

impl<'a> TryFrom<&'a [&'a str; 3]> for ConversionRule {
    type Error = ConvertError<'a>;
    fn try_from(rule: &'a [&'a str; 3]) -> Result<Self, ConvertError<'a>> {
        Ok(ConversionRule {
            from: rule[0].try_into()?,
            to: rule[1].try_into()?,
            factor: rule[2]
                .parse()
                .expect("Conversion from table rule should never fail"),
        })
    }
}

impl ConversionRule {
    /// Apply the conversion factor to the given quantity
    pub(crate) fn convert(&self, quantity: f64) -> f64 {
        self.factor * quantity
    }

    fn invert(self) -> ConversionRule {
        ConversionRule {
            from: self.to,
            to: self.from,
            factor: 1.0 / self.factor,
        }
    }

    // Combine two rule to get a new one
    fn combine(&self, other: &ConversionRule) -> ConversionRule {
        let from = self.from;
        let to = other.to;

        let factor = if from == to {
            1.0
        } else {
            self.factor * other.factor
        };

        // Unfortunately some rule combination give slightly imprecise results
        // when combining rules from metric to metric units (ex: kg -> lb -> g).
        // When this happens we ceil the conversion factor
        let factor = if from.is_metric() && to.is_metric() && factor > 1.0 {
            factor.ceil()
        } else {
            factor
        };

        ConversionRule { from, to, factor }
    }
}

#[cfg(test)]
mod test {
    use crate::ConversionRequest;
    use crate::Unit;
    use speculoos::prelude::*;

    fn test_conversion(from: Unit, to: Unit, quantity: f64) -> f64 {
        ConversionRequest { from, to, quantity }.execute()
    }

    #[test]
    fn from_pound_to_gram() {
        let result = test_conversion(Unit::Lb, Unit::Gram, 1.0);
        assert_that!(result).is_close_to(453.59237, 0.00001);
    }

    #[test]
    fn from_pound_to_kilo() {
        let result = test_conversion(Unit::Lb, Unit::Kilo, 1.0);
        assert_that!(result).is_close_to(0.45359237, 0.00001);
    }

    #[test]
    fn from_pound_to_ton() {
        let result = test_conversion(Unit::Lb, Unit::Ton, 1.0);
        assert_that!(result).is_close_to(0.00045359, 0.00000001);
    }

    #[test]
    fn from_kilo_to_pound() {
        let result = test_conversion(Unit::Kilo, Unit::Lb, 1.0);
        assert_that!(result).is_close_to(2.20462262, 0.00001);
    }

    #[test]
    fn from_kilo_to_gram() {
        let result = test_conversion(Unit::Kilo, Unit::Gram, 1.0);
        assert_that!(result).is_close_to(1000.0, 0.00001);
    }

    #[test]
    fn from_kilo_to_ton() {
        let result = test_conversion(Unit::Kilo, Unit::Ton, 1.0);
        assert_that!(result).is_close_to(0.001, 0.00001);
    }

    #[test]
    fn from_gram_to_pound() {
        let result = test_conversion(Unit::Gram, Unit::Lb, 1.0);
        assert_that!(result).is_close_to(0.00220462, 0.00001);
    }

    #[test]
    fn from_gram_to_kilo() {
        let result = test_conversion(Unit::Gram, Unit::Kilo, 1.0);
        assert_that!(result).is_close_to(0.001, 0.00001);
    }

    #[test]
    fn from_gram_to_ton() {
        let result = test_conversion(Unit::Gram, Unit::Ton, 1.0);
        assert_that!(result).is_close_to(0.00001, 0.00001);
    }

    #[test]
    fn from_ton_to_pound() {
        let result = test_conversion(Unit::Ton, Unit::Lb, 1.0);
        assert_that!(result).is_close_to(2204.62262185, 0.00001);
    }

    #[test]
    fn from_ton_to_kilo() {
        let result = test_conversion(Unit::Ton, Unit::Kilo, 1.0);
        assert_that!(result).is_close_to(1000.0, 0.00001);
    }

    #[test]
    fn from_ton_to_gram() {
        let result = test_conversion(Unit::Ton, Unit::Gram, 1.0);
        assert_that!(result).is_close_to(1000_000.0, 0.00001);
    }
}
