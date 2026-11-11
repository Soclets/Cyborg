// `fmt_print` is in the kit part, so a mark on it is still wrong through `use`.
// trace: D155, D22
import @std;
import os;

use @std::io::fmt_print as fmt_print;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    fmt_print(Lend.mut $out, "hi{newln}");
    return void;
end fn.main
