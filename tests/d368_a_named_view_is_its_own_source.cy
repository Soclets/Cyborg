// A view passes with no word, so a function returning a borrow of one takes its source from the
// view named at the call, as in 2.9's `first_word`.
// trace: D368, D170
import std;
import os;

fn either [$a: str, $b: str, $first: bool] -> Lend from <$a, $b> str:
    if $first:
        return Lend $a;
    end if
    return Lend $b;
end fn either

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $left: str = "left";
    let $right: str = "right";
    let $picked: Lend from <$left, $right> str = either($left, $right, false);
    std::io::fmt_print(Lend.mut $out, "{$picked}{newln}");
    return void;
end fn.main
