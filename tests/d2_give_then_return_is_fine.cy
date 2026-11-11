// Giving inside a block that leaves doesn't touch the path that carries on.
// trace: D2
import std;
import os;

struct box:
    n: i32;
end struct box

fn eat [$b: Given box] returns i32:
    return $b.n;
end fn eat

fn pick [$b: Given box, $early: bool] returns i32:
    if $early:
        return eat(Give $b);
    end if
    return eat(Give $b) + 1;
end fn pick

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", pick(box[n set_to 1], 1 below 2));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", pick(box[n set_to 1], 2 below 1));
    return void;
end fn.main
