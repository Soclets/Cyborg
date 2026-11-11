// trace: D132, D130
import std;
import os;

struct box:
    n: i32;
end struct box

fn box.%half [$b: box] returns i32 fails [odd]:
    if $b.n mod.trunc 2 != 0:
        fail odd;
    end if
    return $b.n div.trunc 2;
end fn box.%half

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $b: box set_to box[n set_to 3];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", (1 + $b.%half() ?? 0));
    return void;
end fn.main
