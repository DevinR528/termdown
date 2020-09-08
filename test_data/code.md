```rust
pub struct Test(pub String);

#[cfg(target_os = "windows")]
unsafe fn heap_size_of_impl(mut ptr: *const c_void) -> usize {
    let heap = GetProcessHeap();

    if HeapValidate(heap, 0, ptr) == 0 {
        ptr = *(ptr as *const *const c_void).offset(-1);
    }

    HeapSize(heap, 0, ptr) as usize
}
```