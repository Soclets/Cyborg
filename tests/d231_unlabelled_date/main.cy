// A version is a labelled date, so `2026-11-01` gets its labels.
// trace: D231
@config(default);
import std;

fn.main [] -> void:
    std::io::fmt_print("hi{newln}");
    return void;
end fn.main
