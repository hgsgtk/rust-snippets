# Outline

# Performance
## Stdin
### Before
```
$ rustc main.rs -O && yes 1 | head -n 30000000 | time ./main
        3.98 real         3.96 user         0.00 sys
```

### After
```
$ rustc main.rs -O && yes 1 | head -n 30000000 | time ./main
        2.79 real         2.48 user         0.01 sys
```
