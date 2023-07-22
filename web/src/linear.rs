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
use std::convert::Into;
use std::ops::Drop;
use std::rc::Rc;

use sycamore::prelude::*;

pub struct LinearSignal<'ctx, Payload> {
    pub signal: &'ctx Signal<Payload>,
    nv: Option<Payload>,
}

impl<'ctx, Payload> Into<LinearSignal<'ctx, Payload>> for &'ctx Signal<Payload> {
    fn into(self) -> LinearSignal<'ctx, Payload> {
        LinearSignal { signal: self, nv: None }
    }
}

impl<'ctx, Payload> LinearSignal<'ctx, Payload> {
    pub fn update(mut self, payload: Payload) -> Self {
        self.nv = Some(payload);
        return self;
    }

    pub fn get(&'ctx self) -> Rc<Payload> {
        self.signal.get()
    }
}

impl<'ctx, Payload> Drop for LinearSignal<'ctx, Payload> {
    fn drop(&mut self) {
        if self.nv.is_some() {
            let mut val: Option<Payload> = None;
            std::mem::swap(&mut val, &mut self.nv);
            let payload = val.unwrap();
            self.signal.set(payload);
        }
    }
}
