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
use static_dir::static_dir;
use warp::{hyper::Uri, Filter};

pub async fn ui_main() {
    let root = warp::path::end().map(|| warp::redirect::found(Uri::from_static("/ui")));
    let ui = warp::path("ui").and(static_dir!("webdist/"));
    // api route goes here eventually.
    let api = warp::path!("api" / " v1").map(|| format!("API stuff goes here!"));

    let routes = root.or(ui).or(api).boxed();

    // TODO(jwall): Take listen address as an argument to this function instead.
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
