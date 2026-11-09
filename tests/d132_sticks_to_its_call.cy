// trace: D132
import std;
@config(default);

fn read [] -> i32:
    fails [none];

    fail none;
end fn read

fn.main [] -> void:
    let $a: i32 = 10;
    std::println(($a + read() ?? 1) as str);
    std::println(($a x read() ?? 2) as str);
    std::println((read() ?? 1 + 2) as str);
    std::println((read() ?? 3 x 2) as str);
    if read() ?? 1 < 2:
        std::println("compared after");
    end if
    return void;
end fn.main
