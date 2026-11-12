// trace: D211
import std;

struct reading:
    celsius: i32;
end struct reading

fn reading.%shown [$r: Lent reading] -> i32:
    return $r.celsius;
end fn reading.%shown

fn.main [none] -> void:
    let $r: reading = reading[celsius = 20];
    let $n: i32 = $r.%%shown();
    return void;
end fn.main
