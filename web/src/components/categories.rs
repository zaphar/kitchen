// Copyright 2022 Jeremy Wall (Jeremy@marzhilsltudios.com)
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
use serde_json::{from_str, to_string};
use sycamore::{futures::spawn_local_in_scope, prelude::*};
use tracing::{debug, error, instrument};
use web_sys::HtmlDialogElement;

use recipes::parse;

use crate::{js_lib::get_element_by_id, service::get_appservice_from_context};

fn get_error_dialog() -> HtmlDialogElement {
    get_element_by_id::<HtmlDialogElement>("error-dialog")
        .expect("error-dialog isn't an html dialog element!")
        .unwrap()
}

fn check_category_text_parses(unparsed: &str, error_text: Signal<String>) -> bool {
    let el = get_error_dialog();
    if let Err(e) = parse::as_categories(unparsed) {
        error!(?e, "Error parsing categories");
        error_text.set(e);
        el.show();
        false
    } else {
        el.close();
        true
    }
}

#[instrument]
#[component(Categories<G>)]
pub fn categories() -> View<G> {
    let app_service = get_appservice_from_context();
    let save_signal = Signal::new(());
    let error_text = Signal::new(String::new());
    let category_text = Signal::new(
        match app_service
            .get_category_text()
            .expect("Failed to get categories.")
        {
            Some(js) => from_str::<String>(&js)
                .map_err(|e| format!("{}", e))
                .expect("Failed to parse categories as json"),
            None => String::new(),
        }, //.unwrap_or_else(|| String::new()),
    );

    create_effect(
        cloned!((app_service, category_text, save_signal, error_text) => move || {
            // TODO(jwall): This is triggering on load which is not desired.
            save_signal.get();
            spawn_local_in_scope({
                cloned!((app_service, category_text, error_text) => async move {
                    // TODO(jwall): Save the categories.
                    if let Err(e) = app_service.save_categories(category_text.get_untracked().as_ref().clone()).await {
                        error!(?e, "Failed to save categories");
                        error_text.set(format!("{:?}", e));
                    }
                })
            });
        }),
    );

    let dialog_view = cloned!((error_text) => view! {
        dialog(id="error-dialog") {
            article{
                header {
                    a(href="#", on:click=|_| {
                        let el = get_error_dialog();
                        el.close();
                    }, class="close")
                    "Invalid Categories"
                }
                p {
                    (error_text.get().clone())
                }
            }
        }
    });

    cloned!((category_text, error_text) => view! {
        (dialog_view)
        textarea(bind:value=category_text.clone(), rows=20)
        a(role="button", href="#", on:click=cloned!((category_text, error_text) => move |_| {
            check_category_text_parses(category_text.get().as_str(), error_text.clone());
        })) { "Check" } " "
        a(role="button", href="#", on:click=cloned!((category_text, error_text) => move |_| {
            // TODO(jwall): check and then save the categories.
            if check_category_text_parses(category_text.get().as_str(), error_text.clone()) {
                debug!("triggering category save");
                save_signal.trigger_subscribers();
            }
        })) { "Save" }
    })
}
