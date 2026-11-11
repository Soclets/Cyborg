# Licences of what's inside `okay`

`okay` has LLVM and lld inside it, through Lyftar, so it ships their licence texts, `LICENSE-LLVM.txt` and
`LICENSE-lld.txt`. Programs it links get compiler-rt's builtins (`LICENSE-compiler-rt.txt`), and Linux programs are
linked statically with musl, whose MIT notice (`LICENSE-musl.txt`) `okay comp` writes next to every Linux program it
makes. These are copied from Lyftar's downloaded LLVM 23.1.2 and system files, as its README section "A
self-contained compiler" lists them.
