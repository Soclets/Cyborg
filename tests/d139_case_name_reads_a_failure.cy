// trace: D139, D47
import std;

fn half [$n: i32] -> i32 fails [negative[val: i32], odd]:
    if $n < 0:
        fail negative[val = $n];
    end if
    if $n mod.trunc 2 != 0:
        fail odd;
    end if
    return $n div.trunc 2;
end fn half

fn show [$n: i32] -> void:
    let $h: i32 = half($n) otherwise:
        case negative $f:
            std::io::fmt_print("negative {}{newln}", $f.val);
            instead 0;
        case odd:
            instead -1;
    end otherwise
    std::io::fmt_print("got {}{newln}", $h);
    return void;
end fn show

fn.main [none] -> void:
    show(4);
    show(-3);
    show(3);
    return void;
end fn.main
