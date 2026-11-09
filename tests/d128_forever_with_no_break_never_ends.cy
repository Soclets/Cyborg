// Nothing runs past a `forever` with no `break`, so this needs no `return` after it.
// trace: D128, D11
@config(default);
import std;

fn first_over [$limit: i32] -> i32:
    let mut $n: i32 = 0;
    forever:
        change $n = $n + 7;
        if $n > $limit:
            return $n;
        end if
    end forever
end fn first_over

fn.main [] -> void:
    std::io::fmt_print("{}{newln}", first_over(10));
    return void;
end fn.main
