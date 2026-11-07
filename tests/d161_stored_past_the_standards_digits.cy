// D161 says this prints what's in memory, which is 55 digits, and the standard's example stops at 34.
// trace: D161
import std;

fn.main [] -> void:
    let $x: f64 = ~0.1;
    std::println($x as str);
    std::println.stored($x);
    return void;
end fn.main
