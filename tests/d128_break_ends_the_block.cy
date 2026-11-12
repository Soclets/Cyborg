// Giving away right before a `break` is fine: the second time round can't happen.
// trace: D128, D2

fn eat [$s: Given str[len.max: 32 bytes]] -> void:
    return void;
end fn eat

fn.main [none] -> void:
    let $s: str[len.max: 32 bytes] = "a";
    for $i in range[from: 1, to: 3]:
        eat(Give $s);
        break;
    end for
    return void;
end fn.main
