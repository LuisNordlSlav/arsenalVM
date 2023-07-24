this is a hobby virtual machine project.

to get started using the arsenal vm, clone the repository and build with cargo.

assembly syntax:

instruction arg1 arg2 argN;
label name:
() var_name = arg1 arg2 argN;
(length_capture) var_name = arg1 arg2 argN;

args:
    Hex: 0x00
    Number: 7, -7, +7
    String: "Hello, world!"
    Label: &name:start->stop=>offset
    Size: $name:start->stop
    LargeNumber: #num:start->stop

start and stop default to 0 and 7 respectively and are optional

example program:

JumpTo &_start;

label _data:
    (length) greeting = "Hello, world!" 0 /*this is null byte to turbinate string*/;

label _start:
    LoadRegisterLong 0 &greeting;
    SysCall PrintCString;

label _end:
    Halt;


to get a list of instructions and syscalls go to ./arsenal-globals/src/lib.rs where they are defined in an enum. ignore __end__ it is not an instruction.
