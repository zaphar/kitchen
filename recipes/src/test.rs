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
use crate::VolumeMeasure::*;
use crate::*;

use std::convert::Into;

use abortable_parser::{Result as ParseResult, StrIter};
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
fn test_volume_measure_metric_affinity() {
    let cup = Cup((4 * 4 * 4).into());
    assert_eq!(cup.metric(), false);
    assert_eq!(cup.normalize().metric(), false);
    let ml = ML(1000.into());
    assert_eq!(ml.metric(), true);
    assert_eq!(ml.normalize().metric(), true);
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
        if let $typ(qty) = $typ(1.into()).$conv().normalize() {
            assert_eq!(qty, 1.into());
        } else {
            assert!(false, $msg);
        }
    };
}

#[test]
fn test_volume_normalize() {
    assert_normalize!(Tbsp, into_tsp, "not a tablespoon after normalize call");
    assert_normalize!(Cup, into_floz, "not a cup after normalize call");
    assert_normalize!(Pint, into_cup, "not a pint after normalize call");
    assert_normalize!(Qrt, into_pint, "not a qrt after normalize call");
    assert_normalize!(Ltr, into_ml, "not a ltr after normalize call");
    assert_normalize!(Gal, into_qrt, "not a gal after normalize call");
    assert_normalize!(Gal, into_tsp, "not a gal after normalize call");
}

#[test]
fn test_ingredient_display() {
    let cases = vec![
        (
            Ingredient::new("onion", Some("chopped".to_owned()), Measure::cup(1.into())),
            "1 cup onion (chopped)",
        ),
        (
            Ingredient::new("onion", Some("chopped".to_owned()), Measure::cup(2.into())),
            "2 cups onion (chopped)",
        ),
        (
            Ingredient::new("onion", Some("chopped".to_owned()), Measure::tbsp(1.into())),
            "1 tbsp onion (chopped)",
        ),
        (
            Ingredient::new("onion", Some("chopped".to_owned()), Measure::tbsp(2.into())),
            "2 tbsps onion (chopped)",
        ),
        (
            Ingredient::new("soy sauce", None, Measure::floz(1.into())),
            "1 floz soy sauce",
        ),
        (
            Ingredient::new("soy sauce", None, Measure::floz(2.into())),
            "2 floz soy sauce",
        ),
        (
            Ingredient::new("soy sauce", None, Measure::qrt(1.into())),
            "1 qrt soy sauce",
        ),
        (
            Ingredient::new("soy sauce", None, Measure::qrt(2.into())),
            "2 qrts soy sauce",
        ),
        (
            Ingredient::new("soy sauce", None, Measure::pint(1.into())),
            "1 pint soy sauce",
        ),
        (
            Ingredient::new("soy sauce", None, Measure::pint(2.into())),
            "2 pints soy sauce",
        ),
        (
            Ingredient::new("soy sauce", None, Measure::gal(1.into())),
            "1 gal soy sauce",
        ),
        (
            Ingredient::new("soy sauce", None, Measure::gal(2.into())),
            "2 gals soy sauce",
        ),
        (
            Ingredient::new("soy sauce", None, Measure::ml(1.into())),
            "1 ml soy sauce",
        ),
        (
            Ingredient::new("soy sauce", None, Measure::ml(2.into())),
            "2 ml soy sauce",
        ),
        (
            Ingredient::new("soy sauce", None, Measure::ltr(1.into())),
            "1 ltr soy sauce",
        ),
        (
            Ingredient::new("soy sauce", None, Measure::ltr(2.into())),
            "2 ltr soy sauce",
        ),
        (Ingredient::new("apple", None, Measure::count(1)), "1 apple"),
        (
            Ingredient::new("salt", None, Measure::gram(1.into())),
            "1 gram salt",
        ),
        (
            Ingredient::new("salt", None, Measure::gram(2.into())),
            "2 grams salt",
        ),
        (
            Ingredient::new("onion", Some("minced".to_owned()), Measure::cup(1.into())),
            "1 cup onion (minced)",
        ),
        (
            Ingredient::new(
                "pepper",
                Some("ground".to_owned()),
                Measure::tsp(Ratio::new(1, 2).into()),
            ),
            "1/2 tsp pepper (ground)",
        ),
        (
            Ingredient::new(
                "pepper",
                Some("ground".to_owned()),
                Measure::tsp(Ratio::new(3, 2).into()),
            ),
            "1 1/2 tsps pepper (ground)",
        ),
        (
            Ingredient::new("apple", Some("sliced".to_owned()), Measure::count(1)),
            "1 apple (sliced)",
        ),
        (
            Ingredient::new("potato", Some("mashed".to_owned()), Measure::count(1)),
            "1 potato (mashed)",
        ),
        (
            Ingredient::new("potato", Some("blanched".to_owned()), Measure::count(1)),
            "1 potato (blanched)",
        ),
    ];
    for (i, expected) in cases {
        assert_eq!(format!("{}", i), expected);
    }
}

