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
	$(CC) build

cross-compile-win64: clean
	$(CC) build --target x86_64-pc-windows-gnu

run:
	$(CC) run

docker-build: clean
	docker build . -t ofv/linux -f docker/Dockerfile.linux
	docker run --rm -it -v $(shell pwd):/app ofv/linux

docker-cross-compile-win64: clean
	docker build . -t ofv/windows -f docker/Dockerfile.windows
	docker run --rm -it -v $(shell pwd):/app ofv/windows
