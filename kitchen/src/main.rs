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
mod cli;

use std::env;

use recipes::Recipe;

use clap;
use clap::{clap_app, crate_authors, crate_version};

fn create_app<'a, 'b>() -> clap::App<'a, 'b>
where
    'a: 'b,
{
    clap_app!(kitchen =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: "Kitchen Management CLI")
        (@subcommand recipe =>
            (about: "parse a recipe file and output info about it")
            (@arg ingredients: -i --ingredients "Output the ingredients list.")
            (@arg INPUT: +required "Input recipe file to parse")
        )
    )
    .setting(clap::AppSettings::SubcommandRequiredElseHelp)
}

fn output_recipe_info(r: Recipe, print_ingredients: bool) {
    println!("Title: {}", r.title);
    println!("");
    if print_ingredients {
        println!("Ingredients:");
        for (_, ing) in r.get_ingredients() {
            print!("\t* {}", ing.amt);
            println!(" {}", ing.name);
        }
    }
}

fn main() {
    let matches = create_app().get_matches();
    if let Some(matches) = matches.subcommand_matches("recipe") {
        // The input argument is required so if we made it here then it's safe to unrwap this value.
        let recipe_file = matches.value_of("INPUT").unwrap();
        match cli::parse_recipe(recipe_file) {
            Ok(r) => {
                // TODO(jwall): handle our recipe dump
                output_recipe_info(r, matches.is_present("ingredients"));
            }
            Err(e) => {
                eprintln!("{:?}", e);
            }
        }
    }
}
