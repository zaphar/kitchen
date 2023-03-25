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
// limitations under the License.\
use sycamore::{easing, motion, prelude::*};
use tracing::debug;
use wasm_bindgen::UnwrapThrowExt;

const SECTION_ID: &'static str = "toast-container";

#[component]
pub fn Container<'a, G: Html>(cx: Scope<'a>) -> View<G> {
    view! {cx,
        section(id=SECTION_ID) { }
    }
}

pub fn create_output_element(msg: &str, class: &str) -> web_sys::Element {
    let document = web_sys::window()
        .expect("No window present")
        .document()
        .expect("No document in window");
    let output = document.create_element("output").unwrap_throw();
    let message_node = document.create_text_node(msg);
    output.set_attribute("class", class).unwrap_throw();
    output.set_attribute("role", "status").unwrap_throw();
    output.append_child(&message_node).unwrap_throw();
    output
}

fn show_toast<'a>(cx: Scope<'a>, msg: &str, class: &str, timeout: Option<chrono::Duration>) {
    let timeout = timeout.unwrap_or_else(|| chrono::Duration::seconds(3));
    // Insert a toast output element into the container.
    let tweened = motion::create_tweened_signal(
        cx,
        0.0 as f32,
        timeout
            .to_std()
            .expect("Failed to convert timeout duration."),
        easing::quad_in,
    );
    tweened.set(1.0);
    create_effect_scoped(cx, move |_cx| {
        if !tweened.is_tweening() {
            debug!("Detected message timeout.");
            let container = crate::js_lib::get_element_by_id::<web_sys::HtmlElement>(SECTION_ID)
                .expect("Failed to get toast-container")
                .expect("No toast-container");
            if let Some(node_to_remove) = container.first_element_child() {
                // Always remove the first child if there is one.
                container.remove_child(&node_to_remove).unwrap_throw();
            }
        }
    });
    let output_element = create_output_element(msg, class);
    crate::js_lib::get_element_by_id::<web_sys::HtmlElement>(SECTION_ID)
        .expect("Failed to get toast-container")
        .expect("No toast-container")
        // Always append after the last child.
        .append_child(&output_element)
        .unwrap_throw();
}

pub fn message<'a>(cx: Scope<'a>, msg: &str, timeout: Option<chrono::Duration>) {
    show_toast(cx, msg, "toast", timeout);
}

pub fn error_message<'a>(cx: Scope<'a>, msg: &str, timeout: Option<chrono::Duration>) {
    show_toast(cx, msg, "toast error", timeout);
}
