// trace: D170, D169
import std;
@config(default);

struct reading:
    celsius: i32;
end struct reading

fn warmer [$a: reading, $b: reading] -> Lend from <$a, $b> reading:
    if $a.celsius > $b.celsius:
        return Lend $a;
    end if
    return Lend $b;
end fn warmer

fn.main [] -> void:
    let $x: reading = reading[celsius = 21];
    let mut $y: reading = reading[celsius = 25];
    let $w: Lend from <$x, $y> reading = warmer(Lend $x, Lend $y);
    std::println($w.celsius as str);
    change $y.celsius = 0;
    std::println($y.celsius as str);
    return void;
end fn.main
