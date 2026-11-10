// `on.target`'s narrowings, like `.os`, aren't designed.
// trace: D233
import std;

fn.main [none] -> void:
    let $a: i64 = 6;
    let $b: i64 = 7;
    on.target.os linux:
        std::io::fmt_print("linux{newln}");
    end on.target
    return void;
end fn.main
