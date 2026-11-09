// trace: D160
import std;
@config(default);

fn.main [] -> void:
    let $a: f64 = 0.5;
    if $a at_least 0.25:
        std::println("at least");
    end if
    if $a at_most 0.25:
        std::println("at most");
    end if
    if $a < 0.75:
        std::println("below");
    end if
    if $a > 0.75:
        std::println("above");
    end if
    return void;
end fn.main
