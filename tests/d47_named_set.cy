// trace: D47
import std;
import os;

fam.failure file_failed:
    file_failed.not_found;
    file_failed.denied;
end fam.failure file_failed

fn load [$which: i32] returns i32 fails [file_failed, bad_syntax]:
    if $which == 1:
        fail file_failed.not_found;
    end if
    if $which == 2:
        fail denied;
    end if
    if $which == 3:
        fail bad_syntax;
    end if
    return 0;
end fn load

fn name_of [$which: i32] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $n: i32 = load($which) otherwise:
        case not_found:
            std::io::fmt_print(Lend.mut $out, "not found{newln}");
            return void;
        case file_failed.denied:
            std::io::fmt_print(Lend.mut $out, "denied{newln}");
            return void;
        case bad_syntax:
            std::io::fmt_print(Lend.mut $out, "bad syntax{newln}");
            return void;
    end otherwise
    std::io::fmt_print(Lend.mut $out, "fine {}{newln}", $n);
    return void;
end fn name_of

fn.main [none] returns void:
    name_of(1);
    name_of(2);
    name_of(3);
    name_of(4);
    return void;
end fn.main
