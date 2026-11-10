// trace: D211
import std;

struct reading:
    celsius: i32;
end struct reading

fn reading.%shown [$r: reading] -> i32:
    return $r.celsius;
end fn reading.%shown

fn reading.%%warm_up [$r: Lent.mut reading, $by: i32] -> void:
    change $r.celsius = $r.celsius + $by;
    return void;
end fn reading.%%warm_up

fn.main [none] -> void:
    let mut $r: reading = reading[celsius = 20];
    $r.%%warm_up(3);
    std::io::fmt_print("{}{newln}", $r.%shown());
    return void;
end fn.main
