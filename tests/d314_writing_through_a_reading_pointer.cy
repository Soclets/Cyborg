// A `ptr.raw` can only read, so writing through one is refused.
// trace: D314
fn.main [none] -> void:
    let $count: i32 = 5;
    shield poke:
        reason "testing a pointer that only reads"
        let $p: ptr.raw<i32> = ptr.raw<i32>.of(Lend $count);
        $p.%%write(val: 6);
    end shield poke
    return void;
end fn.main
