// trace: D130, D4, D5, D8
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

fn reading.%%warm_up [$r: Lent.mut reading, $by: i32] returns void:
    labels [$by as by];
    change $r.celsius = $r.celsius + $by;
    return void;
end fn reading.%%warm_up

fn reading.%show [$r: reading] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $r.celsius);
    return void;
end fn reading.%show

fn.pure reading.%doubled [$r: reading] returns i32:
    return $r.celsius x 2;
end fn.pure reading.%doubled

fn make [none] returns reading:
    return reading[celsius = 5];
end fn make

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $r: reading = reading[celsius = 21];
    $r.%%warm_up(by: 2);
    $r.%show();
    $r.%%warm_up(by: 1);
    $r.%show();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $r.%doubled());
    make().%show();
    return void;
end fn.main
