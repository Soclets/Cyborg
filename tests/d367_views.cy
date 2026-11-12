// A view of a literal is stored in a field and returned freely, and a view of another string
// is a borrow, which writes its source when it's stored or returned. Passing one needs nothing,
// and lending any string gives one.
// trace: D367, D327, D170
import std;
import os;

struct room:
    name: str;
end struct room

fn label [none] -> str:
    return "kitchen";
end fn label

fn whole [$text: str] -> Lend from $text str:
    return Lend $text;
end fn whole

fn shout [$out: Lent.mut os::stdout.unbuffered, $text: str] -> void:
    std::io::fmt_print(Lend.mut $out, "{}!{newln}", $text);
    return void;
end fn shout

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $r: room = room[name = "hall"];
    let $owned: str[len.max: 16 bytes] = "attic";
    let $v: Lend from $owned str = Lend $owned;
    let $w: Lend from $owned str = whole(Lend $owned);
    shout(Lend.mut $out, label());
    shout(Lend.mut $out, $r.name);
    shout(Lend.mut $out, Lend $owned);
    std::io::fmt_print(Lend.mut $out, "{} {}{newln}", $v, $w);
    return void;
end fn.main
