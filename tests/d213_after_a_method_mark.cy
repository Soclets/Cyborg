// No space follows `%`.
// trace: D213
import std;

struct reading:
    celsius: i32;
end struct reading

fn reading.%shown [$r: reading] returns i32:
    return $r.celsius;
end fn reading.%shown

fn.main [none] returns void:
    let $r: reading set_to reading[celsius set_to 1];
    let $n: i32 set_to $r.% shown();
    return void;
end fn.main
