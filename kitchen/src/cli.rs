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
use std::convert::Into;
use std::convert::TryInto;

use rustyline::error::ReadlineError;
use rustyline::Editor;

use recipes::{Ingredient, Recipe, Step};

pub enum CliResponse<T> {
    Interrupt,
    EndOfInput,
    ReadItem(T),
}
use CliResponse::*;

macro_rules! try_cli_resp {
    ($res:expr) => {
        match $res {
            ReadItem(item) => item,
            EndOfInput => return Ok(EndOfInput),
            Interrupt => return Ok(Interrupt),
        }
    };
}

macro_rules! handle_yes_no {
    ($rl:expr, $( $rest:tt )+) => {
        match try_cli_resp!(read_prompt($rl, "Y/n: ")?).as_str() {
            "y" | "Y" | "yes" | "" => {
                $( $rest )*
            }
            _ => {
                break
            }
        }
    };
}

fn read_prompt(rl: &mut Editor<()>, prompt: &str) -> Result<CliResponse<String>, String> {
    Ok(match rl.readline(prompt) {
        Ok(line) => ReadItem(line),
        Err(ReadlineError::Interrupted) => Interrupt,
        Err(ReadlineError::Eof) => EndOfInput,
        Err(e) => return Err(e.to_string()),
    })
}

fn read_new_ingredient(rl: &mut Editor<()>) -> Result<CliResponse<Option<Ingredient>>, String> {
    let read_item = try_cli_resp!(read_prompt(rl, "> ")?);
    Ok(if read_item.is_empty() {
        ReadItem(None)
    } else {
        ReadItem(Some(Ingredient::parse(&read_item)?))
    })
}

fn read_ingredients(rl: &mut Editor<()>) -> Result<CliResponse<Vec<Ingredient>>, String> {
    println!("Enter Ingredients in the following form below: <amt> <unit> <name> [(modifier)]");
    println!("<Ctrl-C> or enter an empty line to stop entering ingredients");
    let mut ingredient_list = Vec::new();
    loop {
        match read_new_ingredient(rl)? {
            Interrupt => break,
            EndOfInput => return Ok(EndOfInput),
            ReadItem(None) => break,
            ReadItem(Some(ingredient)) => ingredient_list.push(ingredient),
        }
    }
    Ok(ReadItem(ingredient_list))
}

fn read_new_step(rl: &mut Editor<()>) -> Result<CliResponse<Step>, String> {
    println!("Enter Recipe Step details below");
    let instructions = try_cli_resp!(read_prompt(rl, "Step Instructions: ")?);
    let ingredients = try_cli_resp!(read_ingredients(rl)?);
    let mut step = Step::new(None, instructions);
    step.add_ingredients(ingredients);
    Ok(ReadItem(step))
}

fn read_steps(rl: &mut Editor<()>) -> Result<CliResponse<Vec<Step>>, String> {
    let mut steps = Vec::new();
    loop {
        println!("Enter a recipe step?");
        handle_yes_no! {rl,
            let step = try_cli_resp!(read_new_step(rl)?);
            steps.push(step);
        };
    }
    Ok(ReadItem(steps))
}

//pub fn read_new_recipe(rl: &mut Editor<()>) -> Result<CliResponse<Recipe>, String> {
//    println!("Enter recipe details below.");
//    let title = try_cli_resp!(read_prompt(rl, "Title: ")?);
//    let desc = try_cli_resp!(read_prompt(rl, "Description: ")?);
//    let steps = try_cli_resp!(read_steps(rl)?);
//    let mut recipe = Recipe::new(title, desc);
//    recipe.add_steps(steps);
//    Ok(ReadItem(recipe))
//}

//fn read_loop<S: RecipeStore>(rl: &mut Editor<()>, store: S) -> Result<CliResponse<()>, String> {
//    loop {
//        println!("Enter a recipe?");
//        handle_yes_no! {rl,
//            let recipe = try_cli_resp!(read_new_recipe(rl)?);
//            // TODO Store this recipe
//            store.store_recipe(&recipe)?;
//        };
//    }
//    Ok(ReadItem(()))
//}

//pub fn main_impl<Factory, Err>(factory: Factory)
//where
//    Factory: TryInto<Error = Err>,
//    Err: std::fmt::Debug,
//{
//    let mut rl = Editor::<()>::new();
//    let store = factory.try_into().unwrap();
//    // TODO(jwall): handle history in a cross platform way?
//    //read_loop(&mut rl, store).unwrap();
//}
