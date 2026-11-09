// How checking carries on past an error in a block's opening line isn't decided, so it stops there.
// trace: D215
@config(default);
import std;

fn.main [] -> void:
    let $a: i32 = 1;
    if $a < "x":
        let $b: i64 = $a;
    end if
    let $c: i64 = $a;
    return void;
end fn.main
