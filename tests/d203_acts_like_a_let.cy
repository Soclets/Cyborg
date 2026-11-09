// trace: D203
@config(default);
import std;

fn.main [] -> void:
    decl $x: i32;
    assign $x = 1;
    change $x = 2;
    return void;
end fn.main
