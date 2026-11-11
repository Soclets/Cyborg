// `$out.%%flush();` pushes output now and keeps the writer going, and `os::done` flushes for
// the last time. A print bigger than the buffer goes straight out.
// trace: D273
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.buffered = os::stdout.buffered(size: 4 bytes);
    std::io::fmt_print(Lend.mut $out, "ab");
    std::io::fmt_print(Lend.mut $out, "cd");
    $out.%%flush();
    std::io::fmt_print(Lend.mut $out, "e");
    std::io::fmt_print(Lend.mut $out, "fghij{newln}");
    std::io::fmt_print(Lend.mut $out, "k{newln}");
    os::done(Give $out);
    return void;
end fn.main
