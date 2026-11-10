// trace: D210, D194
import std;

fn.main [none] -> void:
    let $t: i32 = 21;
    std::io::fmt_print("temp: {$t} C{newln}", $t);
    return void;
end fn.main
