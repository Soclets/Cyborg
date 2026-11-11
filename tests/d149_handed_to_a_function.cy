// trace: D149
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

type.new wrapped set_to reading;

fn take [$w: Given wrapped] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $r: reading set_to Give $w as reading;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $r.celsius);
    return void;
end fn take

fn make [$r: Given reading] returns wrapped:
    return Give $r as wrapped;
end fn make

fn.main [none] returns void:
    let $a: reading set_to reading[celsius set_to 1];
    let $b: reading set_to reading[celsius set_to 2];
    take(Give $a as wrapped);
    take(make(Copy $b));
    return void;
end fn.main
