// Nothing runs past a `forever` with no `break`, so this needs no `return` after it.
// trace: D128, D11
import std;
@config(default);

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
    std::println(first_over(10) as str);
    return void;
end fn.main
