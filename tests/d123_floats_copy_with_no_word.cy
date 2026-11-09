// trace: D123
@config(default);
import std;

fn twice [$x: f64] -> f64:
    return $x + $x;
end fn twice

fn.main [] -> void:
    let $a: f64 = 0.5;
    let $b: f64 = $a;
    let $c: f64 = twice($a);
    if $c: f64[min: 1.0, max: 1.0]:
        if $b: f64[min: 0.5, max: 0.5]:
            std::io::fmt_print("copied{newln}");
        end if
    end if
    return void;
end fn.main
