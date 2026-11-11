// A format hole's shapes are independent settings, so they still go in any order.
// trace: D259, D194
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $n: i32[min: 0, max: 99] set_to 7;
    std::io::fmt_print(Lend.mut $out, "[{$n, width: 4 cols, align: right}] [{$n, align: right, width: 4 cols}]{newln}");
    return void;
end fn.main
