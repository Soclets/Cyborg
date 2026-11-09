// The path that doesn't assign returns, so every path that reaches the read has assigned.
// trace: D203
import std;
@config(default);

fn pick [$hot: bool] -> void:
    decl $x: i32;
    if $hot:
        assign $x = 1;
    else:
        return void;
    end if
    std::io::fmt_print("{}{newln}", $x);
    return void;
end fn pick

fn.main [] -> void:
    pick(true);
    return void;
end fn.main
