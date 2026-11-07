// trace: D160, D159
import std;

fn.main [] -> void:
    let $a: f64.ieee = 1.5;
    let $b: f64.ieee = 2.5;
    if $a at_most $b:
        std::println("at most");
    end if
    return void;
end fn.main
