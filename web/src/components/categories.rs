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
use crate::{
    app_state::{Message, StateHandler},
    js_lib::get_element_by_id,
};
use sycamore::{futures::spawn_local_scoped, prelude::*};
use tracing::{debug, error, instrument};
use web_sys::HtmlDialogElement;

use recipes::parse;

fn get_error_dialog() -> HtmlDialogElement {
    get_element_by_id::<HtmlDialogElement>("error-dialog")
        .expect("error-dialog isn't an html dialog element!")
        .expect("No error-dialog element present")
}

fn check_category_text_parses(unparsed: &str, error_text: &Signal<String>) -> bool {
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

#[instrument(skip_all)]
#[component]
pub fn Categories<'ctx, G: Html>(cx: Scope<'ctx>, sh: StateHandler<'ctx>) -> View<G> {
    let error_text = create_signal(cx, String::new());
    let category_text: &Signal<String> = create_signal(cx, String::new());
    let dirty = create_signal(cx, false);

    spawn_local_scoped(cx, {
        let store = crate::api::HttpStore::get_from_context(cx);
        async move {
            if let Some(js) = store
                .fetch_categories()
                .await
                .expect("Failed to get categories.")
            {
                category_text.set(js);
            };
        }
    });

    let dialog_view = view! {cx,
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
    };

    view! {cx,
        (dialog_view)
        textarea(bind:value=category_text, rows=20, on:change=move |_| {
            dirty.set(true);
        })
        span(role="button", on:click=move |_| {
            check_category_text_parses(category_text.get().as_str(), error_text);
        }) { "Check" } " "
        span(role="button", on:click=move |_| {
            if !*dirty.get() {
                return;
            }
            if check_category_text_parses(category_text.get().as_str(), error_text) {
                debug!("triggering category save");
                sh.dispatch(
                    cx,
                    Message::SetCategoryMap(category_text.get_untracked().as_ref().clone()),
                );
            }
        }) { "Save" }
    }
}
