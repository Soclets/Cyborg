// trace: D203
import std;

fn pick [$hot: bool] returns void:
    decl $x: i32;
    if $hot:
        assign $x set_to 1;
    end if
    assign $x set_to 2;
    return void;
end fn pick

fn.main [none] returns void:
    return void;
end fn.main
