// trace: D128
import std;
@config(default);

fn.main [] -> void:
    let $start: i64 = 2;
    let $stop: i64 = 4;
    for $i in range[from: $start, to: $stop]:
        let $copy: i64 = $i;
        std::io::fmt_print("i64{newln}");
    end for
    let $bounded: i32[min: 0, max: 9] = 1;
    for $i in range[from: $bounded, to: 2]:
        std::io::fmt_print("{}{newln}", ($i + 1));
    end for
    return void;
end fn.main
