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

use recipes::{parse, Recipe};

#[derive(Debug)]
pub enum ParseError {
    IO(std::io::Error),
    Syntax(String),
}

macro_rules! try_open {
    ($path:expr) => {
        match File::open(&$path) {
            Ok(reader) => reader,
            Err(e) => {
                eprintln!("Error opening file for read: {:?}", $path);
                return Err(ParseError::from(e));
            }
        }
    };
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        // TODO(jwall): This error should allow us to collect more information
        // about the cause of the error.
        ParseError::IO(err)
    }
}

impl From<String> for ParseError {
    fn from(s: String) -> Self {
        ParseError::Syntax(s)
    }
}

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

pub fn read_menu_list<P>(path: P) -> Result<Vec<Recipe>, ParseError>
where
    P: AsRef<Path> + Debug,
{
    let path = path.as_ref();
    let wd = path.parent().unwrap();
    let mut br = BufReader::new(try_open!(path));
    eprintln!("Switching to {:?}", wd);
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
