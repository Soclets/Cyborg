// trace: D203
@config(default);
import std;

fn.main [] -> void:
    let mut $x: i32 = 1;
    assign $x = 2;
    return void;
end fn.main
