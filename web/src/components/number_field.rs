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
use maud::html;
use sycamore::prelude::*;
use tracing::{debug, error};
use wasm_bindgen::{JsCast, JsValue};
use wasm_web_component::{web_component, WebComponentBinding};
use web_sys::{CustomEvent, CustomEventInit, Event, HtmlElement, InputEvent, ShadowRoot};

#[web_component(
    observed_attrs = "['val', 'min', 'max', 'step']",
    observed_events = "['change', 'click', 'input']"
)]
pub struct NumberSpinner {
    root: Option<ShadowRoot>,
    min: i32,
    max: i32,
    step: i32,
    value: i32,
}

impl NumberSpinner {
    fn get_input_el(&self) -> HtmlElement {
        self.root
            .as_ref()
            .unwrap()
            .get_element_by_id("nval")
            .unwrap()
            .dyn_into()
            .unwrap()
    }
}

impl WebComponentBinding for NumberSpinner {
    fn init_mut(&mut self, element: &web_sys::HtmlElement) {
        (self.min, self.max, self.step, self.value) = (0, 99, 1, 0);
        debug!("Initializing element instance");
        let root = html! {
            span {
                link rel="stylesheet" href="/ui/static/app.css" { };
                style {
                    r#"
                        span { display: block; }
                        span.button {
                            font-size: 2em; font-weight: bold;
                        }
                        .number-input {
                            border-width: var(--border-width);
                            border-style: inset;
                            padding: 3pt;
                            border-radius: 10px;
                            width: 3em;
                        }
                    "#
                };
                span class="button" id="inc" { "+" }; " "
                // TODO(jwall): plaintext-only would be nice but I can't actually do that yet.
                span id="nval" class="number-input" contenteditable="true" { "0" } " "
                span class="button" id="dec" { "-" };
            };
        };
        self.attach_shadow(element, &root.into_string());
        self.root = element.shadow_root();
    }

    fn connected_mut(&mut self, element: &HtmlElement) {
        debug!("COUNTS: connecting to DOM");
        let val = element.get_attribute("val").unwrap_or_else(|| "0".into());
        let min = element.get_attribute("min").unwrap_or_else(|| "0".into());
        let max = element.get_attribute("max").unwrap_or_else(|| "99".into());
        let step = element.get_attribute("step").unwrap_or_else(|| "1".into());
        debug!(?val, ?min, ?max, ?step, "connecting to DOM");
        let nval_el = self.get_input_el();
        if let Ok(parsed) = val.parse::<i32>() {
            self.value = parsed;
            nval_el.set_inner_text(&val);
        }
        if let Ok(parsed) = min.parse::<i32>() {
            self.min = parsed;
        }
        if let Ok(parsed) = max.parse::<i32>() {
            self.max = parsed;
        }
        if let Ok(parsed) = step.parse::<i32>() {
            self.step = parsed;
        }
    }

    fn handle_event_mut(&mut self, element: &web_sys::HtmlElement, event: &Event) {
        let target: HtmlElement = event.target().unwrap().dyn_into().unwrap();
        let id = target.get_attribute("id");
        let event_type = event.type_();
        let nval_el = self.get_input_el();
        debug!(?id, ?event_type, "saw event");
        match (id.as_ref().map(|s| s.as_str()), event_type.as_str()) {
            (Some("inc"), "click") => {
                if self.value < self.max {
                    self.value += 1;
                    nval_el.set_inner_text(&format!("{}", self.value));
                }
            }
            (Some("dec"), "click") => {
                if self.value > self.min {
                    self.value -= 1;
                    nval_el.set_inner_text(&format!("{}", self.value));
                }
            }
            (Some("nval"), "input") => {
                let input_event = event.dyn_ref::<InputEvent>().unwrap();
                if let Some(data) = input_event.data() {
                    // We only allow numeric input data here.
                    debug!(data, input_type=?input_event.input_type() , "got input");
                    if data.chars().filter(|c| !c.is_numeric()).count() > 0 {
                        nval_el.set_inner_text(&format!("{}", self.value));
                    }
                } else {
                    nval_el.set_inner_text(&format!("{}{}", nval_el.inner_text(), self.value));
                }
            }
            _ => {
                debug!("Ignoring event");
                return;
            }
        };
        let mut event_dict = CustomEventInit::new();
        event_dict.detail(&JsValue::from_f64(self.value as f64));
        element
            .dispatch_event(&CustomEvent::new_with_event_init_dict("updated", &event_dict).unwrap())
            .unwrap();
        debug!("Dispatched updated event");
    }

    fn attribute_changed_mut(
        &mut self,
        _element: &web_sys::HtmlElement,
        name: JsValue,
        old_value: JsValue,
        new_value: JsValue,
    ) {
        let nval_el = self.get_input_el();
        let name = name.as_string().unwrap();
        debug!(
            ?name,
            ?old_value,
            ?new_value,
            "COUNTS: handling attribute change"
        );
        match name.as_str() {
            "val" => {
                debug!("COUNTS: got an updated value");
                if let Some(val) = new_value.as_string() {
                    debug!(val, "COUNTS: got an updated value");
                    if let Ok(val) = val.parse::<i32>() {
                        self.value = val;
                        nval_el.set_inner_text(format!("{}", self.value).as_str());
                    } else {
                        error!(?new_value, "COUNTS: Not a valid f64 value");
                    }
                }
            }
            "min" => {
                if let Some(val) = new_value.as_string() {
                    debug!(val, "COUNTS: got an updated value");
                    if let Ok(val) = val.parse::<i32>() {
                        self.min = val;
                    } else {
                        error!(?new_value, "COUNTS: Not a valid f64 value");
                    }
                }
            }
            "max" => {
                if let Some(val) = new_value.as_string() {
                    debug!(val, "COUNTS: got an updated value");
                    if let Ok(val) = val.parse::<i32>() {
                        self.max = val;
                    } else {
                        error!(?new_value, "COUNTS: Not a valid f64 value");
                    }
                }
            }
            "step" => {
                if let Some(val) = new_value.as_string() {
                    debug!(val, "COUNTS: got an updated value");
                    if let Ok(val) = val.parse::<i32>() {
                        self.step = val;
                    } else {
                        error!(?new_value, "COUNTS: Not a valid f64 value");
                    }
                }
            }
            _ => {
                debug!("Ignoring Attribute Change");
                return;
            }
        }
    }
}

#[derive(Props)]
pub struct NumberProps<'ctx, F>
where
    F: Fn(CustomEvent),
{
    name: String,
    class: String,
    on_change: Option<F>,
    min: f64,
    counter: &'ctx Signal<f64>,
}

#[component]
pub fn NumberField<'ctx, F, G: Html>(cx: Scope<'ctx>, props: NumberProps<'ctx, F>) -> View<G>
where
    F: Fn(CustomEvent) + 'ctx,
{
    let NumberProps {
        name,
        class,
        on_change,
        min,
        counter,
    } = props;
    NumberSpinner::define_once();
    // TODO(jwall): I'm pretty sure this triggers: https://github.com/sycamore-rs/sycamore/issues/602
    // Which means I probably have to wait till v0.9.0 drops or switch to leptos.
    let id = name.clone();
    let initial_count = *counter.get();
    view! {cx,
        number-spinner(id=id, class=(class), val=(initial_count), min=min, on:updated=move |evt: Event| {
            let event = evt.unchecked_into::<CustomEvent>();
            let val: f64 = event.detail().as_f64().unwrap();
            counter.set(val);
            on_change.as_ref().map(|f| f(event));
            debug!(counter=%(counter.get_untracked()), "set counter to new value");
        })
    }
}