#[test]
fn test_ratio_parse() {
    if let ParseResult::Complete(_, rat) = parse::ratio(StrIter::new("1/2")) {
        assert_eq!(rat, Ratio::new(1, 2))
    } else {
        assert!(false)
    }
}

#[test]
fn test_quantity_parse() {
    for (i, expected) in vec![
        ("1 ", Quantity::Whole(1)),
        ("1/2 ", Quantity::Frac(Ratio::new(1, 2))),
        ("1 1/2 ", Quantity::Frac(Ratio::new(3, 2))),
    ] {
        match parse::quantity(StrIter::new(i)) {
            ParseResult::Complete(_, qty) => assert_eq!(qty, expected),
            err => assert!(false, "{:?}", err),
        }
    }
}

#[test]
fn test_ingredient_name_parse() {
    for (i, expected) in vec![("flour ", "flour"), ("flour (", "flour")] {
        match parse::ingredient_name(StrIter::new(i)) {
            ParseResult::Complete(_, n) => assert_eq!(n, expected),
            err => assert!(false, "{:?}", err),
        }
    }
}

#[test]
fn test_ingredient_parse() {
    for (i, expected) in vec![
        (
            "1 cup flour ",
            Ingredient::new("flour", None, Volume(Cup(Quantity::Whole(1)))),
        ),
        (
            "\t1 cup flour ",
            Ingredient::new("flour", None, Volume(Cup(Quantity::Whole(1)))),
        ),
        (
            "1 cup apple (chopped)",
            Ingredient::new(
                "apple",
                Some("chopped".to_owned()),
                Volume(Cup(Quantity::Whole(1))),
            ),
        ),
        (
            "1 cup apple (chopped) ",
            Ingredient::new(
                "apple",
                Some("chopped".to_owned()),
                Volume(Cup(Quantity::Whole(1))),
            ),
        ),
        (
            "1 green bell pepper (chopped) ",
            Ingredient::new(
                "green bell pepper",
                Some("chopped".to_owned()),
                Count(Quantity::Whole(1)),
            ),
        ),
        (
            "1 pkg green onion",
            Ingredient::new(
                "green onion",
                None,
                Package("pkg".into(), Quantity::Whole(1)),
            ),
        ),
        (
            "1 bottle green onion",
            Ingredient::new(
                "green onion",
                None,
                Package("bottle".into(), Quantity::Whole(1)),
            ),
        ),
        (
            "1 bot green onion",
            Ingredient::new(
                "green onion",
                None,
                Package("bot".into(), Quantity::Whole(1)),
            ),
        ),
        (
            "1 bag green onion",
            Ingredient::new(
                "green onion",
                None,
                Package("bag".into(), Quantity::Whole(1)),
            ),
        ),
        (
            "1 can baked beans",
            Ingredient::new(
                "baked bean",
                None,
                Package("can".into(), Quantity::Whole(1)),
            ),
        ),
    ] {
        match parse::ingredient(StrIter::new(i)) {
            ParseResult::Complete(_, ing) => assert_eq!(ing, expected),
            err => assert!(false, "{:?}", err),
        }
    }
}

#[test]
fn test_ingredient_list_parse() {
    for (i, expected) in vec![
        (
            "1 cup flour ",
            vec![Ingredient::new(
                "flour",
                None,
                Volume(Cup(Quantity::Whole(1))),
            )],
        ),
        (
            "1 cup flour \n1/2 tsp butter ",
            vec![
                Ingredient::new("flour", None, Volume(Cup(Quantity::Whole(1)))),
                Ingredient::new(
                    "butter",
                    None,
                    Volume(Tsp(Quantity::Frac(Ratio::new(1, 2)))),
                ),
            ],
        ),
    ] {
        match parse::ingredient_list(StrIter::new(i)) {
            ParseResult::Complete(_, ing) => assert_eq!(ing, expected),
            err => assert!(false, "{:?}", err),
        }
    }
}

#[test]
fn test_single_step() {
    let step = "step: 

1 tbsp flour
2 tbsp butter
1 cup apple (chopped)

Saute apples in butter until golden brown. Add flour slowly
until thickens. Set aside to cool.";

    match parse::step(StrIter::new(step)) {
        ParseResult::Complete(_, step) => {
            assert_eq!(step.ingredients.len(), 3);
            assert_eq!(
                step.instructions,
                "Saute apples in butter until golden brown. Add flour slowly
until thickens. Set aside to cool."
            );
        }
        err => assert!(false, "{:?}", err),
    }
}

#[test]
fn test_single_step_with_duration() {
    let step = "step: 30 min

1 tbsp flour
2 tbsp butter
1 cup apple (chopped)

Saute apples in butter until golden brown. Add flour slowly
until thickens. Set aside to cool.";

    match parse::step(StrIter::new(step)) {
        ParseResult::Complete(_, step) => {
            assert_eq!(step.ingredients.len(), 3);
            assert_eq!(
                step.instructions,
                "Saute apples in butter until golden brown. Add flour slowly
until thickens. Set aside to cool."
            );
            assert_eq!(
                step.prep_time.unwrap(),
                std::time::Duration::new(30 * 60, 0)
            )
        }
        err => assert!(false, "{:?}", err),
    }
}

