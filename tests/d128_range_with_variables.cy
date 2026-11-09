// trace: D128
import std;
@config(default);

fn.main [] -> void:
    let $start: i64 = 2;
    let $stop: i64 = 4;
    for $i in range[from: $start, to: $stop]:
        let $copy: i64 = $i;
        std::println("i64");
    end for
    let $bounded: i32[min: 0, max: 9] = 1;
    for $i in range[from: $bounded, to: 2]:
        std::println(($i + 1) as str);
    end for
    return void;
end fn.main
