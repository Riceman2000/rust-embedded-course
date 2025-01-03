# Minimal Flash
This example is the minimum needed to properly initialize the microcontroller, including the second stage boot loader.

It should be noted that the second stage boot loader is not strictly necessary but if we want to load anything on the flash memory chip then we need to ensure it is initialized. The RP2040 itself does not store the program being executed.
