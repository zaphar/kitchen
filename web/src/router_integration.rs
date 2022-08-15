// Copyright 2022 zaphar
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
use std::rc::Rc;

use sycamore::prelude::*;
use tracing::{debug, error, instrument};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::Event;
use web_sys::{Element, HtmlAnchorElement};

use crate::app_state::AppRoutes;

#[derive(Clone, Debug)]
pub struct BrowserIntegration(Signal<(String, String, String)>);

impl BrowserIntegration {
    pub fn new() -> Self {
        let location = web_sys::window().unwrap_throw().location();
        Self(Signal::new((
            location.origin().unwrap_or(String::new()),
            location.pathname().unwrap_or(String::new()),
            location.hash().unwrap_or(String::new()),
        )))
    }

    #[instrument(skip(self, f))]
    pub fn register_post_state_handler(&self, f: Box<dyn FnMut()>) {
        let closure = Closure::wrap(f);
        web_sys::window()
            .unwrap_throw()
            .add_event_listener_with_callback("popstate", closure.as_ref().unchecked_ref())
            .unwrap_throw();
        closure.forget();
    }

    #[instrument(skip(self))]
    pub fn click_handler(&self) -> Box<dyn Fn(web_sys::Event)> {
        let route_signal = self.0.clone();
        Box::new(move |ev| {
            if let Some(tgt) = ev
                .target()
                .unwrap_throw()
                .unchecked_into::<Element>()
                .closest("a[href]")
                .unwrap_throw()
                .map(|e| e.unchecked_into::<HtmlAnchorElement>())
            {
                debug!("handling navigation event.");
                let location = web_sys::window().unwrap_throw().location();

                if tgt.rel() == "external" {
                    debug!("External Link so ignoring.");
                    return;
                }

                let origin = tgt.origin();
                let tgt_pathname = tgt.pathname();
                let hash = tgt.hash();
                match (location.origin().as_ref() == Ok(&origin), location.pathname().as_ref() == Ok(&tgt_pathname), location.hash().as_ref() == Ok(&hash)) {
                    (true, true, true) // Same location
                    | (false, _, _) /* different origin */ => {
                        // Do nothing this is the same location as we are already at.
                    }
                    (true, _, false) // different hash
                    | (true, false, _) /* different path */ => {
                        debug!("different path or hash");
                        ev.prevent_default();
                        // Signal the pathname change
                        let path = format!("{}{}{}", &origin, &tgt_pathname, &hash);
                        debug!("new route: ({}, {}, {})", origin, tgt_pathname, hash);
                        debug!("new path: ({})", &path);
                        route_signal.set((origin, tgt_pathname, hash));
                        // Update History API.
                        let window = web_sys::window().unwrap_throw();
                        let history = window.history().unwrap_throw();
                        history
                            .push_state_with_url(&JsValue::UNDEFINED, "", Some(&path))
                            .unwrap_throw();
                        window.scroll_to_with_x_and_y(0.0, 0.0);
                    }
                }
            }
        })
    }
}

#[derive(Debug)]
pub struct RouterProps<R, F, G>
where
    G: GenericNode,
    R: DeriveRoute + NotFound + Clone + Default + Debug + 'static,
    F: Fn(ReadSignal<R>) -> View<G> + 'static,
{
    pub route: R,
    pub route_select: F,
    pub browser_integration: BrowserIntegration,
}

#[instrument(fields(?props.route,
        origin=props.browser_integration.0.get().0,
        pathn=props.browser_integration.0.get().1,
        hash=props.browser_integration.0.get().2),
    skip(props))]
#[component(Router<G>)]
pub fn router<R, F>(props: RouterProps<R, F, G>) -> View<G>
where
    R: DeriveRoute + NotFound + Clone + Default + Debug + 'static,
    F: Fn(ReadSignal<R>) -> View<G> + 'static,
{
    debug!("Setting up router");
    let integration = Rc::new(props.browser_integration);
    let route_select = Rc::new(props.route_select);

    let view_signal = Signal::new(View::empty());
    create_effect(
        cloned!((view_signal, integration, route_select) => move || {
            let path_signal = integration.0.clone();
            debug!(origin=%path_signal.get().0, path=%path_signal.get().1, hash=%path_signal.get().2, "new path");
            let path = path_signal.clone();
            let route = R::from(path.get().as_ref());
            debug!(?route, "new route");
            // TODO(jwall): this is an unnecessary use of signal.
            let view = route_select.as_ref()(Signal::new(route).handle());
            register_click_handler(&view, integration.clone());
            view_signal.set(view);
        }),
    );

    let path_signal = integration.0.clone();
    integration.register_post_state_handler(Box::new(cloned!((path_signal) => move || {
        let location = web_sys::window().unwrap_throw().location();
        path_signal.set((location.origin().unwrap_throw(), location.pathname().unwrap_throw(), location.hash().unwrap_throw()));
    })));

    // NOTE(jwall): This needs to be a dynamic node so Sycamore knows to rerender it
    // based on the results of the effect above.
    view! {
        (view_signal.get().as_ref())
    }
}

#[instrument(skip_all)]
fn register_click_handler<G>(view: &View<G>, integration: Rc<BrowserIntegration>)
where
    G: GenericNode<EventType = Event>,
{
    debug!("Registring click handler on node(s)");
    if let Some(node) = view.as_node() {
        node.event("click", integration.click_handler());
    } else if let Some(frag) = view.as_fragment() {
        debug!(fragment=?frag);
        for n in frag {
            register_click_handler(n, integration.clone());
        }
    } else if let Some(dyn_node) = view.as_dyn() {
        debug!(dynamic_node=?dyn_node);
    } else {
        debug!(node=?view, "Unknown node");
    }
}

pub trait NotFound {
    fn not_found() -> Self;
}

impl NotFound for AppRoutes {
    fn not_found() -> Self {
        AppRoutes::NotFound
    }
}

pub trait DeriveRoute {
    fn from(input: &(String, String, String)) -> Self;
}

impl DeriveRoute for AppRoutes {
    #[instrument]
    fn from(input: &(String, String, String)) -> AppRoutes {
        debug!(origin=%input.0, path=%input.1, hash=%input.2, "routing");
        match input.2.as_str() {
            "" => AppRoutes::default(),
            "#plan" => AppRoutes::Plan,
            "#cook" => AppRoutes::Cook,
            "#inventory" => AppRoutes::Inventory,
            h => {
                // TODO(jwall): Parse the recipe hash
                let parts: Vec<&str> = h.splitn(2, "/").collect();
                if let Some(&"#recipe") = parts.get(0) {
                    if let Some(&idx) = parts.get(1) {
                        return AppRoutes::Recipe(idx.to_owned());
                    }
                }
                error!(origin=%input.0, path=%input.1, hash=%input.2, "Path not found");
                AppRoutes::NotFound
            }
        }
    }
}
