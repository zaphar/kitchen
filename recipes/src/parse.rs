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
use std::collections::BTreeMap;
use std::str::FromStr;
use std::time::Duration;

use abortable_parser::{
    ascii_digit, consume_all, discard, do_each, either, eoi, make_fn, must, not, optional, peek,
    repeat, separated, text_token, trap, until, with_err, Result, StrIter,
};
use inflector::Inflector;
use num_rational::Ratio;

use crate::{
    unit::{Measure, Measure::*, Quantity, VolumeMeasure::*, WeightMeasure::*},
    Ingredient, Recipe, Step,
};

pub fn as_recipe(i: &str) -> std::result::Result<Recipe, String> {
    match recipe(StrIter::new(i)) {
        Result::Abort(e) | Result::Fail(e) => Err(format!("Parse Failure: {:?}", e)),
        Result::Incomplete(_) => Err(format!("Incomplete recipe can not parse")),
        Result::Complete(_, r) => Ok(r),
    }
}

pub fn as_categories(i: &str) -> std::result::Result<BTreeMap<String, String>, String> {
    match categories(StrIter::new(i)) {
        Result::Abort(e) | Result::Fail(e) => Err(format!("Parse Failure: {:?}", e)),
        Result::Incomplete(_) => Err(format!("Incomplete categories list can not parse")),
        Result::Complete(_, c) => Ok(c),
    }
}

make_fn!(
    pub categories<StrIter, BTreeMap<String, String>>,
    do_each!(
        first_category => category_line,
        rest => repeat!(category_line),
        ({
            let mut out = BTreeMap::new();
            let mut op = |(cat, mut ingredients): (String, Vec<String>)| {
                for i in ingredients.drain(0..) {
                    out.insert(i, cat.clone());
                }
            };
            op(first_category);
            for line in rest.iter() {
                op(line.clone());
            }
            out
        })
    )
);

make_fn!(
    category_line<StrIter, (String, Vec<String>)>,
    do_each!(
        cat => until!(text_token!(":")),
        _ => text_token!(":"),
        ingredients => cat_ingredients_list,
        _ => either!(
            discard!(eoi),
            discard!(text_token!("\n"))
        ),
        ((cat.trim().to_owned(), ingredients))
    )
);

make_fn!(
    pub cat_ingredients_list<StrIter, Vec<String>>,
    do_each!(
        first_ingredient => must!(cat_ingredient),
        rest => repeat!(cat_ingredient),
        ({
            let mut ingredients = vec![first_ingredient];
            ingredients.extend(rest);
            ingredients
        })
    )
);

make_fn!(
    pub cat_ingredient<StrIter, String>,
    do_each!(
        _ => peek!(not!(
            either!(
                discard!(text_token!("|")),
                discard!(eoi),
                discard!(text_token!("\n"))
            )
        )),
        ingredient => until!(
            either!(
                discard!(text_token!("|")),
                discard!(eoi),
                discard!(text_token!("\n"))
            )
        ),
        _ => either!(
            discard!(text_token!("|")),
            discard!(eoi),
            discard!(peek!(text_token!("\n")))
        ),
        (normalize_name(ingredient))
    )

);

make_fn!(
    pub recipe<StrIter, Recipe>,
    do_each!(
        title => must!(title),
        _ => optional!(para_separator),
        desc => optional!(do_each!(
            _ => peek!(not!(step_prefix)),
            desc => description,
            (desc)
        )),
        _ => optional!(para_separator),
        steps => step_list,
        (Recipe::new(title, desc).with_steps(steps))
    )
);

make_fn!(
    pub title<StrIter, &str>,
    do_each!(
        _ => text_token!("title:"),
        _ => optional!(ws),
        title => until!(text_token!("\n")),
        _ => text_token!("\n"),
        (title)
    )
);

make_fn!(
    para_separator<StrIter, &str>,
    do_each!(
        _ => text_token!("\n"),
        _ => optional!(ws),
        _ => text_token!("\n"),
        ("")
    )
);

