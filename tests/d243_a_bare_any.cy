// `any` says which kind it is: `any.type` or `any.unit`.
// trace: D243
import std;

fam.unit length:
    length.meters: base;
    length.feet: base;
end fam.unit length

fam.unit time:
    time.seconds: base;
end fam.unit time

fn keep<?item: any> [$a: Given ?item] -> ?item:
    return Give $a;
end fn keep

fn.main [none] -> void:
    return void;
end fn.main
