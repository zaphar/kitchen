use chrono::NaiveDate;
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

use crate::app_state::{Message, StateHandler};
use tracing::instrument;

#[derive(Props)]
pub struct PlanListProps<'ctx> {
    sh: StateHandler<'ctx>,
    list: &'ctx ReadSignal<Vec<NaiveDate>>,
}

// TODO(jwall): We also need a "new plan button"
#[instrument(skip_all, fields(dates=?props.list))]
#[component]
pub fn PlanList<'ctx, G: Html>(cx: Scope<'ctx>, props: PlanListProps<'ctx>) -> View<G> {
    let PlanListProps { sh, list } = props;
    view! {cx,
        Indexed(
            iterable=list,
            view=move |cx, date| {
                let date_display = format!("{}", date);
                view!{cx,
                    div(on:click=move |_| {
                        sh.dispatch(cx, Message::SelectPlanDate(date))
                    }) { (date_display) }
                }
            },
        )
    }
}
