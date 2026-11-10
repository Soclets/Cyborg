// trace: D158, D20
@config(default);
import std;

fn pick [$x: f64.stable] -> void:
    if $x: f64.stable[min: 100.0, max: 200.0]:
        std::io::fmt_print("inside{newln}");
    else:
        std::io::fmt_print("outside{newln}");
    end if
    return void;
end fn pick

fn.main [] -> void:
    pick(150.5);
    pick(99.5);
    pick(200.0);
    return void;
end fn.main
