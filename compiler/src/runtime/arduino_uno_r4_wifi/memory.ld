/* The Arduino Uno R4 WiFi's memory, for Cyborg programs (D261, D281). Its RA4M1 has 256 KB of flash at 0 and 32 KB
   of RAM at 0x20000000. Arduino's bootloader keeps the first 16 KB of flash, so a program starts at 0x4000, where the
   bootloader jumps. Not yet run on the real board, so treat it as untested. */

MEMORY
{
  FLASH (rx)  : ORIGIN = 0x00004000, LENGTH = 0x0003C000
  RAM   (rwx) : ORIGIN = 0x20000000, LENGTH = 0x00008000
}

ENTRY(__cyborg_reset)

SECTIONS
{
  .isr_vector : { KEEP(*(.isr_vector)) } > FLASH
  .text : { *(.text*) *(.rodata*) } > FLASH
  .ARM.exidx : { *(.ARM.exidx*) } > FLASH

  .data : ALIGN(4)
  {
    __cyborg_data_start = .;
    *(.data*)
    . = ALIGN(4);
    __cyborg_data_end = .;
  } > RAM AT > FLASH
  __cyborg_data_load = LOADADDR(.data);

  .bss (NOLOAD) : ALIGN(4)
  {
    __cyborg_bss_start = .;
    *(.bss*)
    *(COMMON)
    . = ALIGN(4);
    __cyborg_bss_end = .;
  } > RAM

  __cyborg_stack_top = ORIGIN(RAM) + LENGTH(RAM);
}
