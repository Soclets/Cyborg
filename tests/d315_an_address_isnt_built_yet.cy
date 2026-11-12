// A pointer from `.at(address:)` points at hardware or a memory map, which the oracle doesn't
// have, so reading through one isn't built here.
// trace: D315, D317, D286
fn.main [none] -> void:
    shield blink:
        reason "a hardware register"
        let $led: ptr.volatile.mut<u32> = ptr.volatile.mut<u32>.at(address: 1073999872);
        $led.%%write(val: 1);
    end shield blink
    return void;
end fn.main
