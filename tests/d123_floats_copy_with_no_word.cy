// trace: D123
@config(default);
import std;

fn twice [$x: f64.stable] -> f64.stable:
    return $x + $x;
end fn twice

fn.main [] -> void:
    let $a: f64.stable = 0.5;
    let $b: f64.stable = $a;
    let $c: f64.stable = twice($a);
    if $c: f64.stable[min: 1.0, max: 1.0]:
        if $b: f64.stable[min: 0.5, max: 0.5]:
            std::io::fmt_print("copied{newln}");
        end if
    end if
    return void;
end fn.main
