// A format hole's shapes are independent settings, so they still go in any order.
// trace: D259, D194
import std;

fn.main [none] -> void:
    let $n: i32[min: 0, max: 99] = 7;
    std::io::fmt_print("[{$n, width: 4 cols, align: right}] [{$n, align: right, width: 4 cols}]{newln}");
    return void;
end fn.main
