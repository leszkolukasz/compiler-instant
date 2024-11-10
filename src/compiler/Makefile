.PHONY : build clean

build:
	mkdir -p ./bin

	cargo build --features jvm --release
	cp ./target/release/compiler ./bin/insc_jvm

	cargo build --features llvm --release
	cp ./target/release/compiler ./bin/insc_llvm



clean:
	rm -rf bin target