// Crossing is the same data, so it isn't the allocating kind of `as`.
// trace: D141, D86
import std;
import os;

type.new user_id set_to i64;

fn.pure raw [$u: user_id] returns i64:
    return $u as i64;
end fn.pure raw

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", raw(3));
    return void;
end fn.main
