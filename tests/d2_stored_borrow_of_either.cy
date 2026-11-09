// trace: D2
import std;
@config(default);

struct reading:
    celsius: i32;
end struct reading

fn.main [] -> void:
    let $a: reading = reading[celsius = 1];
    let $c: reading = reading[celsius = 2];
    let $b: Lend from <$a, $c> reading = Lend $c;
    std::println($b.celsius as str);
    return void;
end fn.main
