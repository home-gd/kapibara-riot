# Package release builds with assets/.
#
#   make            both zips
#   make run        play the game (cargo run). Same command on Windows and Linux.
#   make windows    dist/kapibara_riot-windows.zip
#   make linux      dist/kapibara_riot-linux.zip
#
# On Windows, GNU make is a separate install:
#   winget install ezwinports.make
#
# Windows zip:
#   kapibara_riot-windows/kapibara_riot.exe
#   kapibara_riot-windows/assets/
#
# Linux zip:
#   kapibara_riot-linux/kapibara_riot
#   kapibara_riot-linux/assets/
#
# Windows requires cargo-xwin, clang, and zip.
# The Rust target is installed with:
#   rustup target add x86_64-pc-windows-msvc

WIN_TARGET := x86_64-pc-windows-msvc
NAME       := kapibara_riot

WIN_EXE   := target/$(WIN_TARGET)/release/$(NAME).exe
WIN_STAGE := dist/$(NAME)-windows
WIN_ZIP   := dist/$(NAME)-windows.zip

LINUX_BIN   := target/release/$(NAME)
LINUX_STAGE := dist/$(NAME)-linux
LINUX_ZIP   := dist/$(NAME)-linux.zip

SRC    := $(wildcard src/*.rs)
ASSETS := $(wildcard assets/level1/* assets/player/*)

ifeq ($(OS),Windows_NT)
export PATH := $(USERPROFILE)/.cargo/bin;$(PATH)
else
export PATH := $(HOME)/.cargo/bin:$(PATH)
endif

.PHONY: all windows linux clean run
.DELETE_ON_ERROR:

all: windows linux

run:
	cargo run

windows: $(WIN_ZIP)
linux: $(LINUX_ZIP)

$(WIN_ZIP): $(WIN_EXE) $(ASSETS)
	rm -rf $(WIN_STAGE)
	mkdir -p $(WIN_STAGE)
	cp $(WIN_EXE) $(WIN_STAGE)/$(NAME).exe
	cp -a assets $(WIN_STAGE)/assets
	rm -f $(WIN_ZIP)
	cd dist && zip -r $(NAME)-windows.zip $(NAME)-windows
	@echo "wrote $(WIN_ZIP)"

$(WIN_EXE): Cargo.toml Cargo.lock $(SRC)
	cargo xwin build --release --target $(WIN_TARGET)

$(LINUX_ZIP): $(LINUX_BIN) $(ASSETS)
	rm -rf $(LINUX_STAGE)
	mkdir -p $(LINUX_STAGE)
	cp $(LINUX_BIN) $(LINUX_STAGE)/$(NAME)
	cp -a assets $(LINUX_STAGE)/assets
	rm -f $(LINUX_ZIP)
	cd dist && zip -r $(NAME)-linux.zip $(NAME)-linux
	@echo "wrote $(LINUX_ZIP)"

$(LINUX_BIN): Cargo.toml Cargo.lock $(SRC)
	cargo build --release

clean:
	rm -rf dist
