// trace: D47
import std;
import os;

fam.failure file_failed:
    file_failed.denied;
end fam.failure file_failed

fam.failure net_failed:
    net_failed.denied;
end fam.failure net_failed

fn load [none] returns i32 fails [file_failed.denied, net_failed.denied]:
    fail net_failed.denied;
end fn load

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $n: i32 = load() otherwise:
        case file_failed.denied:
            std::io::fmt_print(Lend.mut $out, "file{newln}");
            return void;
        case net_failed.denied:
            std::io::fmt_print(Lend.mut $out, "net{newln}");
            return void;
    end otherwise
    return void;
end fn.main
