// Each package a program uses has a `need` line.
// trace: D183, D228
@config(default);
import std;

fn.main [] -> void:
    std::io::fmt_print("hi{newln}");
    return void;
end fn.main
