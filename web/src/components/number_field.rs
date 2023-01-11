// Copyright 2023 Jeremy Wall (Jeremy@marzhilsltudios.com)
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
use sycamore::prelude::*;
use tracing::debug;
use web_sys::{Event, HtmlInputElement};

use crate::js_lib;

#[derive(Props)]
pub struct NumberProps<'ctx, F>
where
    F: Fn(Event),
{
    name: String,
    on_change: Option<F>,
    min: i32,
    counter: &'ctx Signal<String>,
}

#[component]
pub fn NumberField<'ctx, F, G: Html>(cx: Scope<'ctx>, props: NumberProps<'ctx, F>) -> View<G>
where
    F: Fn(web_sys::Event) + 'ctx,
{
    let NumberProps {
        name,
        on_change,
        min,
        counter,
    } = props;

    let id = name.clone();
    let inc_target_id = id.clone();
    let dec_target_id = id.clone();
    let min_field = format!("{}", min);

    view! {cx,
        div() {
            input(type="number", id=id, name=name, class="item-count-sel", min=min_field, max="99", step="1", bind:value=counter, on:input=move |evt| {
                on_change.as_ref().map(|f| f(evt));
            })
            span(class="item-count-inc-dec", on:click=move |_| {
                let i: i32 = counter.get_untracked().parse().unwrap();
                let target = js_lib::get_element_by_id::<HtmlInputElement>(&inc_target_id).unwrap().expect(&format!("No such element with id {}", inc_target_id));
                counter.set(format!("{}", i+1));
                debug!(counter=%(counter.get_untracked()), "set counter to new value");
                // We force an input event to get triggered for our target.
                target.dispatch_event(&web_sys::Event::new("input").expect("Failed to create new event")).expect("Failed to dispatch event to target");
            }) { "▲" }
            " "
            span(class="item-count-inc-dec", on:click=move |_| {
                let i: i32 = counter.get_untracked().parse().unwrap();
                let target = js_lib::get_element_by_id::<HtmlInputElement>(&dec_target_id).unwrap().expect(&format!("No such element with id {}", dec_target_id));
                if i > min {
                    counter.set(format!("{}", i-1));
                    debug!(counter=%(counter.get_untracked()), "set counter to new value");
                    // We force an input event to get triggered for our target.
                    target.dispatch_event(&web_sys::Event::new("input").expect("Failed to create new event")).expect("Failed to dispatch event to target");
                }
            }) { "▼" }
        }
    }
}
