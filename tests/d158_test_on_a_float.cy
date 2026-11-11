// trace: D158, D20
import std;
import os;

fn pick [$x: f64.stable] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    if $x: f64.stable[min: 100.0, max: 200.0]:
        std::io::fmt_print(Lend.mut $out, "inside{newln}");
    else:
        std::io::fmt_print(Lend.mut $out, "outside{newln}");
    end if
    return void;
end fn pick

fn.main [none] returns void:
    pick(150.5);
    pick(99.5);
    pick(200.0);
    return void;
end fn.main
