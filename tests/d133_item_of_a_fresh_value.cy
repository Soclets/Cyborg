// trace: D133
import std;
import os;

fn make [none] returns std::array<i32>[len: 2 items]:
    return [4, 5];
end fn make

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", make()[pos: 2]);
    return void;
end fn.main
