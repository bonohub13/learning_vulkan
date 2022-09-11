SHELL := bash
CC := $(shell which cargo)
PWD := $(shell pwd)

all: clean-shader build-shader clean build run

# Shader code
clean-shader:
	echo "Performing clean up of existing shaders..."
	./bin/compile_shaders.sh "clean" "${PWD}/shaders"

build-shader: clean-shader
	echo "Compiling shaders..."
	./bin/compile_shaders.sh "build" "${PWD}/shaders"

# Rust code
clean:
	$(CC) clean

fmt:
	$(CC) fmt

build: fmt clean
	mkdir -p bin
	$(CC) build
	cp ./target/debug/learning_vulkan bin

cross-compile-win64: clean
	mkdir -p bin/x86_64-pc-windows-gnu
	$(CC) build --target x86_64-pc-windows-gnu
	cp ./target/x86_64-pc-windows-gnu/debug/learning_vulkan.exe bin/x86_64-pc-windows-gnu

run:
	ENABLE_VKBASALT=0 MANGOHUD=0 ./bin/learning_vulkan

run-win64:
	.\bin\x86_64-pc-windows-gnu\learning_vulkan.exe

rebuild-win64-image:
	docker build . -t ofv/windows -f docker/Dockerfile.windows --no-cache

rebuild-linux-image:
	cp Cargo.toml docker
	docker build . -t ofv/linux -f docker/Dockerfile.linux --no-cache
	rm docker/Cargo.toml

rebuild-all-images: rebuild-linux-image rebuild-win64-image

docker-build: clean
	mkdir -p bin
	docker run --rm -it -v $(shell pwd):/app ofv/linux
	cp ./target/debug/learning_vulkan bin

docker-cross-compile-win64: clean
	mkdir -p bin/x86_64-pc-windows-gnu
	docker run --rm -it -v $(shell pwd):/app ofv/windows
	cp ./target/x86_64-pc-windows-gnu/debug/learning_vulkan.exe bin/x86_64-pc-windows-gnu
