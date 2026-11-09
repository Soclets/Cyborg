// trace: D203
import std;
@config(default);

fn.main [] -> void:
    decl mut $x: i32;
    change $x = 2;
    return void;
end fn.main
