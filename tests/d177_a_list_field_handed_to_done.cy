// A field can be moved out of something you own, so a growable list in a `must_Give` struct is
// ended by handing the field to `list.done`, and then the struct is finished.
// trace: D177, D129, D323
import std;
import os;

struct.must_Give bag:
    items: std::coll::list<i32>[len.max: inf items];
end struct.must_Give bag

fn bag_done [$b: Given bag, $alloc: Lent os::heap] -> void:
    std::coll::list.done(Give $b.items, alloc: Lend $alloc);
    finish $b;
    return void;
end fn bag_done

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $heap: os::heap = os::heap();
    let $b: bag = bag[items = std::coll::list.from([1, 2], alloc: Lend $heap)];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b.items.len as u64);
    bag_done(Give $b, Lend $heap);
    return void;
end fn.main
