// Everything here fits in the digits the standard shows.
// trace: D161
import std;

fn.main [] -> void:
    let $a: f64 = 0.5;
    let $b: f64 = 1.0;
    let $c: f64 = -0.25;
    let $d: f32 = ~0.1;
    std::println.stored($a);
    std::println.stored($b);
    std::println.stored($c);
    std::println.stored($d);
    return void;
end fn.main
