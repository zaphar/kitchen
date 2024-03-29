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
use std::env;
use std::io;
use std::net::SocketAddr;
use std::path::PathBuf;

use clap;
use clap::ArgMatches;
use clap::{clap_app, crate_authors, crate_version};
use tracing::{error, info, instrument, warn, Level};
use tracing_subscriber::FmtSubscriber;

mod cli;
mod web;

fn create_app<'a>() -> clap::App<'a> {
    clap_app!(kitchen =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: "Kitchen Management CLI")
        (@arg verbose: --verbose -v +takes_value "Verbosity level for logging (error, warn, info, debug, trace")
        (@subcommand recipe =>
            (about: "parse a recipe file and output info about it")
            (@arg ingredients: -i --ingredients "Output the ingredients list.")
            (@arg INPUT: +required "Input recipe file to parse")
        )
        (@subcommand groceries =>
            (about: "print out a grocery list for a set of recipes")
            (@arg csv: --csv "output ingredients as csv")
            (@arg INPUT: +required "Input menu file to parse. One recipe file per line.")
        )
        (@subcommand serve =>
            (about: "Serve the interface via the web")
            (@arg recipe_dir: -d --dir +takes_value "Directory containing recipe files to use")
            (@arg session_dir: --session_dir +takes_value +required "Session store directory to use")
            (@arg tls: --tls "Use TLS to serve.")
            (@arg cert_path: --cert +takes_value "Certificate path. Required if you specified --tls.")
            (@arg key_path: --cert_key +takes_value "Certificate key path. Required if you specified --tls")
            (@arg listen: --listen +takes_value "address and port to listen on 0.0.0.0:3030")
        )
        (@subcommand add_user =>
            (about: "add users to to the interface")
            (@arg recipe_dir: -d --dir +takes_value "Directory containing recipe files to load for user")
            (@arg user: -u --user +takes_value +required "username to add")
            (@arg pass: -p --pass +takes_value +required "password to add for this user")
            (@arg session_dir: --session_dir +takes_value +required "Session store directory to use")
        )
    )
    .setting(clap::AppSettings::SubcommandRequiredElseHelp)
}

fn get_session_store_path(matches: &ArgMatches) -> PathBuf {
    if let Some(dir) = matches.value_of("session_dir") {
        PathBuf::from(dir)
    } else {
        let mut dir = std::env::var("HOME")
            .map(PathBuf::from)
            .expect("Unable to get user home directory. Bailing out.");
        dir.push(".kitchen");
        dir
    }
}

#[instrument]
fn main() {
    let matches = create_app().get_matches();
    let subscriber_builder = if let Some(verbosity) = matches.value_of("verbose") {
        // Se want verbosity level
        let level = match verbosity {
            "error" | "ERROR" => Level::ERROR,
            "warn" | "WARN" => Level::WARN,
            "info" | "INFO" => Level::INFO,
            "debug" | "DEBUG" => Level::DEBUG,
            "trace" | "TRACE" => Level::TRACE,
            _ => {
                println!("Invalid logging level using TRACE");
                Level::TRACE
            }
        };
        FmtSubscriber::builder().with_max_level(level)
    } else {
        FmtSubscriber::builder().with_max_level(Level::INFO)
    };
    tracing::subscriber::set_global_default(subscriber_builder.with_writer(io::stderr).finish())
        .expect("setting default subscriber failed");

    if let Some(matches) = matches.subcommand_matches("recipe") {
        // The input argument is required so if we made it here then it's safe to unrwap this value.
        let recipe_file = matches.value_of("INPUT").unwrap();
        match cli::parse_recipe(recipe_file) {
            Ok(r) => {
                cli::output_recipe_info(r, matches.is_present("ingredients"));
            }
            Err(err) => {
                error!(?err);
            }
        }
    } else if let Some(matches) = matches.subcommand_matches("groceries") {
        // The input argument is required so if we made it here then it's safe to unrwap this value.
        let menu_file = matches.value_of("INPUT").unwrap();
        match cli::read_menu_list(menu_file) {
            Ok(rs) => {
                if matches.is_present("csv") {
                    cli::output_ingredients_csv(rs);
                } else {
                    cli::output_ingredients_list(rs);
                }
            }
            Err(err) => {
                error!(?err);
            }
        }
    } else if let Some(matches) = matches.subcommand_matches("serve") {
        let recipe_dir_path = if let Some(dir) = matches.value_of("recipe_dir") {
            PathBuf::from(dir)
        } else {
            std::env::current_dir().expect("Unable to get current directory. Bailing out.")
        };
        let session_store_path: PathBuf = get_session_store_path(matches);
        let listen_socket: SocketAddr = if let Some(listen_socket) = matches.value_of("listen") {
            listen_socket.parse().expect(&format!(
                "--listen must be of the form <addr>:<port> but got {}",
                listen_socket
            ))
        } else {
            "127.0.0.1:3030".parse().unwrap()
        };
        info!(listen=%listen_socket, "Launching web interface...");
        async_std::task::block_on(async {
            if matches.contains_id("tls") {
                web::ui_main_tls(
                    recipe_dir_path,
                    session_store_path,
                    listen_socket,
                    matches
                        .value_of("cert_path")
                        .expect("You must provide a cert path with --cert"),
                    matches
                        .value_of("key_path")
                        .expect("You must provide a key path with --cert_key"),
                )
                .await
            } else {
                web::ui_main(recipe_dir_path, session_store_path, listen_socket).await
            }
        });
    } else if let Some(matches) = matches.subcommand_matches("add_user") {
        let recipe_dir_path = matches.value_of("recipe_dir").map(|dir| PathBuf::from(dir));
        let session_store_path: PathBuf = get_session_store_path(matches);
        async_std::task::block_on(async {
            web::add_user(
                session_store_path,
                matches.value_of("user").unwrap().to_owned(),
                matches.value_of("pass").unwrap().to_owned(),
                recipe_dir_path,
            )
            .await;
        });
    }
}
