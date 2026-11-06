// trace: D59, D54
import std;

const $pi: f64 = ~3.14159265358979323846;

fn.main [] -> void:
    let $a: f64 = 0.5;
    let $b: f64 = ~0.1;
    let $c: f64 = ~-0.1;
    let $d: f32 = 0.25;
    let $e: f32 = ~0.1;
    let $f: f64 = -0.5;
    let $g: f64 = $a + 0.25;
    if $g == 0.75 and $b != $c and $d x 2.0 == 0.5:
        std::println("floats work");
    end if
    return void;
end fn.main
