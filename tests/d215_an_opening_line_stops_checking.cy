// How checking carries on past an error in a block's opening line isn't decided, so it stops there.
// trace: D215
import std;

fn.main [none] returns void:
    let $a: i32 set_to 1;
    if $a below "x":
        let $b: i64 set_to $a;
    end if
    let $c: i64 set_to $a;
    return void;
end fn.main
