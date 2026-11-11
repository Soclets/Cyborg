// trace: D128, D91
import std;
import os;

fn.pure.baked sum_to [$n: i32] returns i32:
    let mut $total: i32 = 0;
    for $i in range[from: 1, to: $n]:
        change $total = $total + $i;
    end for
    return $total;
end fn.pure.baked sum_to

const $ten: i32 = sum_to(4);

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $ten);
    return void;
end fn.main
