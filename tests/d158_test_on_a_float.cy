// trace: D158, D20
import std;

fn pick [$x: f64] -> void:
    if $x: f64[min: 100.0, max: 200.0]:
        std::println("inside");
    else:
        std::println("outside");
    end if
    return void;
end fn pick

fn.main [] -> void:
    pick(150.5);
    pick(99.5);
    pick(200.0);
    return void;
end fn.main
