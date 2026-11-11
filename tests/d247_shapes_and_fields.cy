// Named holes take shapes after a comma and read fields, so values go straight in the text.
// trace: D247, D210
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $name: str set_to "rope";
    let $count: i32 set_to 3;
    let $price: f64.stable set_to 12.5;
    let $r: reading set_to reading[celsius set_to 21];
    std::io::fmt_print(Lend.mut $out, "{$name}: {$count} at ${$price, places: 2}, {$r.celsius} C{newln}");
    std::io::fmt_print(Lend.mut $out, "[{$count, align: right, width: 4 cols}]{newln}");
    return void;
end fn.main
