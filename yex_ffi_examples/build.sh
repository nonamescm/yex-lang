cp ../yex_ffi_lib yex -rv
clang *.c yex/yex.c -shared -o example.so || gcc *.c yex/yex.c -shared -o example.so 

cargo run --release --features=repl -q examples.yex || yex examples.yex
rm -rfv yex
rm -rf *.so
