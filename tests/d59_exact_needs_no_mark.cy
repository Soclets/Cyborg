// trace: D59, D54
import std;
import os;

const $pi: f64.stable = ~3.14159265358979323846;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: f64.stable = 0.5;
    let $b: f64.stable = ~0.1;
    let $c: f64.stable = ~-0.1;
    let $d: f32.stable = 0.25;
    let $e: f32.stable = ~0.1;
    let $f: f64.stable = -0.5;
    let $g: f64.stable = $a + 0.25;
    let $h: f32.stable = $d x 2.0;
    if $g: f64.stable[min: 0.75, max: 0.75]:
        if $b: f64.stable[min: ~0.1, max: ~0.1]:
            if $c: f64.stable[min: ~-0.1, max: ~-0.1]:
                if $h: f32.stable[min: 0.5, max: 0.5]:
                    std::io::fmt_print(Lend.mut $out, "floats work{newln}");
                end if
            end if
        end if
    end if
    return void;
end fn.main
