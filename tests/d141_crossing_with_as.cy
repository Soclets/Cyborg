// trace: D141
import std;

type.new user_id = i64;

fn.main [none] -> void:
    let $n: i64 = 5;
    let $u: user_id = $n as user_id;
    let $m: i64 = $u as i64;
    std::io::fmt_print("{}{newln}", $m);
    let $bounded: i64[min: 0, max: 9] = 3;
    let $v: user_id = $bounded as user_id;
    std::io::fmt_print("{}{newln}", ($v as i64));
    return void;
end fn.main
