// Blinks the Uno R4 WiFi's built-in LED, on pin 13, which is the RA4M1's port 1, pin 2 (P102).
// The register addresses come from Renesas's own register map, R7FA4M1AB.svd.
// No `module` line yet: the compiler is holding modules (D412 to D415) until the tests have them.
// trace: D252, D253, D317

fn.main [none] -> void:
    shield blink:
        reason "the LED is switched by writing port 1's registers on the RA4M1"
        // PCNTR1: bits 0 to 15 set each pin's direction, 1 for output.
        let $control: ptr.volatile.mut<u32> = ptr.volatile.mut<u32>.at(address: hex 4004_0020);
        // PCNTR3: bits 0 to 15 switch a pin on, bits 16 to 31 switch it off.
        let $switch: ptr.volatile.mut<u32> = ptr.volatile.mut<u32>.at(address: hex 4004_0028);
        // P102PFS: the pin's own setting. Arduino's bootloader leaves it handed to the SPI clock
        // (PMR set), which makes the port ignore it, so it goes back to plain output: PDR, bit 2.
        // PFS registers are write-protected, unlocked through PWPR: clear B0WI, then set PFSWE.
        let $protect: ptr.volatile.mut<u8> = ptr.volatile.mut<u8>.at(address: hex 4004_0d03);
        let $pin: ptr.volatile.mut<u32> = ptr.volatile.mut<u32>.at(address: hex 4004_0848);
        $protect.%write(val: hex 0);
        $protect.%write(val: hex 40);
        $pin.%write(val: hex 4);
        $protect.%write(val: hex 0);
        $protect.%write(val: hex 80);
        $control.%write(val: $control.%read() or.bit hex 4);
        forever:
            $switch.%write(val: hex 4);
            for $i in range[from: 1, to: 3000000]:
                let $wait: u32 = $control.%read();
            end for
            $switch.%write(val: hex 4_0000);
            for $i in range[from: 1, to: 3000000]:
                let $wait: u32 = $control.%read();
            end for
        end forever
    end shield blink
    return void;
end fn.main
