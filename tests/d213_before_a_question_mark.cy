// No space comes before a postfix `?`.
// trace: D213
import std;

fn inner [none] returns i32 fails [lost[where: i32]]:
    fail lost[where set_to 9];
end fn inner

fn outer [none] returns i32 fails [lost[where: i32]]:
    return inner() ?;
end fn outer

fn.main [none] returns void:
    return void;
end fn.main
