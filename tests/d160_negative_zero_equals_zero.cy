// trace: D160
import std;

fn.main [] -> void:
    let $n: f64 = -0.0;
    let $p: f64 = 0.0;
    if $n == $p:
        std::println("equal");
    end if
    if $n != $p:
        std::println("different");
    end if
    if $n < $p:
        std::println("below");
    end if
    if $n at_least $p:
        std::println("at least");
    end if
    return void;
end fn.main
