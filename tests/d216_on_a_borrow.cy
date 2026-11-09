// trace: D216
@config(default);
import std;

struct reading:
    celsius: i32;
end struct reading

fn.main [] -> void:
    let $r: reading = reading[celsius = 1];
    let $v := Lend $r;
    return void;
end fn.main
