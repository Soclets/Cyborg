// A block's names end with it, so the next block can use the name again.
// trace: D57, D33
import std;
@config(default);

fn.main [] -> void:
    if 1 < 2:
        let $a: i32 = 1;
        std::println($a as str);
    end if
    if 1 < 2:
        let $a: i32 = 2;
        std::println($a as str);
    end if
    return void;
end fn.main
