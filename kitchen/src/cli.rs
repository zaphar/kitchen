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
use std::fmt::Debug;
use std::fs::File;
use std::io::Read;
use std::io::{BufRead, BufReader};
use std::path::Path;

use csv;

use recipes::{parse, IngredientAccumulator, Recipe};
use tracing::{error, info, instrument, warn};

#[derive(Debug)]
pub enum ParseError {
    IO(std::io::Error),
    Syntax(String),
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::IO(err)
    }
}

impl From<String> for ParseError {
    fn from(s: String) -> Self {
        ParseError::Syntax(s)
    }
}
// TODO(jwall): We should think a little more closely about
// the error modeling for this application.
macro_rules! try_open {
    ($path:expr) => {
        match File::open(&$path) {
            Ok(reader) => reader,
            Err(e) => {
                error!(path=?$path, "Error opening file for read");
                return Err(ParseError::from(e));
            }
        }
    };
}

#[instrument]
pub fn parse_recipe<P>(path: P) -> Result<Recipe, ParseError>
where
    P: AsRef<Path> + Debug,
{
    let mut br = BufReader::new(try_open!(path));
    let mut buf = Vec::new();
    let sz = br.read_to_end(&mut buf)?;
    let i = String::from_utf8_lossy(&buf[0..sz]).to_string();
    Ok(parse::as_recipe(&i)?)
}

#[instrument]
pub fn read_menu_list<P>(path: P) -> Result<Vec<Recipe>, ParseError>
where
    P: AsRef<Path> + Debug,
{
    let path = path.as_ref();
    let wd = path.parent().unwrap();
    let mut br = BufReader::new(try_open!(path));
    info!(directory=?wd, "Switching working directory");
    std::env::set_current_dir(wd)?;
    let mut buf = String::new();
    let mut recipe_list = Vec::new();
    loop {
        let sz = br.read_line(&mut buf)?;
        if sz == 0 {
            break;
        }
        let recipe = parse_recipe(buf.trim())?;
        buf.clear();
        recipe_list.push(recipe);
    }
    Ok(recipe_list)
}

pub fn output_recipe_info(r: Recipe, print_ingredients: bool) {
    println!("Title: {}", r.title);
    println!("");
    if print_ingredients {
        println!("Ingredients:");
        for (_, i) in r.get_ingredients() {
            println!("\t* {} {}", i.amt, i.name);
        }
    }
}

pub fn output_ingredients_list(rs: Vec<Recipe>) {
    let mut acc = IngredientAccumulator::new();
    for r in rs {
        acc.accumulate_from(&r);
    }
    for (_, (i, _)) in acc.ingredients() {
        print!("{}", i.amt.normalize());
        println!(" {}", i.name);
    }
}

pub fn output_ingredients_csv(rs: Vec<Recipe>) {
    let mut acc = IngredientAccumulator::new();
    for r in rs {
        acc.accumulate_from(&r);
    }
    let out = std::io::stdout();
    let mut writer = csv::Writer::from_writer(out);
    for (_, (i, _)) in acc.ingredients() {
        writer
            .write_record(&[format!("{}", i.amt.normalize()), i.name])
            .expect("Failed to write csv.");
    }
}