make_fn!(
    pub description<StrIter, &str>,
    until!(either!(
        discard!(para_separator),
        eoi,
    ))
);

make_fn!(
    pub step_time<StrIter, Duration>,
    do_each!(
        cnt => num,
        _ => optional!(ws),
        u => either!(
            text_token!("ms"),
            text_token!("sec"),
            text_token!("s"),
            text_token!("min"),
            text_token!("m"),
            text_token!("hrs"),
            text_token!("hr"),
            text_token!("h")
        ),
        (
            Duration::from_secs(
                match u {
                    "ms" => cnt / 1000,
                    "s" | "sec" => cnt.into(),
                    "m" | "min" => dbg!(cnt) * 60,
                    "h" | "hr" | "hrs" => cnt * 60 * 60,
                    _ => unreachable!(),
                }.into()
            )
        )
    )
);

make_fn!(
    pub step_prefix<StrIter, Option<Duration>>,
    do_each!(
        _ => text_token!("step:"),
        dur => optional!(do_each!(
            _ => ws,
            dur => step_time,
            (dbg!(dur))
        )),
        _ => optional!(ws),
        _ => para_separator,
        (dur)
    )
);

make_fn!(
    pub step<StrIter, Step>,
    do_each!(
        dur => step_prefix,
        ingredients => with_err!(must!(ingredient_list), "Missing ingredient list"),
        _ => para_separator,
        desc => description,
        _ => either!(discard!(para_separator), eoi),
        (Step::new(dur, desc).with_ingredients(ingredients))
    )
);

make_fn!(
    pub step_list<StrIter, Vec<Step>>,
    do_each!(
        first_step => with_err!(must!(step), "Missing recipe steps"),
        rest => repeat!(step),
        ({
            let mut steps = vec![first_step];
            steps.extend(rest);
            steps
        })
    )
);

make_fn!(ws<StrIter, &str>,
    do_each!(
        _initial => peek!(either!(
            text_token!(" "),
            text_token!("\t"),
            text_token!("\r"))),
        rest => consume_all!(either!(
            text_token!(" "),
            text_token!("\t"),
            text_token!("\r"))),
        (rest)
    )
);

make_fn!(nonzero<StrIter, ()>,
    peek!(not!(do_each!(
        n => consume_all!(text_token!("0")),
        _ => ws,
        (n)
    )))
);

make_fn!(num<StrIter, u32>,
    do_each!(
        _ => peek!(ascii_digit),
        n => consume_all!(ascii_digit),
        (u32::from_str(n).unwrap())
    )
);

make_fn!(
    pub ratio<StrIter, Ratio<u32>>,
    do_each!(
        // First we assert non-zero numerator
        //_ => nonzero,
        numer => num,
        _ => text_token!("/"),
        denom => num,
        (Ratio::new(numer, denom))
    )
);

make_fn!(unit<StrIter, String>,
    do_each!(
        u => either!(
            text_token!("tsps"),
            text_token!("tsp"),
            text_token!("teaspoons"),
            text_token!("teaspoon"),
            text_token!("tablespoons"),
            text_token!("tablespoon"),
            text_token!("tbsps"),
            text_token!("tbsp"),
            text_token!("floz"),
            text_token!("ml"),
            text_token!("ltr"),
            text_token!("pound"),
            text_token!("pounds"),
            text_token!("lbs"),
            text_token!("lb"),
            text_token!("oz"),
            text_token!("cups"),
            text_token!("cup"),
            text_token!("qrts"),
            text_token!("qrt"),
            text_token!("quarts"),
            text_token!("quart"),
            text_token!("pints"),
            text_token!("pint"),
            text_token!("pnt"),
            text_token!("gals"),
            text_token!("gal"),
            text_token!("cnt"),
            text_token!("kilograms"),
            text_token!("kilogram"),
            text_token!("kg"),
            text_token!("grams"),
            text_token!("gram"),
            text_token!("g")),
        _ => ws,
        (u.to_lowercase().to_singular())
    )
);

