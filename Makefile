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
mkfile_path := $(abspath $(lastword $(MAKEFILE_LIST)))
mkfile_dir := $(dir $(mkfile_path))
sqlite_url := sqlite://$(mkfile_dir)/.session_store/store.db
export out := dist
export project := kitchen

kitchen: wasm kitchen/src/*.rs
	cd kitchen; cargo build

release: wasmrelease
	cd kitchen; cargo build --release

static-prep: web/index.html web/favicon.ico web/static/*.css
	mkdir -p web/dist
	cp -r web/index.html web/dist/
	cp -r web/favicon.ico web/dist/
	cp -r web/static web/dist/

wasmrelease: wasm-opt static-prep

wasm-opt: wasmrelease-dist
	cd web; sh ../scripts/wasm-opt.sh release

wasmrelease-dist: web/src/*.rs web/src/components/*.rs
	cd web; sh ../scripts/wasm-build.sh release

wasm: wasm-dist static-prep

wasm-dist: web/src/*.rs web/src/components/*.rs
	cd web; sh ../scripts/wasm-build.sh debug
	cd web; sh ../scripts/wasm-sourcemap.sh

clean:
	rm -rf web/dist/*
	cargo clean

sqlx-migrate:
	cd kitchen; cargo sqlx migrate run --database-url $(sqlite_url)

sqlx-add-%:
	cd kitchen; cargo sqlx migrate add -r $*

sqlx-revert:
	cd kitchen; cargo sqlx migrate revert --database-url $(sqlite_url)

sqlx-prepare: wasm
	cd kitchen; cargo sqlx prepare --database-url $(sqlite_url)