#[test]
fn test_multiple_steps() {
    let steps = "step:

1 tbsp flour
2 tbsp butter
1 cup apple (chopped)

Saute apples in butter until golden brown. Add flour slowly
until thickens. Set aside to cool.

step:

1 tbsp flour
2 tbsp butter

Saute apples in butter until golden brown. Add flour slowly
until thickened. Set aside to cool.
";

    match parse::step_list(StrIter::new(steps)) {
        ParseResult::Complete(_, steps) => {
            assert_eq!(steps.len(), 2);
        }
        err => assert!(false, "{:?}", err),
    }
}

#[test]
fn test_recipe_multiple_steps() {
    let recipe = "title: gooey apple bake

A simple gooey apple bake recipe.

step:

1 tbsp flour
2 tbsp butter
1 cup apple (chopped)

Saute apples in butter until golden brown. Add flour slowly
until thickens. Set aside to cool.

step:

1 tbsp flour
2 tbsp butter

Saute apples in butter until golden brown. Add flour slowly
until thickened. Set aside to cool.

step:

1 tbsp flour
2 tbsp butter

Saute apples in butter until golden brown. Add flour slowly
until thickened. Set aside to cool.
";

    match parse::recipe(StrIter::new(recipe)) {
        ParseResult::Complete(_, recipe) => {
            assert_eq!(recipe.steps.len(), 3);
            assert_eq!(recipe.steps[0].ingredients.len(), 3);
        }
        err => assert!(false, "{:?}", err),
    }
}

#[test]
fn test_recipe_missing_steps_parse_failure() {
    let recipe = "title: gooey apple bake

A simple gooey apple bake recipe.
";
    match parse::recipe(StrIter::new(recipe)) {
        ParseResult::Abort(e) => {
            assert_eq!(e.get_msg(), "Missing recipe steps");
        }
        other => assert!(false, "{:?}", other),
    }
}

#[test]
fn test_step_no_ingredients_parse_failure() {
    let step = "step: 

step: ";
    match parse::step(StrIter::new(step)) {
        ParseResult::Abort(e) => {
            eprintln!("err: {:?}", e);
            assert_eq!(e.get_msg(), "Missing ingredient list");
        }
        other => {
            assert!(false, "{:?}", other);
        }
    }
}

#[test]
fn test_category_single_line_happy_path() {
    let line = "Produce: onion|green pepper|bell pepper|corn|potato|green onion|scallions|lettuce";
    match parse::as_categories(line) {
        Ok(map) => {
            assert_eq!(map.len(), 8);

            assert!(
                map.contains_key("onion"),
                "map does not contain onion {:?}",
                map
            );
        }
        Err(e) => {
            assert!(false, "{:?}", e);
        }
    }
}

#[test]
fn test_category_double_line_happy_path() {
    let line = "Produce: onion|green pepper|bell pepper|corn|potato|green onion|scallions|lettuce\nDairy: milk|butter";
    match parse::as_categories(line) {
        Ok(map) => {
            assert_eq!(map.len(), 10);

            assert!(
                map.contains_key("onion"),
                "map does not contain onion {:?}",
                map
            );
            assert!(
                map.contains_key("milk"),
                "map does not contain milk {:?}",
                map
            );
            assert_eq!(map["milk"], "Dairy");
            println!("{:?}", map);
        }
        Err(e) => {
            assert!(false, "{:?}", e);
        }
    }
}

#[test]
fn test_triple_line() {
    let line = "Produce: onion|green pepper|bell pepper|corn|potato|green onion|scallions|lettuce
Meat: ground beef|beef|pork|chicken|sausage|hot dogs|bacon|lamb
Dairy: milk|butter|heavy cream|cheddar cheese|mozarella|cheddar|white american|american|swiss";
    match parse::as_categories(line) {
        Ok(map) => {
            let mut categories = BTreeSet::new();
            categories.extend(map.values());
            println!("map: {:?}", map);
            assert_eq!(categories.len(), 3);
        }
        Err(e) => {
            assert!(false, "{:?}", e);
        }
    }
}

#[test]
fn test_category_single_ingredient_happy_paths() {
    let ingredients = vec!["foo", "foo\n", "foo|", "foo\nCategory: "];
    for ingredient in ingredients {
        match parse::cat_ingredient(StrIter::new(ingredient)) {
            ParseResult::Complete(_itr, _i) => {
                // yay we pass
                assert_eq!(_i, "foo");
            }
            res => {
                assert!(false, "{:?}", res);
            }
        }
    }
}

#[test]
fn test_ingredients_list_happy_path() {
    let line = "onion|green pepper|bell pepper|corn|potato|green onion|scallions|lettuce|";
    match parse::cat_ingredients_list(StrIter::new(line)) {
        ParseResult::Complete(_itr, i) => {
            assert_eq!(i.len(), 8);
        }
        res => {
            assert!(false, "{:?}", res);
        }
    }
}
