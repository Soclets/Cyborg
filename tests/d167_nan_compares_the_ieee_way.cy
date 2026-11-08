// Every comparison with NaN is false except `!=`, and NaN isn't even equal to itself.
// trace: D167, D159
import std;

fn.main [] -> void:
    let mut $x: f64.ieee = 2.0;
    for $i in range[from: 1, to: 10]:
        change $x = $x x $x;
    end for
    let $nan: f64.ieee = $x - $x;
    if $nan == $nan:
        std::println("==");
    end if
    if $nan != $nan:
        std::println("!=");
    end if
    if $nan < 1.0:
        std::println("<");
    end if
    if $nan > 1.0:
        std::println(">");
    end if
    if $nan at_least 1.0:
        std::println("at_least");
    end if
    if $nan at_most 1.0:
        std::println("at_most");
    end if
    std::println("done");
    return void;
end fn.main
