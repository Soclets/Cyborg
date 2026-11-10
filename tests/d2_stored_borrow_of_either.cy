// trace: D2
import std;

struct reading:
    celsius: i32;
end struct reading

fn.main [none] -> void:
    let $a: reading = reading[celsius = 1];
    let $c: reading = reading[celsius = 2];
    let $b: Lend from <$a, $c> reading = Lend $c;
    std::io::fmt_print("{}{newln}", $b.celsius);
    return void;
end fn.main
