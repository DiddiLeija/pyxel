#
# [How to build]
#
# Required tools:
#	- make
#	- cmake
#	- rustup
#	- python 3.7+
#
#	[Windows]
#	- Git Bash
#
#	[Linux]
#	- SDL2 (e.g. libsdl2-dev for Ubuntu)
#
#	[WASM]
#	- Emscripten 3.1.14
#
# Advance preparation:
#	rustup install nightly
#	git clone --depth 1 https://github.com/kitao/pyxel.git
#	cd pyxel
#	(Create and activate a venv if you prefer)
#	pip3 install -r requirements.txt
#
# Build the package in the dist directory
#	make clean build
#
# Build the package for the specified target:
#	make clean build TARGET=target_triple
#
# Build, install, and test the package in the current Python
#	make clean test
#
# Build the package for WASM in the dist directory
#	make clean-wasm build-wasm
#
# Test the package for WASM in localhost:8000/wasm/
#	make clean-wasm test-wasm
#

ROOT_DIR = .
DIST_DIR = $(ROOT_DIR)/dist
PYXEL_DIR = $(ROOT_DIR)/python/pyxel
CRATES_DIR = $(ROOT_DIR)/crates
SCRIPTS_DIR = $(ROOT_DIR)/scripts
EXAMPLES_DIR = $(PYXEL_DIR)/examples
CRATES = $(wildcard $(CRATES_DIR)/pyxel-*)
EXAMPLES = $(wildcard $(EXAMPLES_DIR)/*.py)
SRC_SDL2 = $(CRATES_DIR)/pyxel-wrapper/target/$(TARGET)/release/SDL2.dll
DST_SDL2 = $(PYXEL_DIR)/SDL2.dll

ifeq ($(TARGET),)
ADD_TARGET =
BUILD_OPTS = --release
else
ADD_TARGET = rustup target add $(TARGET)
BUILD_OPTS = --release --target $(TARGET)
endif

WASM_ENVVARS = RUSTUP_TOOLCHAIN=nightly
WASM_TARGET = wasm32-unknown-emscripten

.PHONY: all clean distclean lint format build install test clean-wasm build-wasm install-wasm test-wasm

all: build

clean:
	@for crate in $(CRATES); do \
		cd $$crate; \
		cargo clean $(BUILD_OPTS); \
		cd -; \
	done

distclean:
	@for crate in $(CRATES); do \
		rm -rf $$crate/target; \
	done
	@rm -rf $(DIST_DIR)

lint:
	@for crate in $(CRATES); do \
		cd $$crate; \
		cargo clippy -q -- --no-deps; \
		cd -; \
	done
	@flake8 $(SCRIPTS_DIR) $(PYXEL_DIR)

format:
	@for crate in $(CRATES); do \
		cd $$crate; \
		cargo +nightly fmt -- --emit=files; \
		cd -; \
	done
	@isort $(ROOT_DIR)
	@black $(ROOT_DIR)
	@$(SCRIPTS_DIR)/update_readme

build: format
	@$(ADD_TARGET)
	@rm -f $(DST_SDL2)
	@maturin build -o $(DIST_DIR) $(BUILD_OPTS)
	@if [ -e $(SRC_SDL2) ]; then \
		cp $(SRC_SDL2) $(DST_SDL2); \
		maturin build -o $(DIST_DIR) $(BUILD_OPTS); \
		rm $(DST_SDL2); \
	fi

test: build
	@pip3 install --force-reinstall `ls -rt $(DIST_DIR)/*.whl | tail -n 1`
	@cd $(CRATES_DIR)/pyxel-engine; cargo test $(BUILD_OPTS)
	@python3 -m unittest discover $(CRATES_DIR)/pyxel-wrapper/tests

	@for example in $(EXAMPLES); do \
		pyxel run $$example; \
	done
	@pyxel play $(EXAMPLES_DIR)/30SecondsOfDaylight.pyxapp
	@pyxel play $(EXAMPLES_DIR)/megaball.pyxapp
	@pyxel edit $(EXAMPLES_DIR)/assets/sample.pyxres

	@rm -rf testapp testapp.pyxapp
	@mkdir -p testapp/assets
	@cp $(EXAMPLES_DIR)/10_platformer.py testapp
	@cp $(EXAMPLES_DIR)/assets/platformer.pyxres testapp/assets
	@pyxel package testapp 10_platformer.py
	@pyxel play testapp.pyxapp
	@rm -rf testapp testapp.pyxapp

clean-wasm:
	@$(WASM_ENVVARS) make clean TARGET=$(WASM_TARGET)

build-wasm:
	@$(WASM_ENVVARS) make build TARGET=$(WASM_TARGET)

test-wasm: build-wasm
	@cp -f $(DIST_DIR)/*-emscripten_*.whl $(ROOT_DIR)/wasm
	@$(SCRIPTS_DIR)/start_server
