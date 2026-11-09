// trace: D203
import std;
@config(default);

fn.main [] -> void:
    decl $x: i32;
    assign $x = 1;
    assign $x = 2;
    return void;
end fn.main
