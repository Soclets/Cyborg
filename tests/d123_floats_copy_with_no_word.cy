// trace: D123
import std;

fn twice [$x: f64] -> f64:
    return $x + $x;
end fn twice

fn.main [] -> void:
    let $a: f64 = 0.5;
    let $b: f64 = $a;
    let $c: f64 = twice($a);
    if $c == 1.0 and $b == 0.5:
        std::println("copied");
    end if
    return void;
end fn.main
