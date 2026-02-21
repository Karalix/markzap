APP_NAME = MarkZap
BINARY_NAME = markzap

.PHONY: build bundle bundle-debug dmg icon install uninstall clean

build:
	cargo build --release

bundle: build
	./scripts/bundle.sh

bundle-debug:
	./scripts/bundle.sh --debug

dmg: bundle
	./scripts/create-dmg.sh

icon:
	@if [ -z "$(SOURCE)" ]; then \
		echo "Usage: make icon SOURCE=path/to/image-1024x1024.png"; \
		exit 1; \
	fi
	./scripts/generate-icon.sh "$(SOURCE)"

install: bundle
	@echo "Installing $(APP_NAME).app to /Applications..."
	cp -r target/release/$(APP_NAME).app /Applications/
	@echo "Registering with Launch Services..."
	/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -f /Applications/$(APP_NAME).app
	@echo ""
	@echo "$(APP_NAME) is now installed."
	@echo "To set as default for .md files: right-click a .md file > Get Info > Open With > $(APP_NAME) > Change All"

uninstall:
	rm -rf /Applications/$(APP_NAME).app
	@echo "$(APP_NAME) has been uninstalled."

clean:
	cargo clean
