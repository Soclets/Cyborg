// trace: D170
import std;

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
    let $y: reading = reading[celsius = 25];
    let $w: Lend from <$y, $x> reading = warmer(Lend $x, Lend $y);
    std::println($w.celsius as str);
    return void;
end fn.main
