// trace: D130, D44, D96
import std;
@config(default);

struct box:
    n: i32;
end struct box

fn box.%half [$b: box] -> i32:
    fails [odd];

    if $b.n mod.trunc 2 != 0:
        fail odd;
    end if
    return $b.n div.trunc 2;
end fn box.%half

fn.main [] -> void:
    let $even: box = box[n = 4];
    let $odd: box = box[n = 3];
    std::println($even.%half() as str);
    std::println(($odd.%half() ?? -1) as str);
    let $c: i32 = $odd.%half();
    std::println("not reached");
    return void;
end fn.main
