// No space comes before a postfix `?`.
// trace: D213
@config(default);
import std;

fn inner [] -> i32 fails [lost[where: i32]]:
    fail lost[where = 9];
end fn inner

fn outer [] -> i32 fails [lost[where: i32]]:
    return inner() ?;
end fn outer

fn.main [] -> void:
    return void;
end fn.main
