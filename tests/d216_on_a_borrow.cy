// trace: D216
import std;

struct reading:
    celsius: i32;
end struct reading

fn.main [none] -> void:
    let $r: reading = reading[celsius = 1];
    let $v := Lend $r;
    return void;
end fn.main
