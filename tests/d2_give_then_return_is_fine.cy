// Giving inside a block that leaves doesn't touch the path that carries on.
// trace: D2
import std;

struct box:
    n: i32;
end struct box

fn eat [$b: Given box] -> i32:
    return $b.n;
end fn eat

fn pick [$b: Given box, $early: bool] -> i32:
    if $early:
        return eat(Give $b);
    end if
    return eat(Give $b) + 1;
end fn pick

fn.main [none] -> void:
    std::io::fmt_print("{}{newln}", pick(box[n = 1], 1 < 2));
    std::io::fmt_print("{}{newln}", pick(box[n = 1], 2 < 1));
    return void;
end fn.main
