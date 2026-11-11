// No space comes before a postfix `?`.
// trace: D213
import std;

fn inner [none] -> i32 fails [lost[where: i32]]:
    fail lost[where = 9];
end fn inner

fn outer [none] -> i32 fails [lost[where: i32]]:
    return inner() ?;
end fn outer

fn.main [none] -> void:
    return void;
end fn.main
