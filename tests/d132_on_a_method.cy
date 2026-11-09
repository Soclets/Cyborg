// trace: D132, D130
@config(default);
import std;

struct box:
    n: i32;
end struct box

fn box.%half [$b: box] -> i32:
    fails [odd];

    if $b.n mod.trunc 2 != 0:
        fail odd;
    end if
    return $b.n div.trunc 2;
end fn box.%half

fn.main [] -> void:
    let $b: box = box[n = 3];
    std::io::fmt_print("{}{newln}", (1 + $b.%half() ?? 0));
    return void;
end fn.main
