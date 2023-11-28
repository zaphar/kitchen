// Copyright 2022 Jeremy Wall
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
use sycamore::{futures::spawn_local_scoped, prelude::*};
use tracing::{debug, error};

use crate::app_state::{Message, StateHandler};
use crate::js_lib;
use recipes::{self, parse};

fn check_ingredients_parses(
    text: &str,
    error_text: &Signal<String>,
    aria_hint: &Signal<&'static str>,
) -> bool {
    if let Err(e) = parse::as_ingredient_list(text) {
        error!(?e, "Error parsing recipe");
        error_text.set(e);
        aria_hint.set("true");
        false
    } else {
        error_text.set(String::from("No parse errors..."));
        aria_hint.set("false");
        true
    }
}

#[derive(Props)]
pub struct IngredientComponentProps<'ctx> {
    sh: StateHandler<'ctx>,
}

#[component]
pub fn IngredientsEditor<'ctx, G: Html>(
    cx: Scope<'ctx>,
    props: IngredientComponentProps<'ctx>,
) -> View<G> {
    let IngredientComponentProps { sh } = props;
    let store = crate::api::HttpStore::get_from_context(cx);
    let text = create_signal(cx, String::new());
    let error_text = create_signal(cx, String::from("Parse results..."));
    let aria_hint = create_signal(cx, "false");

    spawn_local_scoped(cx, {
        let store = store.clone();
        async move {
            let entry = store
                .fetch_staples()
                .await
                .expect("Failure getting staples");
            if let Some(entry) = entry {
                check_ingredients_parses(entry.as_str(), error_text, aria_hint);
                text.set(entry);
            } else {
                error_text.set("Unable to find staples".to_owned());
            }
        }
    });

    let dirty = create_signal(cx, false);
    let ts = create_signal(cx, js_lib::get_ms_timestamp());

    debug!("creating editor view");
    view! {cx,
        div {
            textarea(class="width-third", bind:value=text, aria-invalid=aria_hint.get(), rows=20, on:change=move |_| {
                dirty.set(true);
            }, on:input=move |_| {
                let current_ts = js_lib::get_ms_timestamp();
                if (current_ts - *ts.get_untracked()) > 100 {
                    check_ingredients_parses(text.get_untracked().as_str(), error_text, aria_hint);
                    ts.set(current_ts);
                }
            })
            div(class="parse") { (error_text.get()) }
        }
        button(on:click=move |_| {
            let unparsed = text.get();
            if !*dirty.get_untracked() {
                debug!("Staples text is unchanged");
                return;
            }
            debug!("triggering a save");
            if check_ingredients_parses(unparsed.as_str(), error_text, aria_hint) {
                debug!("Staples text is changed");
                sh.dispatch(cx, Message::UpdateStaples(unparsed.as_ref().clone(), None));
            }
        }) { "Save" }
    }
}
