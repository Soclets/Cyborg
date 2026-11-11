// trace: D123
import std;
import os;

fn twice [$x: f64.stable] returns f64.stable:
    return $x + $x;
end fn twice

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: f64.stable = 0.5;
    let $b: f64.stable = $a;
    let $c: f64.stable = twice($a);
    if $c: f64.stable[min: 1.0, max: 1.0]:
        if $b: f64.stable[min: 0.5, max: 0.5]:
            std::io::fmt_print(Lend.mut $out, "copied{newln}");
        end if
    end if
    return void;
end fn.main
