### To generate new expected images:

```bash
$env:UPDATE_EXPECTED_AFTER_IMAGES=1; 
cargo test; 
$env:UPDATE_EXPECTED_AFTER_IMAGES=0
```