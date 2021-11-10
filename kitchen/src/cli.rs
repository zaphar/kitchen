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
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::path::Path;

use recipes::{parse, Recipe};

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

pub fn parse_recipe<P>(path: P) -> Result<Recipe, ParseError>
where
    P: AsRef<Path>,
{
    let mut br = BufReader::new(File::open(path)?);
    let mut buf = Vec::new();
    br.read_to_end(&mut buf)?;
    let i = String::from_utf8_lossy(&buf).to_string();
    Ok(parse::as_recipe(&i)?)
}
