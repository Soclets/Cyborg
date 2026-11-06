// trace: D2
import std;

struct reading:
    celsius: i32;
end struct reading

fn read [$r: reading] -> i32:
    return $r.celsius;
end fn read

fn.main [] -> void:
    let $r: reading = reading[celsius = 21];
    let $b: Lend from $r reading = Lend $r;
    std::println($b.celsius as str);
    std::println(read(Lend $b) as str);
    std::println($r.celsius as str);
    return void;
end fn.main
