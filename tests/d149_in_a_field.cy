// trace: D149
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

type.new wrapped set_to reading;

struct holder:
    inner: wrapped;
end struct holder

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: reading set_to reading[celsius set_to 1];
    let $h: holder set_to holder[inner set_to Give $a as wrapped];
    std::io::fmt_print(Lend.mut $out, "ok{newln}");
    return void;
end fn.main
