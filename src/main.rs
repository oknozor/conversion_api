#[macro_use]
extern crate rocket;

use crate::error::ConvertError;
use conversion::CONVERSION_TABLE;
use rocket::serde::{json::Json, Deserialize, Serialize};

mod conversion;
mod error;

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![convert])
}

#[post("/convert", data = "<conversion>")]
fn convert(conversion: Json<ConversionRequest>) -> Json<ConversionResponse> {
    Json(ConversionResponse {
        result: conversion.execute(),
    })
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
#[serde(crate = "rocket::serde")]
struct ConversionResponse {
    result: f64,
}

/// Represent a conversion command, from the given unit to the given unit
/// with the provided quantity.
#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ConversionRequest {
    from: Unit,
    to: Unit,
    quantity: f64,
}

impl ConversionRequest {
    /// Execute the given conversion, returning the conversion result truncated after the 8th decimal digit.
    pub fn execute(&self) -> f64 {
        let result = self.from.convert_to(self.to, self.quantity);
        let result = format!("{:.8}", result);
        result
            .parse()
            .expect("Back and forth conversion should never fail")
    }
}

/// A Weight unit, either metric (gram, kilo, ton) or pound.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(crate = "rocket::serde", rename_all = "lowercase")]
pub enum Unit {
    Lb,
    Kilo,
    Ton,
    Gram,
}

impl<'a> TryFrom<&'a str> for Unit {
    type Error = ConvertError<'a>;

    fn try_from(unit: &str) -> Result<Self, ConvertError> {
        match unit {
            "lb" => Ok(Unit::Lb),
            "g" => Ok(Unit::Gram),
            "kg" => Ok(Unit::Kilo),
            "metric ton" => Ok(Unit::Ton),
            unit => Err(ConvertError::UnknownUnit(unit)),
        }
    }
}

impl Unit {
    fn is_metric(&self) -> bool {
        matches!(self, Unit::Kilo | Unit::Ton | Unit::Gram)
    }

    fn convert_to(self, to: Unit, quantity: f64) -> f64 {
        let rules = &CONVERSION_TABLE;
        let conversion_rule = rules.iter().find(|rule| rule.from == self && to == rule.to);

        if let Some(rule) = conversion_rule {
            rule.convert(quantity)
        } else {
            unreachable!("Conversion should be representable")
        }
    }
}

#[cfg(test)]
mod test {
    use super::rocket;
    use crate::{ConversionRequest, ConversionResponse, Unit};
    use rocket::http::Status;
    use rocket::local::blocking::Client;
    use speculoos::assert_that;
    use speculoos::prelude::*;

    #[test]
    fn conversion_should_works() {
        let client = Client::new(rocket()).expect("valid rocket instance");
        let request = ConversionRequest {
            from: Unit::Gram,
            to: Unit::Kilo,
            quantity: 1000.0,
        };

        let response = client.post("/convert").json(&request).dispatch();

        assert_that!(response.status()).is_equal_to(Status::Ok);
        assert_that!(response.into_json())
            .is_some()
            .is_equal_to(ConversionResponse { result: 1.0 });
    }
}
