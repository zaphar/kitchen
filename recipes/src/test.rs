// Copyright 2021 Jeremy Wall
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use crate::*;
use VolumeMeasure::*;

use std::convert::Into;

use num_rational::Ratio;

#[test]
fn test_volume_measure_conversion() {
    let gal = Gal(1.into());
    assert_eq!(gal.into_qrt(), Qrt(4.into()));
    assert_eq!(gal.into_pint(), Pint((4 * 2).into()));
    assert_eq!(gal.into_cup(), Cup((4 * 2 * 2).into()));
    assert_eq!(gal.into_tbsp(), Tbsp((4 * 2 * 2 * 16).into()));
    assert_eq!(gal.into_tbsp(), Tsp((4 * 2 * 2 * 16 * 3).into()));
}

#[test]
fn test_quantity_math() {
    // All frac
    let half: Quantity = Ratio::new(1, 2).into();
    assert_eq!(half + half, 1.into());
    assert_eq!(half * half, Ratio::new(1, 4).into());
    assert_eq!(half - half, 0.into());
    assert_eq!(half / half, 1.into());
    // Mix of whole and frac
    assert_eq!(Quantity::from(2) * half, 1.into());
    assert_eq!(half * Quantity::from(2), 1.into());
    // All whole
    assert_eq!(
        Quantity::from(2) / Quantity::from(3),
        Quantity::from(Ratio::new(2, 3))
    );
}

#[test]
fn test_volume_math() {
    let tsp = Tsp(1.into());
    assert_eq!(tsp + tsp, Tsp(2.into()));
    assert_eq!(tsp - tsp, Tsp(0.into()));
}

macro_rules! assert_normalize {
    ($typ:path, $conv:ident, $msg:expr) => {
        if let $typ(qty) = dbg!($typ(1.into()).$conv().normalize()) {
            assert_eq!(qty, 1.into());
        } else {
            assert!(false, $msg);
        }
    };
}

#[test]
fn test_volume_normalize() {
    assert_normalize!(Tbsp, into_tsp, "not a tablespoon after normalize call");
    assert_normalize!(Floz, into_tbsp, "not a floz after normalize call");
    assert_normalize!(Cup, into_floz, "not a cup after normalize call");
    assert_normalize!(Pint, into_cup, "not a pint after normalize call");
    assert_normalize!(Qrt, into_pint, "not a qrt after normalize call");
    assert_normalize!(Ltr, into_qrt, "not a ltr after normalize call");
    assert_normalize!(Gal, into_ltr, "not a gal after normalize call");
    assert_normalize!(Gal, into_tsp, "not a gal after normalize call");
}

#[test]
fn test_ingredient_display() {
    let cases = vec![
        (
            Ingredient::new(
                "onion",
                Some("chopped".to_owned()),
                Measure::cup(1.into()),
                "Produce",
            ),
            "1 cup onion (chopped)",
        ),
        (
            Ingredient::new(
                "onion",
                Some("chopped".to_owned()),
                Measure::cup(2.into()),
                "Produce",
            ),
            "2 cups onion (chopped)",
        ),
        (
            Ingredient::new(
                "onion",
                Some("chopped".to_owned()),
                Measure::tbsp(1.into()),
                "Produce",
            ),
            "1 tbsp onion (chopped)",
        ),
        (
            Ingredient::new(
                "onion",
                Some("chopped".to_owned()),
                Measure::tbsp(2.into()),
                "Produce",
            ),
            "2 tbsps onion (chopped)",
        ),
        (
            Ingredient::new("soy sauce", None, Measure::floz(1.into()), "Produce"),
            "1 floz soy sauce",
        ),
        (
            Ingredient::new("soy sauce", None, Measure::floz(2.into()), "Produce"),
            "2 floz soy sauce",
        ),
        (
            Ingredient::new("soy sauce", None, Measure::qrt(1.into()), "Produce"),
            "1 qrt soy sauce",
        ),
        (
            Ingredient::new("soy sauce", None, Measure::qrt(2.into()), "Produce"),
            "2 qrts soy sauce",
        ),
        (
            Ingredient::new("soy sauce", None, Measure::pint(1.into()), "Produce"),
            "1 pint soy sauce",
        ),
        (
            Ingredient::new("soy sauce", None, Measure::pint(2.into()), "Produce"),
            "2 pints soy sauce",
        ),
        (
            Ingredient::new("soy sauce", None, Measure::gal(1.into()), "Produce"),
            "1 gal soy sauce",
        ),
        (
            Ingredient::new("soy sauce", None, Measure::gal(2.into()), "Produce"),
            "2 gals soy sauce",
        ),
        (
            Ingredient::new("soy sauce", None, Measure::ml(1.into()), "Produce"),
            "1 ml soy sauce",
        ),
        (
            Ingredient::new("soy sauce", None, Measure::ml(2.into()), "Produce"),
            "2 ml soy sauce",
        ),
        (
            Ingredient::new("soy sauce", None, Measure::ltr(1.into()), "Produce"),
            "1 ltr soy sauce",
        ),
        (
            Ingredient::new("soy sauce", None, Measure::ltr(2.into()), "Produce"),
            "2 ltr soy sauce",
        ),
        (
            Ingredient::new("apple", None, Measure::count(1), "Produce"),
            "1 apple",
        ),
        (
            Ingredient::new("salt", None, Measure::gram(1.into()), "Produce"),
            "1 gram salt",
        ),
        (
            Ingredient::new("salt", None, Measure::gram(2.into()), "Produce"),
            "2 grams salt",
        ),
        (
            Ingredient::new(
                "onion",
                Some("minced".to_owned()),
                Measure::cup(1.into()),
                "Produce",
            ),
            "1 cup onion (minced)",
        ),
        (
            Ingredient::new(
                "pepper",
                Some("ground".to_owned()),
                Measure::tsp(Ratio::new(1, 2).into()),
                "Produce",
            ),
            "1/2 tsp pepper (ground)",
        ),
        (
            Ingredient::new(
                "pepper",
                Some("ground".to_owned()),
                Measure::tsp(Ratio::new(3, 2).into()),
                "Produce",
            ),
            "1 1/2 tsps pepper (ground)",
        ),
        (
            Ingredient::new(
                "apple",
                Some("sliced".to_owned()),
                Measure::count(1),
                "Produce",
            ),
            "1 apple (sliced)",
        ),
        (
            Ingredient::new(
                "potato",
                Some("mashed".to_owned()),
                Measure::count(1),
                "Produce",
            ),
            "1 potato (mashed)",
        ),
        (
            Ingredient::new(
                "potato",
                Some("blanched".to_owned()),
                Measure::count(1),
                "Produce",
            ),
            "1 potato (blanched)",
        ),
    ];
    for (i, expected) in cases {
        assert_eq!(format!("{}", i), expected);
    }
}
