// The variable stays lent for as long as the pointer exists, so using it ends the pointer.
// trace: D315
fn.main [none] -> void:
    let mut $count: i32 = 5;
    shield poke:
        reason "testing a pointer's borrow"
        let $p: ptr.raw.mut<i32> = ptr.raw.mut<i32>.of(Lend.mut $count);
        change $count = 7;
        $p.%%write(val: 6);
    end shield poke
    return void;
end fn.main
