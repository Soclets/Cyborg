// A bracket that breaks its line has its closing one on its own line.
// trace: D219
@config(default);
import std;

fn.main [] -> void:
    std::io::fmt_print(
        "{} {}{newln}",
        1,
        2);
    return void;
end fn.main
