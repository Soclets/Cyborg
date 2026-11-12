// `.%moved` always says its unit, so a bare `offset:` is refused.
// trace: D316
fn.main [none] -> void:
    shield move:
        reason "testing a move"
        let $reg: ptr.raw.mut<u32> = ptr.raw.mut<u32>.at(address: 1073999872);
        let $next: ptr.raw.mut<u32> = $reg.%moved(offset: 1);
    end shield move
    return void;
end fn.main
