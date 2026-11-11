// trace: D211
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

fn reading.%shown [$r: reading] returns i32:
    return $r.celsius;
end fn reading.%shown

fn reading.%%warm_up [$r: Lent.mut reading, $by: i32] returns void:
    change $r.celsius set_to $r.celsius + $by;
    return void;
end fn reading.%%warm_up

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let mut $r: reading set_to reading[celsius set_to 20];
    $r.%%warm_up(3);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $r.%shown());
    return void;
end fn.main
