// A held `Lend` only stops changes, so reading `$r` is fine.
// trace: D169
import std;
@config(default);

struct reading:
    celsius: i32;
    time: i32;
end struct reading

fn.main [] -> void:
    let mut $r: reading = reading[celsius = 21, time = 3600];
    let $lock: Lend.held from $r reading = Lend $r;
    std::println($r.celsius as str);
    return void;
end fn.main
