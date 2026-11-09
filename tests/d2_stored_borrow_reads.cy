// trace: D2
import std;
@config(default);

struct reading:
    celsius: i32;
end struct reading

fn read [$r: reading] -> i32:
    return $r.celsius;
end fn read

fn.main [] -> void:
    let $r: reading = reading[celsius = 21];
    let $b: Lend from $r reading = Lend $r;
    std::io::fmt_print("{}{newln}", $b.celsius);
    std::io::fmt_print("{}{newln}", read(Lend $b));
    std::io::fmt_print("{}{newln}", $r.celsius);
    return void;
end fn.main
