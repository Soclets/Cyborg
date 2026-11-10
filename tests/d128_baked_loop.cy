// trace: D128, D91
import std;

fn.pure.baked sum_to [$n: i32] -> i32:
    let mut $total: i32 = 0;
    for $i in range[from: 1, to: $n]:
        change $total = $total + $i;
    end for
    return $total;
end fn.pure.baked sum_to

const $ten: i32 = sum_to(4);

fn.main [none] -> void:
    std::io::fmt_print("{}{newln}", $ten);
    return void;
end fn.main
