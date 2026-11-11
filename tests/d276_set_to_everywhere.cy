// `set_to` gives every value: a `let`, a `change`, a `const`, a field when building, a default,
// a unit's conversion, a type family's member and a new type.
// trace: D276
import std;
import os;

fam.unit length:
    length.meters: base;
    length.kilometers set_to 1000 length.meters;
end fam.unit length

fam.type.alias ids:
    ids.user set_to i64;
end fam.type.alias ids

type.new percent set_to i32[min: 0, max: 100];

struct reading:
    celsius: i32;
    place: str;
end struct reading

const $limit: i32 set_to 40;

fn capped [$n: i32, $cap: i32] returns i32:
    labels [$cap as cap];
    defaults [$cap set_to 10];
    if $n above $cap:
        return $cap;
    end if
    return $n;
end fn capped

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let mut $total: i32 set_to 0;
    change $total set_to $total + $limit;
    let $r: reading set_to reading[celsius set_to 21, place set_to "hall"];
    let $far: i32[unit: length.meters] set_to 2 length.kilometers as i32[unit: length.meters];
    let $id: ids.user set_to 7;
    let $p: percent set_to 50;
    std::io::fmt_print(Lend.mut $out, "{$total} {$r.celsius} {$far, unit}{newln}");
    std::io::fmt_print(Lend.mut $out, "{} {}{newln}", capped(25), capped(25, cap: 30));
    return void;
end fn.main
