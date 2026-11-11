// trace: D149
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

type.new wrapped = reading;

fn take [$w: Given wrapped] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $r: reading = Give $w as reading;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $r.celsius);
    return void;
end fn take

fn make [$r: Given reading] -> wrapped:
    return Give $r as wrapped;
end fn make

fn.main [none] -> void:
    let $a: reading = reading[celsius = 1];
    let $b: reading = reading[celsius = 2];
    take(Give $a as wrapped);
    take(make(Copy $b));
    return void;
end fn.main
