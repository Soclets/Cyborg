// Every hand-off shows, and a fresh value needs no word.
// trace: D3, D2
import std;
import os;

struct name_tag:
    text: str;
end struct name_tag

fn make [$s: Given str] returns name_tag:
    let $t: name_tag = name_tag[text = Give $s];
    return Give $t;
end fn make

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $s: str = "hello";
    let $k: str = Copy $s;
    let $t: name_tag = make(Give $s);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $t.text);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $k);
    return void;
end fn.main
