// trace: D203
import std;

fn pick [$hot: bool] -> void:
    decl $x: i32;
    if $hot:
        assign $x = 1;
    end if
    assign $x = 2;
    return void;
end fn pick

fn.main [none] -> void:
    return void;
end fn.main
