// trace: D57
import std;
import os;

struct reading:
    celsius: i32;
    label: str;
end struct reading

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $r: reading set_to reading[celsius set_to 21, label set_to "kitchen"];
    std::io::fmt_print(Lend.mut $out, "{}: {}{newln}", $r.label, $r.celsius);
    return void;
end fn.main
