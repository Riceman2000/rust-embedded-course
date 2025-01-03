# Heap allocatioon
This example builds off the last by adding a heap allocator. This allows the use of the `Vec` construct as well as things like `String`s if properly configured. With a `global_allocator` we can use these structs but they need to be manually brought into scope because they are not imported by default by the `std` library.
