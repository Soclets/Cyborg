// Line breaks between things aren't judged, since indentation isn't decided.
// trace: D213
@config(default);
import std;

fn.main [] -> void:
    let $a: i32 = 1;
    let $b: i32 = $a
        + 2;
    std::io::fmt_print(
        "{} {}{newln}",
        $a,
        $b,
    );
    return void;
end fn.main
