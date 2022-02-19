# Copyright 2022 Jeremy Wall
# 
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
# 
#     http://www.apache.org/licenses/LICENSE-2.0
# 
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

kitchen: wasm kitchen/src/*.rs
	cd kitchen; cargo build

release: wasmrelease
	cd kitchen; cargo build --release

wasmrelease: web/index.html web/src/*.rs web/src/components/*.rs
	cd web; trunk build --release --public-url /ui/

wasm: web/index.html web/src/*.rs web/src/components/*.rs
	cd web; trunk build --public-url /ui/

clean:
	rm -rf web/dist/*