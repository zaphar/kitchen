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

static-prep: web/index.html web/static/*.css
	mkdir -p web/dist
	cp -r web/index.html web/dist/
	cp -r web/static web/dist/

wasmrelease: wasmrelease-dist static-prep

wasmrelease-dist: web/src/*.rs web/src/components/*.rs
	cd web; wasm-pack build --release --target web --out-dir dist/

wasm: wasm-dist static-prep

wasm-dist: web/src/*.rs web/src/components/*.rs
	cd web; wasm-pack build --target web --out-dir dist/

clean:
	rm -rf web/dist/*
	cargo clean