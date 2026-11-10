// Named holes take shapes after a comma and read fields, so values go straight in the text.
// trace: D247, D210
@config(default);
import std;

struct reading:
    celsius: i32;
end struct reading

fn.main [] -> void:
    let $name: str = "rope";
    let $count: i32 = 3;
    let $price: f64 = 12.5;
    let $r: reading = reading[celsius = 21];
    std::io::fmt_print("{$name}: {$count} at ${$price, places: 2}, {$r.celsius} C{newln}");
    std::io::fmt_print("[{$count, align: right, width: 4 cols}]{newln}");
    return void;
end fn.main
