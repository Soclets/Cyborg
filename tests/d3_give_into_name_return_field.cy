// Every hand-off shows, and a fresh value needs no word.
// trace: D3, D2
import std;
import os;

struct name_tag:
    text: str[len.max: 32 bytes];
end struct name_tag

fn make [$s: Given str[len.max: 32 bytes]] -> name_tag:
    let $t: name_tag = name_tag[text = Give $s];
    return Give $t;
end fn make

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $s: str[len.max: 32 bytes] = "hello";
    let $k: str[len.max: 32 bytes] = Copy $s;
    let $t: name_tag = make(Give $s);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $t.text);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $k);
    return void;
end fn.main
