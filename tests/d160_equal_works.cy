// trace: D160
import std;
@config(default);

fn.main [] -> void:
    let $a: f64 = 0.5;
    if $a == 0.5:
        std::println("equal");
    end if
    if $a != 0.5:
        std::println("different");
    end if
    return void;
end fn.main
