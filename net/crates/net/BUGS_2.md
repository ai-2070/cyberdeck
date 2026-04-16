# Known Bugs — Sweep 2

## Critical — FFI safety

### 1. Layout::array unwrap on untrusted count
- **File**: `src/ffi/mod.rs:876, 960`
- `Layout::array::<NetEvent>(count).unwrap()` panics if count overflows layout calculations. At line 960 the count comes from caller-controlled `result.count` in FFI — a corrupted or malicious value panics the process.

### 2. CString::new().unwrap_or_default() silently loses cursor
- **File**: `src/ffi/mod.rs:911`
- If the cursor string contains a null byte, this silently returns an empty CString. The caller gets an empty cursor, causing pagination to restart from the beginning instead of where it left off.

## Medium — Logic bugs

### 3. Integer division truncation in per-shard limit
- **File**: `src/consumer/merge.rs:198-199`
- `request.limit / shards.len()` truncates to 0 when limit < shard count. The `.max(1)` saves it from zero but makes pagination inefficient with many shards and small limits. Performance bug, not correctness.

### 4. Potential divide-by-zero in candidate selection
- **File**: `src/shard/mapper.rs:424`
- `candidates.len()` could theoretically be 0 if all active shards have weights that don't pass the tolerance filter. Low risk since `min_weight` is derived from the same set, but floating-point edge cases (NaN, infinity) could cause it.
