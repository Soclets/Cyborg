// trace: D132
import std;
@config(default);

fn read [] -> i32:
    fails [none];

    fail none;
end fn read

fn.main [] -> void:
    let $x: i32 = 5;
    std::println((read() ?? -1) as str);
    std::println((read() ?? $x) as str);
    std::println((read() ?? ($x + 1)) as str);
    return void;
end fn.main
