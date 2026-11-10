// trace: D211
import std;

struct reading:
    celsius: i32;
end struct reading

fn reading.%%shown [$r: reading] -> i32:
    return $r.celsius;
end fn reading.%%shown

fn.main [none] -> void:
    return void;
end fn.main
