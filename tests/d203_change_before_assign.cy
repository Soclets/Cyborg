// trace: D203
@config(default);
import std;

fn.main [] -> void:
    decl mut $x: i32;
    change $x = 2;
    return void;
end fn.main
