// trace: D149
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

type.new wrapped set_to reading;

fn look [$r: reading] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $w: wrapped set_to Copy $r as wrapped;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $r.celsius);
    return void;
end fn look

fn.main [none] returns void:
    let $a: reading set_to reading[celsius set_to 5];
    look(Lend $a);
    return void;
end fn.main