make_fn!(
    pub quantity<StrIter, Quantity>,
     either!(
        do_each!(
            whole => num,
            _ => ws,
            frac => ratio,
            _ => ws,
            (Quantity::Whole(whole) + Quantity::Frac(frac))
        ),
        do_each!(
            frac => ratio,
            _ => ws,
            (Quantity::Frac(frac))
        ),
        do_each!(
            whole => num,
            _ => ws,
            (Quantity::whole(whole))
        )
    )
);

make_fn!(
    pub measure_parts<StrIter, (Quantity, Option<String>)>,
    do_each!(
        qty => quantity,
        unit => optional!(unit),
        ((qty, unit))
    )
);

pub fn measure(i: StrIter) -> abortable_parser::Result<StrIter, Measure> {
    match measure_parts(i) {
        Result::Complete(i, (qty, unit)) => {
            let count = Count(qty.clone());
            return Result::Complete(
                i.clone(),
                unit.map(|s| match s.as_str() {
                    "tbsp" | "tablespoon" => Volume(Tbsp(qty)),
                    "tsp" | "teaspoon" => Volume(Tsp(qty)),
                    "floz" => Volume(Floz(qty)),
                    "ml" => Volume(ML(qty)),
                    "ltr" | "liter" => Volume(Ltr(qty)),
                    "cup" | "cp" => Volume(Cup(qty)),
                    "qrt" | "quart" => Volume(Qrt(qty)),
                    "pint" | "pnt" => Volume(Pint(qty)),
                    "gal" => Volume(Gal(qty)),
                    "cnt" | "count" => Count(qty),
                    "lb" | "pound" => Weight(Pound(qty)),
                    "oz" => Weight(Oz(qty)),
                    "kg" | "kilogram" => Weight(Kilogram(qty)),
                    "g" | "gram" => Weight(Gram(qty)),
                    _u => {
                        eprintln!("Invalid unit: {}", _u);
                        unreachable!()
                    }
                })
                .unwrap_or(count),
            );
        }
        Result::Fail(e) => {
            return Result::Fail(e);
        }
        Result::Abort(e) => {
            return Result::Abort(e);
        }
        Result::Incomplete(i) => return Result::Incomplete(i),
    }
}

pub fn normalize_name(name: &str) -> String {
    let parts: Vec<&str> = name.split_whitespace().collect();
    if parts.len() >= 2 {
        let mut prefix = parts[0..parts.len() - 1].join(" ");
        // NOTE(jwall): The below unwrap is safe because of the length
        // check above.
        let last = parts.last().unwrap();
        let normalized = last.to_singular();
        prefix.push(' ');
        prefix.push_str(&normalized);
        return prefix;
    }
    return name.trim().to_lowercase().to_owned();
}

make_fn!(
    pub ingredient_name<StrIter, String>,
    do_each!(
        name => until!(either!(
            discard!(text_token!("\n")),
            eoi,
            discard!(text_token!("(")))),
        (normalize_name(name))
    )
);

make_fn!(
    ingredient_modifier<StrIter, &str>,
    do_each!(
        _ => text_token!("("),
        modifier => must!(until!(text_token!(")"))),
        _ => must!(text_token!(")")),
        (modifier)
    )
);

make_fn!(
    pub ingredient<StrIter, Ingredient>,
    do_each!(
        _ => optional!(ws),
        measure => measure,
        name => ingredient_name,
        modifier => optional!(ingredient_modifier),
        _ => optional!(ws),
        (Ingredient::new(name, modifier.map(|s| s.to_owned()), measure, String::new()))
    )
);

make_fn!(
    pub ingredient_list<StrIter, Vec<Ingredient>>,
    separated!(text_token!("\n"), ingredient)
);
