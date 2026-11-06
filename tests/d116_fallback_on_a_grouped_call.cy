// trace: D116, D44
import std;

fn read [] -> i32:
    fails [none];

    fail none;
end fn read

fn.main [] -> void:
    let $a: i32 = 1;
    let $b: i32 = $a + (read() ?? 5);
    std::println($b as str);
    return void;
end fn.main
