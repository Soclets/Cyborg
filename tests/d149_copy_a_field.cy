// trace: D149
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

type.new wrapped set_to reading;

struct holder:
    inner: reading;
end struct holder

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $h: holder set_to holder[inner set_to reading[celsius set_to 1]];
    let $w: wrapped set_to Copy $h.inner as wrapped;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $h.inner.celsius);
    return void;
end fn.main
