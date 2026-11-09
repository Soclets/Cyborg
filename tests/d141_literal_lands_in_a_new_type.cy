// trace: D141, D134
import std;
@config(default);

type.new user_id = i64;
type.new label = str;
type.new flag = bool;
type.new row = std::array<i32>[len: 2 items];

fam.type.new ids:
    ids.user = i64;
    ids.order = i64;
end fam.type.new ids

fn first_user [] -> user_id:
    return 5;
end fn first_user

fn same [$u: user_id] -> user_id:
    return $u;
end fn same

fn.main [] -> void:
    let $u: user_id = 5;
    let $l: label = "x";
    let $f: flag = true;
    let $r: row = [1, 2];
    let $i: ids.user = 9;
    let $back: i64 = first_user() as i64;
    std::println($back as str);
    let $again: i64 = same(7) as i64;
    std::println($again as str);
    let $whole: i64 = $i as i64;
    std::println($whole as str);
    std::println("all built");
    return void;
end fn.main
