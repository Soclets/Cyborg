// trace: D148, D20
import std;
@config(default);

fn.main [] -> void:
    let $start: i32[min: 1, max: 3] = 2;
    for $i in range[from: $start, to: 3]:
        std::println($i as str);
    end for
    return void;
end fn.main
