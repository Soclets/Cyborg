// `fmt_print` is in the kit part, so a mark on it is still wrong through `use`.
// trace: D155, D22
import @std;

use @std::io::fmt_print as fmt_print;

fn.main [none] -> void:
    fmt_print("hi{newln}");
    return void;
end fn.main
