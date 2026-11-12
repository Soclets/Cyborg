// A Dialect is another compiler, which CyJet loads from its registry, so the oracle doesn't build it.
// trace: D309
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.buffered = os::stdout.buffered(size: 1024 bytes);
    std::io::fmt_print(Lend.mut $out, "hi{newln}");
    os::done(Give $out);
    return void;
end fn.main
