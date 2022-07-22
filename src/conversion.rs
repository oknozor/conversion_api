use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use once_cell::sync::Lazy;
use rocket::serde::{Deserialize};

const CONVERSION_TABLE: [[&str; 3]; 4] = [
    ["lb", "kg", "0.45359237"],
    ["lb", "g", "453.59237"],
    ["kg", "lb", "2.20462262"],
    ["kg", "metric ton", "0.001"]
];

const ALL_CONVERSION: Lazy<HashSet<ConversionRule>> = Lazy::new(|| {
    let mut rules = HashSet::with_capacity(16);
    CONVERSION_TABLE.iter()
        .map(ConversionRule::from)
        .for_each(|rule| {
            rules.insert(rule);
        });

    CONVERSION_TABLE.iter()
        .map(ConversionRule::from)
        .map(|rule| rule.invert())
        .for_each(|rule| {
            rules.insert(rule);
        });

    loop {
        // Fill the conversion rules until we have all the possible permutations
        if rules.len() == 16 {
            break;
        }

        let current_rules: HashSet<ConversionRule> = rules.clone();

        for rule in &current_rules {
            current_rules.iter()
                .filter(|other| other.from == rule.to)
                .for_each(|other| {
                    println!("FROM {:?}", rule);
                    let rule = rule.combine(&other);

                    println!("TO {:?}", other);
                    println!("COMBINED {:?}", rule);

                    if !rules.contains(&rule) {
                        let inverted = rule.invert();
                        println!("INVERTED {:?}", inverted);
                        rules.insert(rule);
                        if !rules.contains(&inverted) {
                            rules.insert(inverted);
                        }
                    }
                });
        }
    }

    rules
});


#[derive(Copy, Clone, Debug)]
pub struct ConversionRule {
    from: Unit,
    to: Unit,
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

impl From<&[&str; 3]> for ConversionRule {
    fn from(rule: &[&str; 3]) -> Self {
        ConversionRule {
            from: rule[0].into(),
            to: rule[1].into(),
            factor: rule[2].parse().expect("Conversion from table rule should not fail"),
        }
    }
}


impl ConversionRule {
    fn invert(self) -> ConversionRule {
        ConversionRule {
            from: self.to,
            to: self.from,
            factor: 1.0 / self.factor,
        }
    }

    fn apply_to(&self, quantity: f64) -> f64 {
        self.factor * quantity
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

        let factor = if from.is_metric() && to.is_metric() && factor > 1.0 {
            println!("{} -> {}", factor, factor.ceil());
            factor.ceil()
        } else {
            factor
        };

        ConversionRule {
            from,
            to,
            factor,
        }
    }
}

/// Represent a conversion command, from the given unit to the given unit
/// with the provided quantity.
#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Conversion {
    from: Unit,
    to: Unit,
    quantity: f64,
}


/// A Weight unit, either metric (gram, kilo, ton) or pound.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize)]
#[serde(crate = "rocket::serde")]
enum Unit {
    Lb,
    Kilo,
    Ton,
    Gram,
}

impl From<&str> for Unit {
    fn from(unit: &str) -> Self {
        match unit {
            "lb" => Unit::Lb,
            "g" => Unit::Gram,
            "kg" => Unit::Kilo,
            "metric ton" => Unit::Ton,
            _ => todo!("error")
        }
    }
}

impl Conversion {
    /// Execute the given conversion, returning the conversion result truncated after the 8th decimal digit.
    pub fn execute(&self) -> f64 {
        let result = self.from.convert_to(self.to, self.quantity);
        let result = format!("{:.8}", result);
        result.parse().expect("Back and forth conversion should never fail")
    }
}


impl Unit {
    fn is_metric(&self) -> bool {
        matches!(self, Unit::Kilo | Unit::Ton | Unit::Gram)
    }

    fn convert_to(self, to: Unit, quantity: f64) -> f64 {
        let rules = ALL_CONVERSION;
        let conversion_rule = rules
            .iter()
            .find(|rule| rule.from == self && to == rule.to);

        if let Some(rule) = conversion_rule {
            return rule.apply_to(quantity)
        } else {
            todo!("Error")
        }
    }
}

#[cfg(test)]
mod test {
    use crate::Conversion;
    use crate::conversion::Unit;
    use speculoos::prelude::*;

    fn test_conversion(from: Unit, to: Unit, quantity: f64) -> f64 {
        Conversion {
            from,
            to,
            quantity,
        }.execute()
    }

    #[test]
    fn from_pound_to_gram() {
        let result = test_conversion(Unit::Lb, Unit::Gram, 1.0);
        assert_that!(result).is_equal_to(453.59237);
    }

    #[test]
    fn from_pound_to_kilo() {
        let result = test_conversion(Unit::Lb, Unit::Kilo, 1.0);
        assert_that!(result).is_equal_to(0.45359237);
    }

    #[test]
    fn from_pound_to_ton() {
        let result = test_conversion(Unit::Lb, Unit::Ton, 1.0);
        assert_that!(result).is_equal_to(0.00045359);
    }

    #[test]
    fn from_kilo_to_pound() {
        let result = test_conversion(Unit::Kilo, Unit::Lb, 1.0);
        assert_that!(result).is_equal_to(2.20462262);
    }

    #[test]
    fn from_kilo_to_gram() {
        let result = test_conversion(Unit::Kilo, Unit::Gram, 1.0);
        assert_that!(result).is_equal_to(1000.0);
    }

    #[test]
    fn from_kilo_to_ton() {
        let result = test_conversion(Unit::Kilo, Unit::Ton, 1.0);
        assert_that!(result).is_equal_to(0.001);
    }

    #[test]
    fn from_gram_to_pound() {
        let result = test_conversion(Unit::Gram, Unit::Lb, 1.0);
        assert_that!(result).is_equal_to(0.00220462);
    }

    #[test]
    fn from_gram_to_kilo() {
        let result = test_conversion(Unit::Gram, Unit::Kilo, 1.0);
        assert_that!(result).is_equal_to(0.001);
    }

    #[test]
    fn from_gram_to_ton() {
        let result = test_conversion(Unit::Gram, Unit::Ton, 1.0);
        assert_that!(result).is_equal_to(0.000001);
    }

    #[test]
    fn from_ton_to_pound() {
        let result = test_conversion(Unit::Ton, Unit::Lb, 1.0);
        assert_that!(result).is_equal_to(2204.62262185);
    }

    #[test]
    fn from_ton_to_kilo() {
        let result = test_conversion(Unit::Ton, Unit::Kilo, 1.0);
        assert_that!(result).is_equal_to(1000.0);
    }

    #[test]
    fn from_ton_to_gram() {
        let result = test_conversion(Unit::Ton, Unit::Gram, 1.0);
        assert_that!(result).is_equal_to(1000_000.0);
    }
}
