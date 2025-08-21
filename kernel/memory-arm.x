MEMORY {
    RAM : ORIGIN = 0x40000000, LENGTH = 128M
}

/* Stack grows downward from end of RAM */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);
