// trace: D141, D134
import std;
import os;

type.new user_id set_to i64;
type.new label set_to str;
type.new flag set_to bool;
type.new row set_to std::array<i32>[len: 2 items];

fam.type.new ids:
    ids.user set_to i64;
    ids.order set_to i64;
end fam.type.new ids

fn first_user [none] returns user_id:
    return 5;
end fn first_user

fn same [$u: user_id] returns user_id:
    return $u;
end fn same

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $u: user_id set_to 5;
    let $l: label set_to "x";
    let $f: flag set_to true;
    let $r: row set_to [1, 2];
    let $i: ids.user set_to 9;
    let $back: i64 set_to first_user() as i64;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $back);
    let $again: i64 set_to same(7) as i64;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $again);
    let $whole: i64 set_to $i as i64;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $whole);
    std::io::fmt_print(Lend.mut $out, "all built{newln}");
    return void;
end fn.main
