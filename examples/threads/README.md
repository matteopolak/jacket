# Threads

A Thread_Group sharing out work, and a producer and consumer passing items through semaphores.
In the browser the threads take turns on one host thread; natively they are real threads.

## Map

- [main.jai](main.jai): Calls the section.
- [threads.jai](threads.jai): Picks the browser or the native version.
- [cooperative.jai](cooperative.jai): The browser version.
- [native.jai](native.jai): The native version, on the Thread module.

## Things to try

- Change the number of workers or the ring's size and see how often the producer waits.
