.PHONY : build clean

TRANSPILER_DIR = "./src/transpiler"
COMPILER_DIR = "./src/compiler"

all: build

install-cargo:
	curl https://sh.rustup.rs -sSf | sh

build:
	make -C ${TRANSPILER_DIR}
	make -C ${COMPILER_DIR}

	cp ${TRANSPILER_DIR}/Transpiler ./lib/transpiler
	cp ${COMPILER_DIR}/bin/insc_jvm .
	cp ${COMPILER_DIR}/bin/insc_llvm .

clean:
	make -C ${TRANSPILER_DIR} clean
	make -C ${COMPILER_DIR} clean
	rm -f insc_jvm insc_llvm ./lib/transpiler